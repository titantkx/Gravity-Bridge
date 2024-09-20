use crate::airdrop_proposal::wait_for_proposals_to_execute;
use crate::get_gravity_chain_id;
use crate::happy_path::send_erc20_deposit;
use crate::ibc_auto_forward;
use crate::ibc_auto_forward::get_channel_id;
use crate::ibc_auto_forward::get_ibc_balance;
use crate::signature_slashing::wait_for_height;
use crate::utils::*;
use crate::EVM_CHAIN_PREFIX;
use crate::GRAVITY_DENOM_SEPARATOR;
use crate::IBC_STAKING_TOKEN;
use crate::OPERATION_TIMEOUT;
use crate::TOTAL_TIMEOUT;
use crate::{
    get_ibc_chain_id, one_eth, ADDRESS_PREFIX, COSMOS_NODE_GRPC, IBC_ADDRESS_PREFIX, IBC_NODE_GRPC,
    STAKING_TOKEN,
};
use clarity::Address as EthAddress;
use cosmos_gravity::proposals::UPDATE_HRP_IBC_CHANNEL_PROPOSAL;
use cosmos_gravity::send::MSG_EXECUTE_IBC_AUTO_FORWARDS_TYPE_URL;
use deep_space::address::Address as CosmosAddress;
use deep_space::client::type_urls::MSG_TRANSFER_TYPE_URL;
use deep_space::error::CosmosGrpcError;
use deep_space::private_key::{CosmosPrivateKey, PrivateKey};
use deep_space::utils::decode_any;
use deep_space::utils::encode_any;
use deep_space::{Coin as DSCoin, Contact, Msg};
use gravity_proto::cosmos_sdk_proto::bech32ibc::bech32ibc::v1::UpdateHrpIbcChannelProposal;
use gravity_proto::cosmos_sdk_proto::cosmos::bank::{
    v1beta1 as Bank, v1beta1::query_client::QueryClient as BankQueryClient,
};
use gravity_proto::cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use gravity_proto::cosmos_sdk_proto::ibc::applications::transfer::v1::MsgTransfer;
use gravity_proto::cosmos_sdk_proto::ibc::applications::transfer::{
    v1 as IbcTransferV1, v1::query_client::QueryClient as IbcTransferQueryClient,
};
use gravity_proto::cosmos_sdk_proto::ibc::core::channel::v1::query_client::QueryClient as IbcChannelQueryClient;
use gravity_proto::cosmos_sdk_proto::ibc::core::channel::v1::IdentifiedChannel;
use gravity_proto::cosmos_sdk_proto::ibc::core::channel::v1::{
    QueryChannelClientStateRequest, QueryChannelsRequest,
};
use gravity_proto::cosmos_sdk_proto::ibc::lightclients::tendermint::v1::ClientState;
use gravity_proto::gravity::query_client::QueryClient as GravityQueryClient;
use gravity_proto::gravity::{
    MsgExecuteIbcAutoForwards, PendingIbcAutoForward, QueryPendingIbcAutoForwards,
};
use gravity_utils::error::GravityError;
use gravity_utils::get_with_retry::get_balances_with_retry;
use gravity_utils::num_conversion::one_atom;
use num256::Uint256;
use std::cmp::Ordering;
use std::ops::{Add, Mul};
use std::str::FromStr;
use std::time::Instant;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use tonic::transport::Channel;
use web30::client::Web3;

