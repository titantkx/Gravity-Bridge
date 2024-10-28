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
BIN=/titan/bin/titand

VALIDATOR_HOME="/root/.titand"
CHAIN_ID="titan_18887-1"
DENOM="atkx"
ALLOCATION="10000000000000000000000000000$DENOM"

### init chain
$BIN init --home $VALIDATOR_HOME --chain-id $CHAIN_ID val-1 --default-denom $DENOM

### config node
$SED_INPLACE 's/^indexer = ".*"/indexer = "kv"/' $VALIDATOR_HOME/config/config.toml
$SED_INPLACE 's/^timeout_commit = ".*"/timeout_commit = "0.5s"/' $VALIDATOR_HOME/config/config.toml

$SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(enable = \).*/\1true/' $VALIDATOR_HOME/config/app.toml
$SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(swagger = \).*/\1true/' $VALIDATOR_HOME/config/app.toml
$SED_INPLACE '/^\[api\]$/,/^\[/ s/^\(address = \).*/\1\"tcp:\/\/0.0.0.0:1417\"/' $VALIDATOR_HOME/config/app.toml

### config genesis

# config denom
jq '.app_state.evm.params.evm_denom = "atkx"' $VALIDATOR_HOME/config/genesis.json >/denom-edited-genesis.json

# a 60 second voting period to allow us to pass governance proposals in the tests
jq '.app_state.gov.voting_params.voting_period = "60s"' /denom-edited-genesis.json >/gov-edited-genesis.json

mv /gov-edited-genesis.json $VALIDATOR_HOME/config/genesis.json

### config validator
$BIN keys add --home $VALIDATOR_HOME --keyring-backend test val-1 2>>/validator-phrases

VALIDATOR_KEY=$($BIN keys show val-1 -a --home $VALIDATOR_HOME --keyring-backend test)

$BIN add-genesis-account --home $VALIDATOR_HOME $VALIDATOR_KEY $ALLOCATION

$BIN gentx --home $VALIDATOR_HOME --keyring-backend test val-1 2048tkx

$BIN collect-gentxs --home $VALIDATOR_HOME
