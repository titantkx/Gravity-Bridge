//! This is a test for validator set relaying rewards

use crate::airdrop_proposal::wait_for_proposals_to_execute;
use crate::get_fee;
use crate::happy_path::test_valset_update;
use crate::happy_path_v2::deploy_cosmos_representing_erc20_and_check_adoption;
use crate::utils::{
    create_parameter_change_proposal, footoken_metadata, get_erc20_balance_safe,
    vote_yes_on_proposals, EvmChainParamForProposal, ValidatorKeys,
};
use clarity::Address as EthAddress;
use cosmos_gravity::query::get_gravity_params;
use deep_space::coin::Coin;
use deep_space::Contact;
use gravity_proto::cosmos_sdk_proto::cosmos::params::v1beta1::ParamChange;
use gravity_proto::gravity::query_client::QueryClient as GravityQueryClient;
use tonic::transport::Channel;
use web30::client::Web3;

pub async fn valset_rewards_test(
    web30: &Web3,
    grpc_client: GravityQueryClient<Channel>,
    evm_chain_prefix: &str,
    contact: &Contact,
    keys: Vec<ValidatorKeys>,
    gravity_address: EthAddress,
) {
    let mut grpc_client = grpc_client;
    let token_to_send_to_eth = footoken_metadata(contact).await.base;

    // first we deploy the Cosmos asset that we will use as a reward and make sure it is adopted
    // by the Cosmos chain
    let erc20_contract = deploy_cosmos_representing_erc20_and_check_adoption(
        gravity_address,
        web30,
        Some(keys.clone()),
        &mut grpc_client,
        false,
        footoken_metadata(contact).await,
    )
    .await;

    // reward of 1 mfootoken
    let valset_reward = Coin {
        denom: token_to_send_to_eth,
        amount: 1_000_000u64.into(),
    };

    let params = get_gravity_params(&mut grpc_client).await.unwrap();
    // let evm_chain_params = params
    //     .evm_chain_params
    //     .iter()
    //     .find(|p| p.evm_chain_prefix.eq(evm_chain_prefix))
    //     .unwrap();

    // clone `params.evm_chain_params` and update `BridgeEthereumAddress` and `BridgeChainID`
    let mut new_evm_chains_params: Vec<EvmChainParamForProposal> = params
        .evm_chain_params
        .clone()
        .iter()
        .map(|c| EvmChainParamForProposal::from_evm_chain_param(c.clone()))
        .collect();
    new_evm_chains_params.iter_mut().for_each(|p| {
        if p.evm_chain_prefix.eq(evm_chain_prefix) {
            p.bridge_ethereum_address = gravity_address.to_string();
            p.bridge_chain_id = "15".to_string();
        }
    });

    let new_evm_chains_params_json = serde_json::to_string(&new_evm_chains_params).unwrap();

    let mut params_to_change = Vec::new();

    let evm_chain_param = ParamChange {
        subspace: "gravity".to_string(),
        key: "EvmChainParams".to_string(),
        value: new_evm_chains_params_json.clone(),
    };
    params_to_change.push(evm_chain_param);
    let json_value = serde_json::to_string(&valset_reward).unwrap().to_string();
    let valset_reward_param = ParamChange {
        subspace: "gravity".to_string(),
        key: "ValsetReward".to_string(),
        value: json_value.clone(),
    };
    params_to_change.push(valset_reward_param);

    // next we create a governance proposal to use the newly bridged asset as the reward
    // and vote to pass the proposal
    info!("Creating parameter change governance proposal");
    create_parameter_change_proposal(
        contact,
        keys[0].validator_key,
        params_to_change,
        get_fee(None),
    )
    .await;

    vote_yes_on_proposals(contact, &keys, None).await;

    // wait for the voting period to pass
    wait_for_proposals_to_execute(contact).await;

    let params = get_gravity_params(&mut grpc_client).await.unwrap();
    let evm_chain_params = params
        .evm_chain_params
        .iter()
        .find(|p| p.evm_chain_prefix.eq(evm_chain_prefix))
        .unwrap();
    // check that params have changed
    assert_eq!(evm_chain_params.bridge_chain_id, 15);
    assert_eq!(
        evm_chain_params.bridge_ethereum_address,
        gravity_address.to_string()
    );

    // get old footoken balance of all validators and store into a map
    let mut old_balances = Vec::new();
    for key in keys.iter() {
        let target_address = key.eth_key.to_address();
        let balance_of_footoken = get_erc20_balance_safe(erc20_contract, web30, target_address)
            .await
            .unwrap();
        old_balances.push((target_address, balance_of_footoken));
    }

    // trigger a valset update
    test_valset_update(web30, contact, &mut grpc_client, &keys, gravity_address).await;

    // check that one of the relayers has footoken now
    let mut found = false;
    for key in keys.iter() {
        let target_address = key.eth_key.to_address();
        let balance_of_footoken = get_erc20_balance_safe(erc20_contract, web30, target_address)
            .await
            .unwrap();
        // get old balance
        let old_balance = old_balances
            .iter()
            .find(|x| x.0 == target_address)
            .unwrap()
            .1;

        if balance_of_footoken == old_balance + valset_reward.amount {
            found = true;
        }
    }
    if !found {
        panic!("No relayer was rewarded in footoken for relaying validator set!")
    }
    info!("Successfully Issued validator set reward!");
}