pub async fn ibc_auto_send_eth_test(
    web30: &Web3,
    gravity_client: GravityQueryClient<Channel>,
    gravity_contact: &Contact,
    ibc_contact: &Contact,
    keys: Vec<ValidatorKeys>,
    ibc_keys: Vec<CosmosPrivateKey>,
    gravity_address: EthAddress,
    erc20_address: EthAddress,
) {
    let no_relay_market_config = create_default_test_config();
    start_orchestrators(keys.clone(), gravity_address, false, no_relay_market_config).await;

    let gravity_channel_qc = IbcChannelQueryClient::connect(COSMOS_NODE_GRPC.as_str())
        .await
        .expect("Could not connect channel query client");
    let ibc_channel_qc = IbcChannelQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect channel query client");
    let gravity_bank_qc = BankQueryClient::connect(COSMOS_NODE_GRPC.as_str())
        .await
        .expect("Could not connect bank query client");
    let ibc_bank_qc = BankQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect bank query client");
    let gravity_transfer_qc = IbcTransferQueryClient::connect(COSMOS_NODE_GRPC.as_str())
        .await
        .expect("Could not connect ibc-transfer query client");
    let ibc_transfer_qc = IbcTransferQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect ibc-transfer query client");

    // Wait for the ibc channel to be created and find the channel ids
    let channel_id_timeout = Duration::from_secs(60 * 5);
    let gravity_channel_id = get_channel_id(
        gravity_channel_qc,
        get_ibc_chain_id(),
        Some(channel_id_timeout),
    )
    .await
    .expect("Could not find gravity-test-1 channel");
    info!(
        "Found {} channel id: {}",
        get_gravity_chain_id(),
        gravity_channel_id
    );

    let ibc_channel_id = get_channel_id(
        ibc_channel_qc,
        get_gravity_chain_id(),
        Some(channel_id_timeout),
    )
    .await
    .expect(&format!("Could not find {} channel", get_ibc_chain_id()));
    info!(
        "Found {} channel id: {}",
        get_ibc_chain_id(),
        ibc_channel_id
    );

    // // Test an IBC transfer of 1 stake from ibc-test-1 to gravity-test-1
    // let sender = ibc_keys[0];
    // let receiver = keys[0].validator_key.to_address(&ADDRESS_PREFIX).unwrap();

    // test_ibc_transfer(
    //     ibc_contact,
    //     gravity_bank_qc.clone(),
    //     gravity_transfer_qc.clone(),
    //     sender,
    //     receiver,
    //     None,
    //     None,
    //     ibc_channel_id.clone(),
    //     Duration::from_secs(60 * 5),
    // )
    // .await;

    //
    //
    //
    info!("\n\n!!!!!!!!!! Start IBC Auto-Send-Eth Happy Path Test !!!!!!!!!!\n\n");
    // send some amount from ethereum to the ibc chain
    info!("Sending token from ether to the IBC chain");
    let sender = keys[0].validator_key;
    let receiver = ibc_keys[0].to_address(&IBC_ADDRESS_PREFIX).unwrap();
    ibc_auto_forward::setup_gravity_auto_forwards(
        gravity_contact,
        (*IBC_ADDRESS_PREFIX).clone(),
        gravity_channel_id.clone(),
        sender,
        &keys,
    )
    .await;
    ibc_auto_forward::test_ibc_auto_forward_happy_path(
        web30,
        gravity_contact,
        gravity_client.clone(),
        ibc_bank_qc.clone(),
        ibc_transfer_qc.clone(),
        sender,
        receiver,
        gravity_address,
        erc20_address,
        one_eth(),
    )
    .await
    .expect("Failed send some amount from ethereum to the ibc chain");
    info!("Successful send some amount from ethereum to the ibc chain");
    // send some amount from ethereum to the ibc chain
    info!("Now send back the token from the IBC chain to ethereum by auto send to eth");

    // Test an IBC transfer of 1 stake from ibc-test-1 to gravity-test-1
    let sender = ibc_keys[0];
    let receiver = keys[0].eth_key.to_address();
    test_ibc_auto_send_eth_happy_path(
        web30,
        ibc_contact,
        gravity_contact,
        sender,
        receiver,
        erc20_address,
        one_eth(),
        ibc_transfer_qc.clone(),
        ibc_channel_id.clone(),
        Duration::from_secs(60 * 5),
    )
    .await
    .expect("Failed send back the token from the IBC chain to ethereum by auto send to eth");

    info!("Successful send back the token from the IBC chain to ethereum by auto send to eth");
}

