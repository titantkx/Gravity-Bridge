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
    constant::TKX_NATIVE_DENOM,
    contract::instantiate,
    error::ContractError,
    execute, query, reply, sudo,
    types::{
        msg::{AddTKXIbcDenomMsg, ExecuteMsg, InstantiateMsg, WithdrawMsg},
        query::QueryMsg,
        state::WithdrawInfo,
        sudo::{IBCLifecycleComplete, SudoMsg},
    },
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
                                amount: (1_000_000u128 * (1e18 as u128)).into(),
                            },
                            Coin {
                                denom: OTHER_DENOM.to_string(),
                                amount: (1_000_000u128).into(),
                            },
                            Coin {
                                denom: "ibc/123".to_string(),
                                amount: (1_000_000u128).into(),
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
                                amount: (1_000_000u128 * (1e18 as u128)).into(),
                            },
                            Coin {
                                denom: OTHER_DENOM.to_string(),
                                amount: (1_000_000u128).into(),
                            },
                        ],
                    )
                    .unwrap();
                api.with_prefix("titan");
            });

    let code = ContractWrapper::new(execute, instantiate, query)
        .with_reply(reply)
        .with_sudo(sudo);
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

    // add ibc token for contract
    let contract_ibc_coin = Coin {
        denom: "ibc/123".to_string(),
        amount: (1_000_000u128).into(),
    };

    app.send_tokens(ADMIN.clone(), contract_addr.clone(), &[contract_ibc_coin])
        .unwrap();

    // * NOTE: This is a workaround to set the initial balance of the contract. avoid rust error about conflict borrowing
    // let mut storage = MemoryStorage::new();
    // std::mem::swap(&mut storage, &mut app.storage_mut());
    // // let storage = app.storage_mut();
    // app.router()
    //     .bank
    //     .init_balance(&mut storage, &contract_addr, vec![contract_ibc_coin])
    //     .unwrap();
    // std::mem::swap(&mut storage, &mut app.storage_mut());

    // config tkx ibc token info
    let msg: ExecuteMsg = ExecuteMsg::AddTKXIbcDenom(AddTKXIbcDenomMsg {
        chain_prefix: "eth".to_string(),
        address_regex: "0x[a-fA-F0-9]{40}".to_string(),
        denom: "ibc/123".to_string(),
        decimals: 8,
        channel_id: "channel-0".to_string(),
        forwarder_prefix: "gravity".to_string(),
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
        key: "address_regex".to_string(),
        value: "0x[a-fA-F0-9]{40}".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "denom".to_string(),
        value: "ibc/123".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "channel_id".to_string(),
        value: "channel-0".to_string()
    }));

    // supply tkx token into contract
    let msg = ExecuteMsg::SupplyTKXToken {};
    let res = app
        .execute_contract(
            ADMIN.clone(),
            contract_addr.clone(),
            &msg,
            &coins(1_000u128 * (1e18 as u128), TKX_NATIVE_DENOM),
        )
        .unwrap();
    let attributes = res.custom_attrs(1);
    assert!(attributes.contains(&Attribute {
        key: "method".to_string(),
        value: "supply_tkx_token".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "sender".to_string(),
        value: ADMIN.to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "amount".to_string(),
        value: (1_000u128 * (1e18 as u128)).to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "denom".to_string(),
        value: TKX_NATIVE_DENOM.to_string()
    }));

    (app, contract_addr)
}

#[test]
fn wrong_recipient_request() {
    let (mut app, contract_addr) = init_test();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "abcahihi".to_string(),
        forwarder: "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj".to_string(),
        amount: Uint128::new(100 * (1e10 as u128)),
        bridge_fee: Uint128::new(10 * (1e10 as u128)),
    });

    let res = app.execute_contract(
        USER.clone(),
        contract_addr.clone(),
        &msg,
        &coins(110 * (1e10 as u128), TKX_NATIVE_DENOM),
    );

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidRecipient {}.to_string()
    );
}

