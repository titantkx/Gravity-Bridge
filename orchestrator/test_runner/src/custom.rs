use clarity::Address as EthAddress;
use deep_space::{Address as CosmosAddress, Contact};
use gravity_proto::gravity::query_client::QueryClient as GravityQueryClient;
use tonic::transport::Channel;
use wasmd_proto_titan::cosmwasm::wasm::v1::{
    msg_client::MsgClient as IbcWasmMsgClient, query_client::QueryClient as IbcWasmQueryClient,
};
use web30::client::Web3;

use crate::{
    ibc_auto_forward_tkx::setup_tkx_exchange_contract, types::IBCPrivateKey, ValidatorKeys,
    IBC_NODE_GRPC,
};

pub async fn custom_test(
    web30: &Web3,
    contact: &Contact,
    gravity_client: GravityQueryClient<Channel>,
    ibc_contact: &Contact,
    keys: Vec<ValidatorKeys>,
    ibc_keys: Vec<IBCPrivateKey>,
    erc20_address: EthAddress,
    gravity_address: EthAddress,
    tkx_exchange_address: Option<CosmosAddress>,
) {
    let _ = contact;
    let mut _gravity_client = gravity_client;

    let wasm_qc = IbcWasmQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect wasm msg client");
    let wasm_mc = IbcWasmMsgClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect wasm msg client");

    setup_tkx_exchange_contract(
        ibc_contact,
        wasm_qc.clone(),
        wasm_mc.clone(),
        ibc_keys,
        tkx_exchange_address.unwrap(),
    )
    .await;
}
