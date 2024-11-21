use clarity::Address as EthAddress;
use deep_space::Contact;
use ethereum_gravity::send_to_cosmos::send_to_cosmos;
use gravity_proto::gravity::query_client::QueryClient as GravityQueryClient;
use tonic::transport::Channel;
use web30::client::Web3;

use crate::{
    create_default_test_config, get_user_key, start_orchestrators, ValidatorKeys,
    MINER_PRIVATE_KEY, OPERATION_TIMEOUT,
};

pub async fn custom_test(
    web30: &Web3,
    contact: &Contact,
    grpc_client: GravityQueryClient<Channel>,
    keys: Vec<ValidatorKeys>,
    gravity_address: EthAddress,
    erc20_address: EthAddress,
) {
    let _ = contact;
    let mut _grpc_client = grpc_client;

    let no_relay_market_config = create_default_test_config();
    start_orchestrators(keys.clone(), gravity_address, false, no_relay_market_config).await;

    // generate an address for coin sending tests, this ensures test imdepotency
    let user_keys = get_user_key(None);

    let dest = user_keys.cosmos_address;
    let amount = 100u64.into();

    // match test_erc20_deposit_result(
    //     web30,
    //     contact,
    //     &mut grpc_client,
    //     dest,
    //     gravity_address,
    //     erc20_address,
    //     100u64.into(),
    //     None,
    //     None,
    // )
    // .await
    // {
    //     Ok(_) => {
    //         info!("Successfully bridged ERC20!")
    //     }
    //     Err(_) => {
    //         panic!("Failed to bridge ERC20!")
    //     }
    // }

    // send_erc20_deposit(
    //     web30,
    //     &mut grpc_client,
    //     dest,
    //     gravity_address,
    //     erc20_address,
    //     amount,
    //     "",
    // )
    // .await
    // .expect("Failed to send erc20!");

    let tx_id = send_to_cosmos(
        erc20_address,
        gravity_address,
        amount,
        dest,
        "",
        *MINER_PRIVATE_KEY,
        None,
        web30,
        vec![],
    )
    .await
    .expect("Failed to send tokens to Cosmos");

    info!("Send to Cosmos txid: {:#066x}", tx_id);
    error!("ahihi");

    let _tx_res = web30
        .wait_for_transaction(tx_id, OPERATION_TIMEOUT, None)
        .await
        .expect("Send to cosmos transaction failed to be included into ethereum side");

    info!("Send to Cosmos tx included in block {:?}", _tx_res);
}
