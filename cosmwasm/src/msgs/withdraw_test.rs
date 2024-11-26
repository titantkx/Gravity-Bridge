use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, MockApi},
    Addr, Attribute, Coin, Empty, IbcMsg, IbcQuery, MemoryStorage, Uint128,
};

use cw_multi_test::{
    App, AppBuilder, BankKeeper, ContractWrapper, Executor, FailingModule, WasmKeeper,
};
use lazy_static::lazy_static;

use crate::{
    contract::instantiate,
    execute, query,
    types::msg::{AddTKXIbcDenomMsg, ExecuteMsg, InstantiateMsg, WithdrawMsg},
    TKX_NATIVE_DENOM,
};
use crate::{msgs::test_helper::MockIbcModule, types::msg::IbcAutoSendEthMemo};

use super::test_helper::MockIbcKeeper;

lazy_static! {
    static ref ADMIN: Addr = mock_dependencies()
        .api
        .with_prefix("titan")
        .addr_make("admin");
    static ref USER: Addr = mock_dependencies()
        .api
        .with_prefix("titan")
        .addr_make("user");
}

static OTHER_DENOM: &str = "other";

fn init_test() -> (
    App<
        BankKeeper,
        MockApi,
        MemoryStorage,
        FailingModule<Empty, Empty, Empty>,
        WasmKeeper<Empty, Empty>,
        FailingModule<Empty, Empty, Empty>,
        FailingModule<Empty, Empty, Empty>,
        MockIbcKeeper<IbcMsg, IbcQuery, Empty>,
    >,
    Addr,
) {
    let api = MockApi::default().with_prefix("titan");
    let ibc = MockIbcModule::new();

    let mut app =
        AppBuilder::default()
            .with_api(api)
            .with_ibc(ibc)
            .build(|router, api, storage| {
                router
                    .bank
                    .init_balance(
                        storage,
                        &ADMIN,
                        vec![
                            Coin {
                                denom: TKX_NATIVE_DENOM.to_string(),
                                amount: 1_000_000u128.into(),
                            },
                            Coin {
                                denom: OTHER_DENOM.to_string(),
                                amount: 1_000_000u128.into(),
                            },
                        ],
                    )
                    .unwrap();
                router
                    .bank
                    .init_balance(
                        storage,
                        &USER,
                        vec![
                            Coin {
                                denom: TKX_NATIVE_DENOM.to_string(),
                                amount: 1_000_000u128.into(),
                            },
                            Coin {
                                denom: OTHER_DENOM.to_string(),
                                amount: 1_000_000u128.into(),
                            },
                        ],
                    )
                    .unwrap();
                api.with_prefix("titan");
            });

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    let contract_addr = app
        .instantiate_contract(
            code_id,
            ADMIN.clone(),
            &InstantiateMsg { admin: None },
            &[],
            "tkx-exchange",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    // config tkx ibc token info
    let msg: ExecuteMsg = ExecuteMsg::AddTKXIbcDenom(AddTKXIbcDenomMsg {
        chain_prefix: "eth".to_string(),
        denom: "ibc/123".to_string(),
        channel_id: "channel-0".to_string(),
    });

    let res = app
        .execute_contract(ADMIN.clone(), contract_addr.clone(), &msg, &[])
        .unwrap();

    let attributes = res.custom_attrs(1);
    assert!(attributes.contains(&Attribute {
        key: "method".to_string(),
        value: "add_tkx_ibc_token_info".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "chain_prefix".to_string(),
        value: "eth".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "denom".to_string(),
        value: "ibc/123".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "channel_id".to_string(),
        value: "channel-0".to_string()
    }));

    (app, contract_addr)
}

#[test]
fn success_request() {
    let (mut app, contract_addr) = init_test();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x123456789".to_string(),
        forwarder: "forwarder".to_string(),
        amount: Uint128::new(100),
        bridge_fee: Uint128::new(10),
    });

    let res = app
        .execute_contract(
            USER.clone(),
            contract_addr.clone(),
            &msg,
            &coins(110, TKX_NATIVE_DENOM),
        )
        .unwrap();

    let attributes = res.custom_attrs(1);
    assert!(attributes.contains(&Attribute {
        key: "method".to_string(),
        value: "withdraw".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "chain_prefix".to_string(),
        value: "eth".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "recipient".to_string(),
        value: "0x123456789".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "forwarder".to_string(),
        value: "forwarder".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "total_amount".to_string(),
        value: "110".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "amount".to_string(),
        value: "100".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "bridge_fee".to_string(),
        value: "10".to_string()
    }));

    let captured_calls = app.router().ibc.get_captured_calls();

    let calls = captured_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    let (sender, msg) = &calls[0];
    assert_eq!(sender.to_string(), contract_addr.to_string());
    if let IbcMsg::Transfer {
        channel_id,
        to_address,
        amount,
        timeout,
        memo,
    } = msg
    {
        assert_eq!(channel_id, "channel-0");
        assert_eq!(to_address, "forwarder");
        assert_eq!(amount.denom, "ibc/123");
        assert_eq!(amount.amount, Uint128::new(110));
        // timeout more than 29 days from now
        let current_time = app.block_info().time;
        let expected_timeout = current_time.plus_days(29);
        assert!(timeout.timestamp() > Some(expected_timeout));

        assert!(memo.is_some());
        println!("memo {:?}", memo);
        let memo_data: IbcAutoSendEthMemo = serde_json::from_str(memo.as_ref().unwrap()).unwrap();
        assert_eq!(memo_data.send_to_eth.evm_chain_prefix, "eth");
        assert_eq!(memo_data.send_to_eth.eth_dest, "0x123456789");
        assert_eq!(memo_data.send_to_eth.amount, Uint128::new(100));
        assert_eq!(memo_data.send_to_eth.bridge_fee, Uint128::new(10));
        assert_eq!(memo_data.ibc_callback, contract_addr.to_string());
    } else {
        panic!("Unexpected message type");
    }
}

#[test]
fn not_accept_other_token() {
    let (mut app, contract_addr) = init_test();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x123456789".to_string(),
        forwarder: "forwarder".to_string(),
        amount: Uint128::new(100),
        bridge_fee: Uint128::new(10),
    });

    let res = app.execute_contract(
        USER.clone(),
        contract_addr.clone(),
        &msg,
        &coins(110, OTHER_DENOM),
    );

    assert!(res.is_err());
}
