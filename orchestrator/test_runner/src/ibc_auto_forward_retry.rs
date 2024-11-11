use clarity::Address as EthAddress;
use deep_space::Contact;
use gravity_proto::gravity::query_client::QueryClient as GravityQueryClient;
use tonic::transport::Channel;
use web30::client::Web3;

use crate::{types::IBCPrivateKey, ValidatorKeys};

pub async fn ibc_auto_forward_retry_test(
    web30: &Web3,
    gravity_client: GravityQueryClient<Channel>,
    contact: &Contact,
    ibc_contact: &Contact,
    keys: Vec<ValidatorKeys>,
    ibc_keys: Vec<IBCPrivateKey>,
    gravity_address: EthAddress,
    erc20_address: EthAddress,
) {
}
