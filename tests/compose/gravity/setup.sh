#!/bin/bash

set -eux

# Detect platform
platform=$(uname)
if [ "$platform" = "Darwin" ]; then
  SED_INPLACE="sed -i ''"
else
  SED_INPLACE="sed -i"
fi

# your gaiad binary name
BIN=gravity
SHARED_FOLDER="/shared_tmp"

mkdir -p $SHARED_FOLDER

VALIDATOR_HOME="/root/.gravity"
CHAIN_ID="gravity-test-1"
DENOM="ugraviton"
ALLOCATION="1000000000000000footoken,1000000000000000footoken2,1000000000000000ibc/nometadatatoken,1000000000000000$DENOM"

### init chain
$BIN init --home $VALIDATOR_HOME --chain-id $CHAIN_ID validator1

### config node
$SED_INPLACE 's/^indexer = ".*"/indexer = "kv"/' $VALIDATOR_HOME/config/config.toml
$SED_INPLACE 's/^timeout_commit = ".*"/timeout_commit = "0.5s"/' $VALIDATOR_HOME/config/config.toml

$SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(enable = \).*/\1true/' $VALIDATOR_HOME/config/app.toml
$SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(swagger = \).*/\1true/' $VALIDATOR_HOME/config/app.toml
$SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(address = \).*/\1\"tcp:\/\/0.0.0.0:1317\"/' $VALIDATOR_HOME/config/app.toml

### config genesis

config='
.app_state.bank.denom_metadata += [{"name": "Foo Token", "symbol": "FOO", "base": "footoken", display: "mfootoken", "description": "A non-staking test token", "denom_units": [{"denom": "footoken", "exponent": 0}, {"denom": "mfootoken", "exponent": 6}]}] |
.app_state.bank.denom_metadata += [{"name": "Foo Token2", "symbol": "F20", "base": "footoken2", display: "mfootoken2", "description": "A second non-staking test token", "denom_units": [{"denom": "footoken2", "exponent": 0}, {"denom": "mfootoken2", "exponent": 6}]}] |
.app_state.bank.denom_metadata += [{"name": "Stake Token", "symbol": "GRAV", "base": "ugraviton", display: "ugraviton", "description": "A staking test token", "denom_units": [{"denom": "ugraviton", "exponent": 0}, {"denom": "graviton", "exponent": 6}]}] |
.app_state.bech32ibc.nativeHRP = "gravity" |
.app_state.gov.voting_params.voting_period = "120s" |
.app_state.auth.accounts += [{"@type":"/cosmos.auth.v1beta1.BaseAccount","account_number":"13","address":"gravity1hanqss6jsq66tfyjz56wz44z0ejtyv0724h32c","pub_key":null,"sequence":"0"}] |
.app_state.bank.balances += [{"address": "gravity1hanqss6jsq66tfyjz56wz44z0ejtyv0724h32c", "coins": [{"amount": "1000000000", "denom": "ugraviton"}]}] |
.app_state.distribution.fee_pool.community_pool = [{"denom": "footoken", "amount": "1000000000000000000000000"},{"denom": "ugraviton", "amount": "1000000000000000000000000"}] |
.app_state.auth.accounts += [{"@type": "/cosmos.auth.v1beta1.ModuleAccount", "base_account": { "account_number": "0", "address": "gravity1jv65s3grqf6v6jl3dp4t6c9t9rk99cd8r0kyvh","pub_key": null,"sequence": "0"},"name": "distribution","permissions": ["basic"]}] |
.app_state.bank.balances += [{"address": "gravity1jv65s3grqf6v6jl3dp4t6c9t9rk99cd8r0kyvh", "coins": [{"denom": "footoken", "amount": "1000000000000000000000000"},{"amount": "1000000000000000000000000", "denom": "ugraviton"}]}] |
.app_state.gravity.evm_chains = [{"evm_chain": {"evm_chain_prefix": "ethereum","evm_chain_name": "ethereum"},"gravity_nonces": {"latest_valset_nonce": "0","last_observed_nonce": "0","last_slashed_valset_nonce": "0","last_slashed_batch_block": "0","last_slashed_logic_call_block": "0","last_tx_pool_id": "0","last_batch_id": "0"},"valsets": [],"valset_confirms": [],"batches": [],"batch_confirms": [],"logic_calls": [],"logic_call_confirms": [],"attestations": [],"delegate_keys": [],"erc20_to_denoms": [],"unbatched_transfers": []}] |
.app_state.gravity.params.evm_chain_params = [{"evm_chain_prefix":"ethereum","average_ethereum_block_time":"15000","bridge_active":true,"bridge_chain_id":"15","bridge_ethereum_address":"0x0000000000000000000000000000000000000000","contract_source_hash":"","ethereum_blacklist":[],"gravity_id":"ethereum"}]
'

jq "$config" $VALIDATOR_HOME/config/genesis.json >/edited-genesis.json

# Change the stake token to be ugraviton instead
$SED_INPLACE 's/\<stake\>/'"$DENOM"'/g' /edited-genesis.json

mv /edited-genesis.json $VALIDATOR_HOME/config/genesis.json

VESTING_AMOUNT="1000000000$DENOM"
START_VESTING=$(expr $(date +%s) + 300)  # Start vesting 15 minutes from now
END_VESTING=$(expr $START_VESTING + 120) # End vesting 20 minutes from now, giving a 5 minute window for the test to work

### config validator

ARGS="--home $VALIDATOR_HOME --keyring-backend test"
i=1

$BIN keys add $ARGS validator$i 2>>$SHARED_FOLDER/validator-phrases
$BIN keys add $ARGS orchestrator$i 2>>$SHARED_FOLDER/orchestrator-phrases
$BIN keys add $ARGS vesting$i 2>>$SHARED_FOLDER/vesting-phrases
$BIN eth_keys add >>$SHARED_FOLDER/validator-eth-keys

VALIDATOR_KEY=$($BIN keys show validator$i -a $ARGS)
ORCHESTRATOR_KEY=$($BIN keys show orchestrator$i -a $ARGS)
ETHEREUM_KEY=$(grep address $SHARED_FOLDER/validator-eth-keys | sed -n "$i"p | sed 's/.*://')
VESTING_KEY=$($BIN keys show vesting$i -a $ARGS)

$BIN add-genesis-account $ARGS $VALIDATOR_KEY $ALLOCATION
$BIN add-genesis-account $ARGS $ORCHESTRATOR_KEY $ALLOCATION
# Add a vesting account
$BIN add-genesis-account $ARGS $VESTING_KEY --vesting-amount $VESTING_AMOUNT --vesting-start-time $START_VESTING --vesting-end-time $END_VESTING $VESTING_AMOUNT

$BIN gentx $ARGS --moniker=validator$i --chain-id=$CHAIN_ID validator$i 500000000$DENOM $ETHEREUM_KEY $ORCHESTRATOR_KEY

$BIN collect-gentxs --home $VALIDATOR_HOME
