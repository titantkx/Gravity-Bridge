#!/bin/bash

set -eux

BIN=/titan/bin/titand
SHARED_FOLDER="/shared_tmp"

mkdir -p $SHARED_FOLDER

VALIDATOR_HOME="/root/.titand"
DENOM="atkx"

# check /cosmwasm.wasm exists
if [ ! -f /cosmwasm.wasm ]; then
  echo "Missing /cosmwasm.wasm"
  exit 1
fi

COMMON_QUERY_ARGS="--home $VALIDATOR_HOME --output json"
COMMON_TX_ARGS="--home $VALIDATOR_HOME --from val-1 --keyring-backend test --gas-prices 100000000000$DENOM --gas auto --gas-adjustment 1.3 -y --output json"

# store the code on chain
# shellcheck disable=SC2086
RES=$($BIN tx wasm store /cosmwasm.wasm $COMMON_TX_ARGS)
TX_HASH=$(echo "$RES" | jq -r '.txhash')
sleep 5
# shellcheck disable=SC2086
RES=$($BIN $COMMON_QUERY_ARGS q tx --type=hash $TX_HASH)

# get CODE_ID
CODE_ID=$(echo "$RES" | jq -r '.logs[0].events[-1].attributes[1].value')
echo "CODE_ID: $CODE_ID"

# instantiate the contract
# shellcheck disable=SC2086
$BIN tx wasm instantiate $CODE_ID "{}" --label "tkx exchange" $COMMON_TX_ARGS --no-admin
sleep 5
# shellcheck disable=SC2086
CONTRACT_ADDR=$($BIN $COMMON_QUERY_ARGS query wasm list-contract-by-code "$CODE_ID" | jq -r '.contracts[0]')

echo "TKX exchange contract address: $CONTRACT_ADDR"
echo "tkx-exchange: $CONTRACT_ADDR" 1>>$SHARED_FOLDER/tkx-exchange-contract-address