// Sends 1 ibc-test-1 stake from `sender` to `receiver` on gravity-test-1 and asserts receipt of funds
#[allow(clippy::too_many_arguments)]
pub async fn test_ibc_transfer(
    contact: &Contact,                     // Src chain's deep_space client
    dst_bank_qc: BankQueryClient<Channel>, // Dst chain's GRPC x/bank query client
    dst_ibc_transfer_qc: IbcTransferQueryClient<Channel>, // Dst chain's GRPC ibc-transfer query client
    sender: impl PrivateKey,                              // The Src chain's funds sender
    receiver: CosmosAddress,                              // The Dst chain's funds receiver
    coin: Option<Coin>,                                   // The coin to send to receiver
    fee_coin: Option<DSCoin>, // The fee to pay for submitting the transfer msg
    channel_id: String,       // The Src chain's ibc channel connecting to Dst
    packet_timeout: Duration, // Used to create ibc-transfer timeout-timestamp
) -> bool {
    let sender_address = sender.to_address(&IBC_ADDRESS_PREFIX).unwrap().to_string();
    let pre_bal = get_ibc_balance(
        receiver,
        (*IBC_STAKING_TOKEN).to_string(),
        None,
        dst_bank_qc.clone(),
        dst_ibc_transfer_qc.clone(),
        None,
    )
    .await;

    let timeout_timestamp = SystemTime::now()
        .add(packet_timeout)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    info!("Calculated 150 minutes from now: {:?}", timeout_timestamp);
    let coin = coin.unwrap_or(Coin {
        denom: IBC_STAKING_TOKEN.to_string(),
        amount: one_atom().to_string(),
    });
    let msg_transfer = MsgTransfer {
        source_port: "transfer".to_string(),
        source_channel: channel_id,
        token: Some(coin.clone()),
        sender: sender_address,
        receiver: receiver.to_string(),
        timeout_height: None,
        timeout_timestamp, // 150 minutes from now
        ..Default::default()
    };
    info!("Submitting MsgTransfer {:?}", msg_transfer);
    let msg_transfer = Msg::new(MSG_TRANSFER_TYPE_URL, msg_transfer);
    let fee_coin = fee_coin.unwrap_or(DSCoin {
        amount: 100u16.into(),
        denom: (*IBC_STAKING_TOKEN).to_string(),
    });
    let send_res = contact
        .send_message(
            &[msg_transfer],
            Some("Test Relaying".to_string()),
            &[fee_coin],
            Some(OPERATION_TIMEOUT),
            sender,
        )
        .await;
    info!("Sent MsgTransfer with response {:?}", send_res);

    // Give the ibc-relayer a bit of time to work in the event of multiple runs
    // delay_for(Duration::from_secs(10)).await;
    wait_for_height(5, contact).await;

    let start_bal = Some(match pre_bal.clone() {
        Some(coin) => Uint256::from_str(&coin.amount).unwrap(),
        None => 0u8.into(),
    });

    let post_bal = get_ibc_balance(
        receiver,
        (*IBC_STAKING_TOKEN).to_string(),
        start_bal,
        dst_bank_qc,
        dst_ibc_transfer_qc,
        None,
    )
    .await;
    match (pre_bal, post_bal) {
        (None, None) => {
            error!(
                "Failed to transfer stake to gravity-test-1 user {}!",
                receiver
            );
            return false;
        }
        (None, Some(post)) => {
            if post.amount != coin.amount {
                error!(
                    "Incorrect ibc stake balance for user {}: actual {} != expected {}",
                    receiver, post.amount, coin.amount,
                );
                return false;
            }
            info!(
                "Successfully transfered {} stake (aka {}) to gravity-test-1!",
                coin.amount, post.denom
            );
        }
        (Some(pre), Some(post)) => {
            let amount_uint = Uint256::from_str(&coin.amount).unwrap();
            let pre_amt = Uint256::from_str(&pre.amount).unwrap();
            let post_amt = Uint256::from_str(&post.amount).unwrap();
            if post_amt < pre_amt || post_amt - pre_amt != amount_uint {
                error!(
                    "Incorrect ibc stake balance for user {}: actual {} != expected {}",
                    receiver,
                    post.amount,
                    (pre_amt + amount_uint),
                );
                return false;
            }
            info!(
                "Successfully transfered {} stake (aka {}) to gravity-test-1!",
                coin.amount, post.denom
            );
        }
        (Some(_), None) => {
            error!(
                "User wound up with no balance after ibc transfer? {}",
                receiver,
            );
            return false;
        }
    }
    true
}