#[test]
fn wrong_forwarder_request() {
    let (mut app, contract_addr) = init_test();

    // wrong format
    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string(),
        forwarder: "abcahihi".to_string(),
        amount: Uint128::new(100 * (1e10 as u128)),
        bridge_fee: Uint128::new(10 * (1e10 as u128)),
    });

    let res = app.execute_contract(
        USER.clone(),
        contract_addr.clone(),
        &msg,
        &coins(110 * (1e10 as u128), TKX_NATIVE_DENOM),
    );

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidForwarder {}.to_string()
    );

    // correct format but wrong prefix
    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string(),
        forwarder: "noble1t33elcp8sv3fv0rydew25f9hc9f9l8snadtyq5".to_string(),
        amount: Uint128::new(100 * (1e10 as u128)),
        bridge_fee: Uint128::new(10 * (1e10 as u128)),
    });

    let res = app.execute_contract(
        USER.clone(),
        contract_addr.clone(),
        &msg,
        &coins(110 * (1e10 as u128), TKX_NATIVE_DENOM),
    );

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidForwarder {}.to_string()
    );
}

#[test]
fn can_not_withdraw_more_than_contract_ibc_balance() {
    let (mut app, contract_addr) = init_test();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string(),
        forwarder: "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj".to_string(),
        amount: Uint128::new(10 * (1e18 as u128)),
        bridge_fee: Uint128::new(1 * (1e18 as u128)),
    });

    let res = app.execute_contract(
        USER.clone(),
        contract_addr.clone(),
        &msg,
        &coins(11 * (1e18 as u128), TKX_NATIVE_DENOM),
    );

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InsufficientContractBalance {}.to_string()
    );
}

#[test]
fn success_request() {
    let (mut app, contract_addr) = init_test();

    // pre balance of USER
    let pre_withdraw_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    // pre tkx balance of contract
    let pre_withdraw_contract_balance = app
        .wrap()
        .query_balance(contract_addr.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string(),
        forwarder: "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj".to_string(),
        amount: Uint128::new(100 * (1e10 as u128)),
        bridge_fee: Uint128::new(10 * (1e10 as u128)),
    });

    let res = app
        .execute_contract(
            USER.clone(),
            contract_addr.clone(),
            &msg,
            &coins(110 * (1e10 as u128), TKX_NATIVE_DENOM),
        )
        .unwrap();

    // get `wasm` event type in res.events {ty: string}
    let reply_event = res.events.iter().find(|event| event.ty == "wasm").unwrap();

    let attributes = reply_event.attributes.clone();
    assert!(attributes.contains(&Attribute {
        key: "method".to_string(),
        value: "withdraw".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "chain_prefix".to_string(),
        value: "eth".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "channel_id".to_string(),
        value: "channel-0".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "sequence".to_string(),
        value: "1".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "recipient".to_string(),
        value: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "forwarder".to_string(),
        value: "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "total_amount".to_string(),
        value: "1100000000000".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "amount".to_string(),
        value: "1000000000000".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "bridge_fee".to_string(),
        value: "100000000000".to_string()
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
        assert_eq!(to_address, "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj");
        assert_eq!(amount.denom, "ibc/123");
        assert_eq!(amount.amount, Uint128::new(110));
        // timeout more than 29 days from now
        let current_time = app.block_info().time;
        let expected_timeout = current_time.plus_days(29);
        assert!(timeout.timestamp() > Some(expected_timeout));

        assert!(memo.is_some());
        // println!("memo {:?}", memo);
        let memo_data: IbcAutoSendEthMemo = serde_json::from_str(memo.as_ref().unwrap()).unwrap();
        assert_eq!(memo_data.send_to_eth.evm_chain_prefix, "eth");
        assert_eq!(
            memo_data.send_to_eth.eth_dest,
            "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2"
        );
        assert_eq!(memo_data.send_to_eth.amount, Uint128::new(100));
        assert_eq!(memo_data.send_to_eth.bridge_fee, Uint128::new(10));
        assert_eq!(memo_data.ibc_callback, contract_addr.to_string());
    } else {
        panic!("Unexpected message type");
    }

    // check save withdraw info
    let withdraw_info: WithdrawInfo = app
        .wrap()
        .query_wasm_smart(
            &contract_addr,
            &QueryMsg::GetWithdrawInfo {
                channel_id: "channel-0".to_string(),
                sequence: 1,
            },
        )
        .unwrap();

    assert_eq!(withdraw_info.ibc_channel_id, "channel-0");
    assert_eq!(withdraw_info.sequence, 1);
    assert_eq!(withdraw_info.chain_prefix, "eth");
    assert_eq!(withdraw_info.sender.to_string(), USER.to_string());
    assert_eq!(
        withdraw_info.recipient,
        "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2"
    );
    assert_eq!(
        withdraw_info.forwarder,
        "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj"
    );
    assert_eq!(
        withdraw_info.total_amount,
        Uint128::new(110 * (1e10 as u128))
    );
    assert_eq!(withdraw_info.amount, Uint128::new(100 * (1e10 as u128)));
    assert_eq!(withdraw_info.bridge_fee, Uint128::new(10 * (1e10 as u128)));

    // post balance of USER
    let post_withdraw_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    assert_eq!(
        pre_withdraw_user_balance.amount - Uint128::new(110 * (1e10 as u128)),
        post_withdraw_user_balance.amount
    );

    // post tkx balance of contract
    let post_withdraw_contract_balance = app
        .wrap()
        .query_balance(contract_addr.to_string(), TKX_NATIVE_DENOM)
        .unwrap();
    assert_eq!(
        pre_withdraw_contract_balance.amount + Uint128::new(110 * (1e10 as u128)),
        post_withdraw_contract_balance.amount
    );
}

#[test]
fn not_accept_other_token() {
    let (mut app, contract_addr) = init_test();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string(),
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

#[test]
fn success_request_ibc_timeout() {
    let (mut app, contract_addr) = init_test();

    // pre balance of USER
    let pre_withdraw_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string(),
        forwarder: "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj".to_string(),
        amount: Uint128::new(100 * (1e10 as u128)),
        bridge_fee: Uint128::new(10 * (1e10 as u128)),
    });

    app.execute_contract(
        USER.clone(),
        contract_addr.clone(),
        &msg,
        &coins(110 * (1e10 as u128), TKX_NATIVE_DENOM),
    )
    .unwrap();

    let post_withdraw_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    assert_eq!(
        pre_withdraw_user_balance.amount - Uint128::new(110 * (1e10 as u128)),
        post_withdraw_user_balance.amount
    );

    // trigger timeout callback
    let resp = app
        .wasm_sudo(
            contract_addr.clone(),
            &SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCTimeout {
                channel: "channel-0".to_string(),
                sequence: 1,
            }),
        )
        .unwrap();

    let attributes = resp.custom_attrs(1);
    assert!(attributes.contains(&Attribute {
        key: "method".to_string(),
        value: "refund_to_sender".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "sender".to_string(),
        value: USER.to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "chain_prefix".to_string(),
        value: "eth".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "channel_id".to_string(),
        value: "channel-0".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "sequence".to_string(),
        value: "1".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "total_amount".to_string(),
        value: "1100000000000".to_string()
    }));

    let post_callback_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    assert_eq!(
        pre_withdraw_user_balance.amount,
        post_callback_user_balance.amount
    );
}

