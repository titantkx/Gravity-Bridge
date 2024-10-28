#!/bin/bash

set -eux

# your gaiad binary name
BIN=/titan/bin/titand

VALIDATOR_HOME="/root/.titand"
RPC_ADDRESS="--rpc.laddr tcp://0.0.0.0:27657"
GRPC_ADDRESS="--grpc.address 0.0.0.0:9190"
# Must remap the grpc-web address because it conflicts with what we want to use
GRPC_WEB_ADDRESS="--grpc-web.address 0.0.0.0:9192"

LOG_LEVEL="--log_level info"

ARG="--home $VALIDATOR_HOME $RPC_ADDRESS $GRPC_ADDRESS $GRPC_WEB_ADDRESS $LOG_LEVEL"

# shellcheck disable=SC2086
$BIN $ARG start
