#!/bin/bash

set -eux

NODES=$1

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

CHAIN_ID="gravity-test-1"
DENOM="ugraviton"
ALLOCATION="1000000000000000footoken,1000000000000000footoken2,1000000000000000ibc/nometadatatoken,1000000000000000$DENOM"

# first we start a genesis.json with validator 1
# validator 1 will also collect the gentx's once gnerated
STARTING_VALIDATOR=1
STARTING_VALIDATOR_HOME="/validator$STARTING_VALIDATOR"

### init chain
$BIN init --home "$STARTING_VALIDATOR_HOME" --chain-id $CHAIN_ID validator1

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

jq "$config" "$STARTING_VALIDATOR_HOME"/config/genesis.json >/edited-genesis.json

# Change the stake token to be ugraviton instead
$SED_INPLACE 's/\<stake\>/'"$DENOM"'/g' /edited-genesis.json

mv /edited-genesis.json /genesis.json

VESTING_AMOUNT="1000000000$DENOM"
START_VESTING=$(expr $(date +%s) + 300)  # Start vesting 15 minutes from now
END_VESTING=$(expr $START_VESTING + 120) # End vesting 20 minutes from now, giving a 5 minute window for the test to work

### config validator
for i in $(seq 1 $NODES); do
  # move the genesis in
  mkdir -p /validator$i/config/
  mv /genesis.json /validator$i/config/genesis.json
  VALIDATOR_HOME="--home /validator$i"
  ARGS="$VALIDATOR_HOME --keyring-backend test"

  $BIN keys add $ARGS validator$i 2>>$SHARED_FOLDER/validator-phrases
  $BIN keys add $ARGS orchestrator$i 2>>$SHARED_FOLDER/orchestrator-phrases
  $BIN eth_keys add >>$SHARED_FOLDER/validator-eth-keys
  $BIN keys add $ARGS vesting$i 2>>$SHARED_FOLDER/vesting-phrases

  VALIDATOR_KEY=$($BIN keys show validator$i -a $ARGS)
  echo "validator$i: $VALIDATOR_KEY" 1>>$SHARED_FOLDER/validator-addresses
  ORCHESTRATOR_KEY=$($BIN keys show orchestrator$i -a $ARGS)
  echo "orchestrator$i: $ORCHESTRATOR_KEY" 1>>$SHARED_FOLDER/orchestrator-addresses
  VESTING_KEY=$($BIN keys show vesting$i -a $ARGS)
  echo "vesting$i: $VESTING_KEY" 1>>$SHARED_FOLDER/vesting-addresses

  $BIN add-genesis-account $ARGS $VALIDATOR_KEY $ALLOCATION
  $BIN add-genesis-account $ARGS $ORCHESTRATOR_KEY $ALLOCATION
  # Add a vesting account
  $BIN add-genesis-account $ARGS $VESTING_KEY --vesting-amount $VESTING_AMOUNT --vesting-start-time $START_VESTING --vesting-end-time $END_VESTING $VESTING_AMOUNT
  # move the genesis back out
  mv /validator$i/config/genesis.json /genesis.json
done

for i in $(seq 1 $NODES); do
  cp /genesis.json /validator$i/config/genesis.json
  VALIDATOR_HOME="--home /validator$i"
  ARGS="$VALIDATOR_HOME --keyring-backend test"

  ORCHESTRATOR_KEY=$($BIN keys show orchestrator$i -a $ARGS)
  ETHEREUM_KEY=$(grep address $SHARED_FOLDER/validator-eth-keys | sed -n "$i"p | sed 's/.*://')
  echo "validator$i: $ETHEREUM_KEY" 1>>$SHARED_FOLDER/validator-eth-addresses

  $BIN gentx $ARGS --moniker=validator$i --chain-id=$CHAIN_ID --ip 7.7.7.$i validator$i 500000000$DENOM $ETHEREUM_KEY $ORCHESTRATOR_KEY
  if [ $i -gt 1 ]; then
    cp /validator$i/config/gentx/* /validator1/config/gentx/
  fi
done

$BIN collect-gentxs --home $STARTING_VALIDATOR_HOME

cp /validator1/config/genesis.json /genesis.json
cp /genesis.json $SHARED_FOLDER/gravity-genesis.json

# put the now final genesis.json into the correct folders
for i in $(seq 1 $NODES); do
  cp /genesis.json /validator$i/config/genesis.json
  $SED_INPLACE 's/^timeout_commit = ".*"/timeout_commit = "2.5s"/' /validator$i/config/config.toml

  if [[ "$i" -eq 1 ]]; then
    ### config node
    $SED_INPLACE 's/^indexer = ".*"/indexer = "kv"/' "$STARTING_VALIDATOR_HOME"/config/config.toml

    $SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(enable = \).*/\1true/' "$STARTING_VALIDATOR_HOME"/config/app.toml
    $SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(swagger = \).*/\1true/' "$STARTING_VALIDATOR_HOME"/config/app.toml
    $SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(address = \).*/\1\"tcp:\/\/0.0.0.0:1317\"/' "$STARTING_VALIDATOR_HOME"/config/app.toml
  fi
done