#[test]
fn success_request_ibc_fail() {
    let (mut app, contract_addr) = init_test();

    // pre balance of USER
    let pre_withdraw_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    let msg = ExecuteMsg::Withdraw(WithdrawMsg {
        chain_prefix: "eth".to_string(),
        recipient: "0x24f1d3119CdF338eE56AC55Dd724Fe4ddb6365C2".to_string(),
        forwarder: "gravity1t33elcp8sv3fv0rydew25f9hc9f9l8sn37v5aj".to_string(),
        amount: Uint128::new(100 * (1e10 as u128)),
        bridge_fee: Uint128::new(10 * (1e10 as u128)),
    });

    app.execute_contract(
        USER.clone(),
        contract_addr.clone(),
        &msg,
        &coins(110 * (1e10 as u128), TKX_NATIVE_DENOM),
    )
    .unwrap();

    let post_withdraw_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    assert_eq!(
        pre_withdraw_user_balance.amount - Uint128::new(110 * (1e10 as u128)),
        post_withdraw_user_balance.amount
    );

    // trigger timeout callback
    let resp = app
        .wasm_sudo(
            contract_addr.clone(),
            &SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCAck {
                channel: "channel-0".to_string(),
                sequence: 1,
                success: false,
                ack: "error".to_string(),
            }),
        )
        .unwrap();

    let attributes = resp.custom_attrs(1);
    assert!(attributes.contains(&Attribute {
        key: "method".to_string(),
        value: "refund_to_sender".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "sender".to_string(),
        value: USER.to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "chain_prefix".to_string(),
        value: "eth".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "channel_id".to_string(),
        value: "channel-0".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "sequence".to_string(),
        value: "1".to_string()
    }));
    assert!(attributes.contains(&Attribute {
        key: "total_amount".to_string(),
        value: "1100000000000".to_string()
    }));

    let post_callback_user_balance = app
        .wrap()
        .query_balance(USER.to_string(), TKX_NATIVE_DENOM)
        .unwrap();

    assert_eq!(
        pre_withdraw_user_balance.amount,
        post_callback_user_balance.amount
    );
}
