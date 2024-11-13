use std::str::FromStr;
use std::time::{Duration, Instant};

use clarity::Address as EthAddress;
use cosmos_gravity::send::MSG_EXECUTE_IBC_AUTO_FORWARDS_TYPE_URL;
use deep_space::address::Address as CosmosAddress;
use deep_space::private_key::{CosmosPrivateKey, PrivateKey};
use deep_space::{Coin, Contact, Msg};
use gravity_proto::cosmos_sdk_proto::cosmos::bank::v1beta1::query_client::QueryClient as BankQueryClient;
use gravity_proto::cosmos_sdk_proto::ibc::applications::transfer::v1::query_client::QueryClient as IbcTransferQueryClient;
use gravity_proto::cosmos_sdk_proto::ibc::core::channel::v1::query_client::QueryClient as IbcChannelQueryClient;
use gravity_proto::gravity::query_client::QueryClient as GravityQueryClient;
use gravity_proto::gravity::{
    MsgExecuteIbcAutoForwards, QuerySendingIbcAutoForwards, SendingIbcAutoForward,
};
use gravity_utils::error::GravityError;
use num256::Uint256;
use tokio::time::sleep;
use tonic::transport::Channel;
use web30::client::Web3;

use crate::happy_path::send_erc20_deposit;
use crate::ibc_auto_forward::{get_ibc_balance, wait_for_pending_ibc_auto_forwards};
use crate::{
    create_default_test_config, get_ibc_chain_id, get_user_key,
    ibc_auto_forward::{get_channel_id, setup_gravity_auto_forwards},
    prepare_ibc_relayer, start_orchestrators,
    types::IBCPrivateKey,
    ValidatorKeys, COSMOS_NODE_GRPC, IBC_ADDRESS_PREFIX, IBC_NODE_GRPC,
};
use crate::{
    get_event_nonce_safe, start_ibc_relayer_process, stop_ibc_relayer_process, ADDRESS_PREFIX,
    EVM_CHAIN_PREFIX, GRAVITY_DENOM_SEPARATOR, MINER_PRIVATE_KEY, OPERATION_TIMEOUT, STAKING_TOKEN,
};

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
    prepare_ibc_relayer(contact, ibc_contact, &keys, &ibc_keys).await;

    let no_relay_market_config = create_default_test_config();
    start_orchestrators(keys.clone(), gravity_address, false, no_relay_market_config).await;

    let ibc_user_keys = get_user_key(Some(&IBC_ADDRESS_PREFIX));

    let gravity_channel_qc = IbcChannelQueryClient::connect(COSMOS_NODE_GRPC.as_str())
        .await
        .expect("Could not connect channel query client");
    let ibc_bank_qc = BankQueryClient::connect(IBC_NODE_GRPC.as_str())
        .await
        .expect("Could not connect bank query client");
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

    info!("Found gravity-test-1 channel id: {}", gravity_channel_id);

    info!("\n\n!!!!!!!!!  Send token from ether to titan chain but not run IBC relayer  !!!!!!!!!!! \n\n");
    info!("Setup auto forward: create gov proposal to map chain prefix to ibc source channel id");
    setup_gravity_auto_forwards(
        contact,
        (*IBC_ADDRESS_PREFIX).clone(),
        gravity_channel_id.clone(),
        keys[0].validator_key,
        &keys,
    )
    .await;

    info!("Test ibc auto forward sending stage");
    let receiver = ibc_keys[0].to_address(&IBC_ADDRESS_PREFIX).unwrap();
    test_ibc_auto_forward_sending(
        web30,
        contact,
        gravity_client,
        ibc_bank_qc,
        ibc_transfer_qc,
        keys[0].validator_key,
        receiver,
        gravity_address,
        erc20_address,
        100u8.into(),
    )
    .await
    .expect("Failed to test ibc auto forward sending");
}

