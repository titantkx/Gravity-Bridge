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
}