// Initiates a SendToCosmos with a CosmosReceiver prefixed by "cosmos1", potentially clears a pending
// IBC Auto-Forward and asserts that the bridged ERC20 is received on ibc-test-1
#[allow(clippy::too_many_arguments)]
pub async fn test_ibc_auto_send_eth_happy_path(
    web30: &Web3,
    contact: &Contact,
    gravity_contact: &Contact,
    sender: CosmosPrivateKey,  // user who submits ibc transfer
    dest: EthAddress,          // The bridged + auto-forwarded ERC20 receiver
    erc20_address: EthAddress, // Address of the ERC20 to send to dest on ibc-test-1
    amount: Uint256,           // The amount of erc20_address token to send to dest on ibc-test-1
    ibc_transfer_qc: IbcTransferQueryClient<Channel>,
    channel_id: String,
    packet_timeout: Duration, // Used to create ibc-transfer timeout-timestamp
) -> Result<(), GravityError> {
    let mut ibc_transfer_qc = ibc_transfer_qc;
    let sender_address = sender.to_address(&IBC_ADDRESS_PREFIX).unwrap().to_string();
    // Make the test idempotent by getting the user's balance now
    let bridged_erc20 = EVM_CHAIN_PREFIX.to_string()
        + &GRAVITY_DENOM_SEPARATOR.to_string()
        + &erc20_address.clone().to_string();

    // get ibc denom of `bridged_erc20` in ibc-test-1 get hash from `transfer/<port>/<base denom>`
    let denom_hash_res = ibc_transfer_qc
        .denom_hash(IbcTransferV1::QueryDenomHashRequest {
            trace: format!("transfer/{}/{}", channel_id.clone(), bridged_erc20.clone()),
        })
        .await
        .unwrap();
    let denom_hash = denom_hash_res.into_inner().hash;

    // get pre send balance of dest
    let starting_balance = get_erc20_balance_safe(erc20_address, web30, dest)
        .await
        .unwrap();
    info!("Found pre-forward-balance of {:?}", starting_balance);
    // send ibc token to from ibc to gravity with memo for auto send eth
    let timeout_timestamp = SystemTime::now()
        .add(packet_timeout)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    info!("Calculated 150 minutes from now: {:?}", timeout_timestamp);
    let coin = Coin {
        denom: format!("ibc/{}", denom_hash),
        amount: amount.to_string(),
    };
    let forwarder_keys = get_user_key(Some("gravity"));
    info!("Forwarder {:?}", forwarder_keys.cosmos_address.to_string());
    let msg_transfer = MsgTransfer {
        source_port: "transfer".to_string(),
        source_channel: channel_id,
        token: Some(coin.clone()),
        sender: sender_address,
        receiver: forwarder_keys.cosmos_address.to_string(),
        timeout_height: None,
        timeout_timestamp,
        memo: format!(
            r#"{{"send_to_eth":{{"evm_chain_prefix":"{}","eth_dest":"{}","amount":"{}" }} }}"#,
            EVM_CHAIN_PREFIX.to_string(),
            dest,
            amount.to_string()
        ),
        ..Default::default()
    };
    info!("Submitting MsgTransfer {:?}", msg_transfer);
    let msg_transfer = Msg::new(MSG_TRANSFER_TYPE_URL, msg_transfer);
    let fee_coin = DSCoin {
        amount: 100u16.into(),
        denom: (*IBC_STAKING_TOKEN).to_string(),
    };
    let send_res = contact
        .send_message(
            &[msg_transfer],
            Some("Test Relaying".to_string()),
            &[fee_coin],
            Some(OPERATION_TIMEOUT),
            sender,
        )
        .await;
    info!("Sent MsgTransfer with response {:?}", send_res);
    info!("Locked up {} to send to Gravity to Eth", amount);

    // Give the ibc-relayer a bit of time to work in the event of multiple runs
    // delay_for(Duration::from_secs(10)).await;
    wait_for_height(5, contact).await;
    let start = Instant::now();
    while Instant::now() - start < TOTAL_TIMEOUT {
        let new_balance = get_erc20_balance_safe(erc20_address, web30, dest).await;
        // only keep trying if our error is gas related
        if new_balance.is_err() {
            continue;
        }
        let balance = new_balance.unwrap();
        if balance - starting_balance == amount {
            info!("Successfully bridged {} to Ethereum!", amount);
            assert!(balance == amount);

            let balances =
                get_balances_with_retry(forwarder_keys.cosmos_address, gravity_contact).await;
            for balance in balances {
                if balance.denom == bridged_erc20 {
                    if balance.amount != Uint256::from(0u64) {
                        panic!("Expected 0 but got {} instead", balance.amount);
                    }
                }
            }
            return Ok(());
        } else if balance - starting_balance != 0u8.into() {
            panic!("Expected {} but got {} instead", amount, balance);
        }
        sleep(Duration::from_secs(1)).await;
    }
    panic!("Timed out waiting for ethereum balance");
}
