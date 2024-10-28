#!/bin/bash

set -eux

# copy from `/shared_tmp` to `/shared`
cp -r /shared_tmp/* /shared

# your gaiad binary name
BIN=gravity

VALIDATOR_HOME="/root/.gravity"
RPC_ADDRESS="--rpc.laddr tcp://0.0.0.0:26657"
GRPC_ADDRESS="--grpc.address 0.0.0.0:9090"
GRPC_WEB_ADDRESS="--grpc-web.address 0.0.0.0:9092"

LOG_LEVEL="--log_level info"
INVARIANTS_CHECK="--inv-check-period 1"

ARG="--home $VALIDATOR_HOME $RPC_ADDRESS $GRPC_ADDRESS $GRPC_WEB_ADDRESS $LOG_LEVEL $INVARIANTS_CHECK"

# shellcheck disable=SC2086
$BIN $ARG start