pub async fn test_ibc_auto_forward_sending(
    web30: &Web3,
    contact: &Contact,
    gravity_client: GravityQueryClient<Channel>, // Src chain's Gravity GRPC client
    dst_bank_qc: BankQueryClient<Channel>,       // Dst chain's Bank GRPC client
    dst_ibc_transfer_qc: IbcTransferQueryClient<Channel>, // Dst chain's ibc-transfer GRPC client
    forwarder: CosmosPrivateKey, // user who submits MsgExecutePendingIbcAutoForwards
    dest: CosmosAddress,         // The bridged + auto-forwarded ERC20 receiver
    gravity_address: EthAddress, // Address of the gravity contract
    erc20_address: EthAddress,   // Address of the ERC20 to send to dest on IBC_CHAIN_ID
    amount: Uint256, // The amount of erc20_address token to send to dest on IBC_CHAIN_ID
) -> Result<(), GravityError> {
    let bridged_erc20_denom = EVM_CHAIN_PREFIX.to_string()
        + &GRAVITY_DENOM_SEPARATOR.to_string()
        + &erc20_address.clone().to_string();

    let pre_forward_balance = get_ibc_balance(
        dest,
        bridged_erc20_denom.clone(),
        None,
        dst_bank_qc.clone(),
        dst_ibc_transfer_qc.clone(),
        Some(Duration::from_secs(5)),
    )
    .await;
    info!("Found pre-forward-balance of {:?}", pre_forward_balance);

    stop_ibc_relayer_process();

    // get current event nonce of gravity contract
    let current_event_nonce =
        get_event_nonce_safe(gravity_address, web30, MINER_PRIVATE_KEY.to_address())
            .await
            .unwrap();

    // First Send to Titan
    send_erc20_deposit(
        web30,
        &mut gravity_client.clone(),
        dest,
        gravity_address,
        erc20_address,
        amount,
        r#"{"This" is memo string : value}"#,
    )
    .await
    .expect("Failed to send erc20 deposit");

    // Check for a Pending IBC Auto-Forward (which may have already been cleared by the running relayer)
    let pending =
        wait_for_pending_ibc_auto_forwards(gravity_client.clone(), None, Some(OPERATION_TIMEOUT))
            .await;

    // Attempt to clear the Pending forward
    if !(pending.is_err() || pending.unwrap().is_empty()) {
        info!("Discovered pending IBC Auto Forward(s) that the relayer hasn't picked up, clearing it!");
        let msg_execute_forwards = Msg::new(
            MSG_EXECUTE_IBC_AUTO_FORWARDS_TYPE_URL,
            MsgExecuteIbcAutoForwards {
                evm_chain_prefix: EVM_CHAIN_PREFIX.to_string(),
                forwards_to_clear: 1,
                executor: forwarder.to_address(&ADDRESS_PREFIX).unwrap().to_string(),
            },
        );
        let _res = contact
            .send_message(
                &[msg_execute_forwards],
                None,
                &[Coin {
                    denom: (*STAKING_TOKEN).clone(),
                    amount: 0u8.into(),
                }],
                Some(OPERATION_TIMEOUT),
                forwarder,
            )
            .await;
    }

    let sending = wait_for_sending_ibc_auto_forwards(
        gravity_client.clone(),
        current_event_nonce + 1,
        Some(OPERATION_TIMEOUT),
    )
    .await
    .unwrap();

    info!("Sending IBC Auto Forward: {:?}", sending);

    start_ibc_relayer_process();

    // wait for the ibc relayer to relay the IBC packet
    wait_for_sending_ibc_auto_forwards_empty(gravity_client.clone(), Some(OPERATION_TIMEOUT))
        .await
        .expect("Failed to wait for sending ibc auto forwards to be empty");

    // Check the Foreign Receiver's balance has increased by the appropriate amount
    let start_bal = match pre_forward_balance.clone() {
        Some(coin) => Some(Uint256::from_str(&coin.amount).unwrap()),
        None => None,
    };
    let post_forward_balance = get_ibc_balance(
        dest,
        bridged_erc20_denom.clone(),
        start_bal,
        dst_bank_qc.clone(),
        dst_ibc_transfer_qc.clone(),
        Some(Duration::from_secs(60)),
    )
    .await;
    // Potential race condition: Slow gravity relayers and/or ibc relayer
    info!("Found a post-forward-balance of {:?}", post_forward_balance);

    match (pre_forward_balance, post_forward_balance) {
        (None, None) => {
            panic!("Never found an ibc auto-forward balance for user {}", dest);
        }
        (None, Some(post)) => {
            if Uint256::from_str(&post.amount).unwrap() != amount {
                panic!(
                    "Incorrect ibc auto-forward balance for user {}: actual {} != expected {}",
                    dest, post.amount, amount,
                );
            }
            info!(
                "Successful IBC auto-forward of amount {} to {}",
                amount, dest,
            );
            Ok(())
        }
        (Some(pre), Some(post)) => {
            let pre_amt = Uint256::from_str(&pre.amount).unwrap();
            let post_amt = Uint256::from_str(&post.amount).unwrap();

            if post_amt < pre_amt || (pre_amt + amount) != post_amt {
                info!("post_amt < pre_amt: {}", post_amt < pre_amt);
                info!(
                    "(pre_amt + amount) != post_amt: {}",
                    (pre_amt + amount) != post_amt
                );
                panic!(
                    "Incorrect ibc auto-forward balance for user {}: actual {} != expected {}",
                    dest,
                    post.amount,
                    (pre_amt + amount)
                );
            }
            Ok(())
        }
        (Some(_), None) => {
            panic!(
                "User wound up with no balance after ibc auto-forward? {}",
                dest,
            );
        }
    }
}

