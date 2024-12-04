use std::{convert::TryInto, time::Duration};

use clarity::Address as EthAddress;
use deep_space::{Address as CosmosAddress, Contact, Msg, PrivateKey};
use gravity_proto::{
    cosmos_sdk_proto::{
        cosmos::bank::v1beta1::query_client::QueryClient as BankQueryClient,
        ibc::{
            applications::transfer::v1::query_client::QueryClient as IbcTransferQueryClient,
            core::channel::v1::query_client::QueryClient as IbcChannelQueryClient,
        },
    },
    gravity::query_client::QueryClient as GravityQueryClient,
};
use ibc_relayer::chain;
use sha2::{Digest, Sha256};
use tkx_exchange_contract::types::{
    msg::{AddTKXIbcDenomMsg, ExecuteMsg, SetAdminMsg},
    query::{GetAdminResp, ListTxkIbcDenomResp},
};
use tonic::transport::Channel;
use wasmd_proto_titan::cosmwasm::wasm::v1::{
    msg_client::MsgClient as IbcWasmMsgClient, QuerySmartContractStateRequest,
};
use wasmd_proto_titan::cosmwasm::wasm::v1::{
    query_client::QueryClient as IbcWasmQueryClient, MsgExecuteContract,
};
use web30::client::Web3;

use crate::{
    create_default_test_config, get_gravity_chain_id, get_ibc_chain_id,
    ibc_auto_forward::{get_channel_id, setup_gravity_auto_forwards},
    prepare_ibc_relayer, start_ibc_relayer, start_orchestrators,
    types::IBCPrivateKey,
    wait_for_number_blocks, ValidatorKeys, COSMOS_NODE_GRPC, EVM_CHAIN_PREFIX,
    GRAVITY_DENOM_SEPARATOR, IBC_ADDRESS_PREFIX, IBC_NODE_GRPC,
};

pub async fn ibc_auto_forward_tkx_test(
    web30: &Web3,
    gravity_client: GravityQueryClient<Channel>,
    contact: &Contact,
    ibc_contact: &Contact,
    keys: Vec<ValidatorKeys>,
    ibc_keys: Vec<IBCPrivateKey>,
    erc20_address: EthAddress,
    gravity_address: EthAddress,
    tkx_exchange_address: CosmosAddress,
) {
    let no_relay_market_config = create_default_test_config();
    start_orchestrators(keys.clone(), gravity_address, false, no_relay_market_config).await;

    start_ibc_relayer(&contact, &ibc_contact, &keys, &ibc_keys).await;

    let gravity_channel_qc = IbcChannelQueryClient::connect(COSMOS_NODE_GRPC.as_str())
        .await
        .expect("Could not connect channel query client");
    let ibc_bank_qc = BankQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect bank query client");
    let ibc_transfer_qc = IbcTransferQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect ibc-transfer query client");
    let ibc_channel_qc = IbcChannelQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect channel query client");
    let ibc_transfer_qc = IbcTransferQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect ibc-transfer query client");
    let wasm_qc = IbcWasmQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect wasm msg client");

    // Wait for the ibc channel to be created and find the channel ids
    let channel_id_timeout = Duration::from_secs(60 * 5);
    let gravity_channel_id = get_channel_id(
        gravity_channel_qc,
        get_ibc_chain_id(),
        Some(channel_id_timeout),
    )
    .await
    .expect("Could not find gravity-test-1 channel");
    info!("Found gravity-test-1 channel id: {}", gravity_channel_id);

    let ibc_channel_id = get_channel_id(ibc_channel_qc, get_gravity_chain_id(), None)
        .await
        .expect(&format!("Could not find {} channel", get_ibc_chain_id()));
    info!("Found ibc-test-1 channel id: {}", ibc_channel_id);

    // denom at gravity chain
    let tkx_gravity_denom = format!(
        "{}{}{}",
        EVM_CHAIN_PREFIX.as_str(),
        GRAVITY_DENOM_SEPARATOR.as_str(),
        erc20_address
    );
    info!("TKX gravity denom: {}", tkx_gravity_denom);

    // denom at titan chain
    let tkx_ibc_trace = format!("transfer/{}/{}", ibc_channel_id, tkx_gravity_denom);
    let mut hasher = Sha256::new();
    hasher.update(tkx_ibc_trace.as_bytes());
    let tkx_ibc_denom = format!("ibc/{:X}", hasher.finalize());
    info!("TKX ibc denom: {}", tkx_ibc_denom);

    info!("Setup auto forward: create gov proposal to map chain prefix {} to ibc source channel id {}", *EVM_CHAIN_PREFIX, ibc_channel_id);
    setup_gravity_auto_forwards(
        contact,
        (*IBC_ADDRESS_PREFIX).clone(),
        gravity_channel_id.clone(),
        keys[0].validator_key,
        &keys,
    )
    .await;

    setup_tkx_exchange_contract(
        ibc_contact,
        wasm_qc.clone(),
        ibc_keys,
        tkx_exchange_address,
        ibc_channel_id,
        tkx_ibc_denom.clone(),
    )
    .await;
}

