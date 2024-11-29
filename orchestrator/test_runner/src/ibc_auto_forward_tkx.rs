use std::time::Duration;

use clarity::Address as EthAddress;
use deep_space::{Address as CosmosAddress, Contact};
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
use tonic::transport::Channel;
use tonic_v012::transport::Channel as ChannelV012;
use wasmd_proto_titan::cosmwasm::wasm::v1::query_client::QueryClient as IbcWasmQueryClient;
use wasmd_proto_titan::cosmwasm::wasm::v1::{
    msg_client::MsgClient as IbcWasmMsgClient, QuerySmartContractStateRequest,
};
use web30::client::Web3;

use crate::{
    create_default_test_config, get_ibc_chain_id,
    ibc_auto_forward::{get_channel_id, setup_gravity_auto_forwards},
    prepare_ibc_relayer, start_ibc_relayer, start_orchestrators,
    types::IBCPrivateKey,
    ValidatorKeys, COSMOS_NODE_GRPC, IBC_ADDRESS_PREFIX, IBC_NODE_GRPC,
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
    let wasm_qc = IbcWasmQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect wasm msg client");
    let wasm_mc = IbcWasmMsgClient::connect(IBC_NODE_GRPC.as_str())
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

    info!("Setup auto forward: create gov proposal to map chain prefix to ibc source channel id");
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
        wasm_mc.clone(),
        ibc_keys,
        tkx_exchange_address,
    )
    .await;
}

pub async fn setup_tkx_exchange_contract(
    ibc_contact: &Contact,
    ibc_wasm_qc: IbcWasmQueryClient<ChannelV012>,
    ibc_wasm_mc: IbcWasmMsgClient<ChannelV012>,
    ibc_keys: Vec<IBCPrivateKey>,
    tkx_exchange_address: CosmosAddress,
) {
    let mut ibc_wasm_qc = ibc_wasm_qc;

    let query = QuerySmartContractStateRequest {
        address: tkx_exchange_address.to_string(),
        query_data: br#"{"get_admin":{}}"#.to_vec(),
    };

    let data = ibc_wasm_qc.smart_contract_state(query).await.unwrap();
    println!("Got data: {:?}", data);
}