// Waits for Sending IBC Auto Forwards to enter the queue by repeatedly querying via GRPC
// Returns a TimeoutError error if the GRPC endpoint never contains at least `expected`
// Optionally takes an `expected` number of pending forwards to wait for
// Optionally takes a timeout, otherwise waits for OPERATION_TIMEOUT seconds
pub async fn wait_for_sending_ibc_auto_forwards(
    gravity_client: GravityQueryClient<Channel>,
    expected_event_nonce: u64,
    timeout: Option<Duration>,
) -> Result<SendingIbcAutoForward, GravityError> {
    let mut gravity_client = gravity_client;
    let timeout = match timeout {
        Some(t) => t,
        None => OPERATION_TIMEOUT,
    };

    let start = Instant::now();
    while Instant::now() - start < timeout {
        let res = gravity_client
            .get_sending_ibc_auto_forwards(QuerySendingIbcAutoForwards { limit: 0 })
            .await?
            .into_inner();
        if res.sending_ibc_auto_forwards.is_empty() {
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        // scan in res have expected event nonce
        for forward in res.sending_ibc_auto_forwards.iter() {
            if let Some(ibc_packet) = forward.ibc_packet.as_ref() {
                if ibc_packet.event_nonce == expected_event_nonce {
                    return Ok(forward.clone());
                }
            }
        }
    }
    Err(GravityError::TimeoutError)
}

pub async fn wait_for_sending_ibc_auto_forwards_empty(
    gravity_client: GravityQueryClient<Channel>,
    timeout: Option<Duration>,
) -> Result<(), GravityError> {
    let mut gravity_client = gravity_client;
    let timeout = match timeout {
        Some(t) => t,
        None => OPERATION_TIMEOUT,
    };

    let start = Instant::now();

    while Instant::now() - start < timeout {
        let res = gravity_client
            .get_sending_ibc_auto_forwards(QuerySendingIbcAutoForwards { limit: 0 })
            .await?
            .into_inner();
        if res.sending_ibc_auto_forwards.is_empty() {
            sleep(Duration::from_secs(5)).await;
            return Ok(());
        }
    }
    Err(GravityError::TimeoutError)
}