pub async fn setup_tkx_exchange_contract(
    ibc_contact: &Contact,
    ibc_wasm_qc: IbcWasmQueryClient<Channel>,
    ibc_keys: Vec<IBCPrivateKey>,
    tkx_exchange_address: CosmosAddress,
    ibc_channel_id: String,
    tkx_ibc_denom: String,
) {
    get_tkx_contract_admin(ibc_wasm_qc.clone(), tkx_exchange_address.clone()).await;

    add_tkx_ibc_denom(
        ibc_contact,
        ibc_wasm_qc.clone(),
        ibc_keys.clone(),
        tkx_exchange_address.clone(),
        ibc_channel_id.to_string(),
        tkx_ibc_denom.clone(),
    )
    .await;

    list_tkx_ibc_denoms(ibc_wasm_qc.clone(), tkx_exchange_address.clone()).await;
}

pub async fn set_contract_admin(
    ibc_contact: &Contact,
    ibc_wasm_qc: IbcWasmQueryClient<Channel>,
    ibc_keys: Vec<IBCPrivateKey>,
    tkx_exchange_address: CosmosAddress,
    new_admin: CosmosAddress,
) {
    let set_admin_msg = ExecuteMsg::SetAdmin(SetAdminMsg {
        admin: new_admin.to_string(),
    });
    let exec_msg = MsgExecuteContract {
        sender: ibc_keys[0]
            .to_address(IBC_ADDRESS_PREFIX.as_str())
            .unwrap()
            .to_string(),
        contract: tkx_exchange_address.to_string(),
        funds: vec![],
        msg: serde_json::to_vec(&set_admin_msg).unwrap(),
    };

    let msg = Msg::new("/cosmwasm.wasm.v1.MsgExecuteContract", exec_msg);
    let res = ibc_contact
        .send_message(&[msg], None, &[], None, ibc_keys[0].clone())
        .await;

    if let Err(e) = res {
        panic!("Failed to set contract admin: {:?}", e);
    }

    // wait for 3 blocks
    wait_for_number_blocks(ibc_contact, 3).await.unwrap();

    // verify the admin was set
    let admin = get_tkx_contract_admin(ibc_wasm_qc, tkx_exchange_address).await;
    assert_eq!(admin, new_admin);
}

pub async fn get_tkx_contract_admin(
    ibc_wasm_qc: IbcWasmQueryClient<Channel>,
    tkx_exchange_address: CosmosAddress,
) -> CosmosAddress {
    let mut ibc_wasm_qc = ibc_wasm_qc;
    let query = QuerySmartContractStateRequest {
        address: tkx_exchange_address.to_string(),
        query_data: br#"{"get_admin":{}}"#.to_vec(),
    };

    let data = ibc_wasm_qc.smart_contract_state(query).await.unwrap();

    let resp: GetAdminResp = serde_json::from_slice(data.into_inner().data.as_slice()).unwrap();
    println!("Got admin: {:?}", resp);

    CosmosAddress::from_bech32(resp.address).unwrap()
}

pub async fn add_tkx_ibc_denom(
    ibc_contact: &Contact,
    ibc_wasm_qc: IbcWasmQueryClient<Channel>,
    ibc_keys: Vec<IBCPrivateKey>,
    tkx_exchange_address: CosmosAddress,
    ibc_channel_id: String,
    tkx_ibc_denom: String,
) {
    // let add_tkx_msg = AddTKXIbcDenomMsg {
    //     chain_prefix: EVM_CHAIN_PREFIX.to_string(),
    //     channel_id: ibc_channel_id,
    //     denom: tkx_ibc_denom,
    // };

    let add_tkx_msg = ExecuteMsg::AddTKXIbcDenom(AddTKXIbcDenomMsg {
        chain_prefix: EVM_CHAIN_PREFIX.to_string(),
        channel_id: ibc_channel_id,
        denom: tkx_ibc_denom,
    });
    let exec_msg = MsgExecuteContract {
        sender: ibc_keys[0]
            .to_address(IBC_ADDRESS_PREFIX.as_str())
            .unwrap()
            .to_string(),
        contract: tkx_exchange_address.to_string(),
        funds: vec![],
        msg: serde_json::to_vec(&add_tkx_msg).unwrap(),
    };
    let msg = Msg::new("/cosmwasm.wasm.v1.MsgExecuteContract", exec_msg);
    let res = ibc_contact
        .send_message(&[msg], None, &[], None, ibc_keys[0].clone())
        .await;
    if let Err(e) = res {
        panic!("Failed to set contract admin: {:?}", e);
    }
    // wait for 3 blocks
    wait_for_number_blocks(ibc_contact, 3).await.unwrap();
}

pub async fn list_tkx_ibc_denoms(
    ibc_wasm_qc: IbcWasmQueryClient<Channel>,
    tkx_exchange_address: CosmosAddress,
) -> Vec<String> {
    let mut ibc_wasm_qc = ibc_wasm_qc;
    let query = QuerySmartContractStateRequest {
        address: tkx_exchange_address.to_string(),
        query_data: br#"{"list_tkx_ibc_denom":{}}"#.to_vec(),
    };

    let data = ibc_wasm_qc.smart_contract_state(query).await.unwrap();

    let resp: ListTxkIbcDenomResp =
        serde_json::from_slice(data.into_inner().data.as_slice()).unwrap();
    println!("Got resp: {:?}", resp);

    resp.denoms
}
