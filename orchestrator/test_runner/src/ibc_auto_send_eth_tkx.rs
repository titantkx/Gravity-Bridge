use std::time::Duration;

use clarity::Address as EthAddress;
use deep_space::{Address as CosmosAddress, Contact, PrivateKey};
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
use num::FromPrimitive;
use num256::Uint256;
use sha2::{Digest, Sha256};
use std::ops::{Add, Mul};
use tonic::transport::Channel;
use wasmd_proto_titan::cosmwasm::wasm::v1::query_client::QueryClient as IbcWasmQueryClient;
use web30::client::Web3;

use crate::{
    bootstrapping::start_ibc_relayer,
    get_gravity_chain_id, get_ibc_chain_id,
    ibc_auto_forward::{get_channel_id, setup_gravity_auto_forwards},
    ibc_auto_forward_tkx::{setup_tkx_exchange_contract, test_tkx_ibc_auto_forward_happy_path},
    one_eth,
    types::IBCPrivateKey,
    utils::{create_default_test_config, start_orchestrators, ValidatorKeys},
    COSMOS_NODE_GRPC, EVM_CHAIN_PREFIX, GRAVITY_DENOM_SEPARATOR, IBC_ADDRESS_PREFIX, IBC_NODE_GRPC,
};

pub async fn ibc_auto_send_eth_tkx_test(
    web30: &Web3,
    gravity_contact: &Contact,
    gravity_client: GravityQueryClient<Channel>,
    ibc_contact: &Contact,
    keys: Vec<ValidatorKeys>,
    ibc_keys: Vec<IBCPrivateKey>,
    erc20_address: EthAddress,
    gravity_address: EthAddress,
    tkx_exchange_address: CosmosAddress,
) {
    let no_relay_market_config = create_default_test_config();
    start_orchestrators(keys.clone(), gravity_address, false, no_relay_market_config).await;

    start_ibc_relayer(&gravity_contact, &ibc_contact, &keys, &ibc_keys).await;

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
        gravity_contact,
        (*IBC_ADDRESS_PREFIX).clone(),
        gravity_channel_id.clone(),
        keys[0].validator_key,
        &keys,
    )
    .await;

    setup_tkx_exchange_contract(
        ibc_contact,
        wasm_qc.clone(),
        ibc_keys.clone(),
        tkx_exchange_address,
        ibc_channel_id,
        tkx_ibc_denom.clone(),
    )
    .await;

    // First send some tkx to the titan chain

    test_tkx_ibc_auto_forward_happy_path(
        web30,
        gravity_contact,
        ibc_contact,
        gravity_client,
        ibc_bank_qc,
        ibc_transfer_qc,
        keys[0].validator_key,
        gravity_address,
        erc20_address,
        tkx_exchange_address,
        ibc_keys[0].to_address(IBC_ADDRESS_PREFIX.as_str()).unwrap(),
        one_eth().mul(2u8.into()),
    )
    .await
    .expect("Failed to test tkx ibc auto forward happy path");
}
