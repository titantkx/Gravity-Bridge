#!/bin/bash

set -eux

NODES=$1

# copy from `/shared_tmp` to `/shared`
cp -r /shared_tmp/* /shared

# your gaiad binary name
BIN=gravity

STARTING_VALIDATOR=1

for i in $(seq 1 $NODES); do
  # add this ip for loopback dialing
  ip addr add 7.7.7.$i/32 dev eth0 || true # allowed to fail
  VALIDATOR_HOME="--home /validator$i"

  if [ $i -gt $STARTING_VALIDATOR ]; then
    RPC_ADDRESS="--rpc.laddr tcp://7.7.7.$i:26658"
    GRPC_ADDRESS="--grpc.address 7.7.7.$i:9091"
    GRPC_WEB_ADDRESS="--grpc-web.address 7.7.7.$i:9093"

    LISTEN_ADDRESS="--address tcp://7.7.7.$i:26655"
    P2P_ADDRESS="--p2p.laddr tcp://7.7.7.$i:26656"

    ARGS="$VALIDATOR_HOME $LISTEN_ADDRESS $P2P_ADDRESS $RPC_ADDRESS $GRPC_ADDRESS $GRPC_WEB_ADDRESS"

    # shellcheck disable=SC2086
    $BIN $ARGS start &>/validator$i/logs &
  fi

done

STARTING_VALIDATOR_HOME="/validator$STARTING_VALIDATOR"
RPC_ADDRESS="--rpc.laddr tcp://0.0.0.0:26657"
GRPC_ADDRESS="--grpc.address 0.0.0.0:9090"
GRPC_WEB_ADDRESS="--grpc-web.address 0.0.0.0:9092"

LISTEN_ADDRESS="--address tcp://7.7.7.$STARTING_VALIDATOR:26655"
P2P_ADDRESS="--p2p.laddr tcp://7.7.7.$STARTING_VALIDATOR:26656"

LOG_LEVEL="--log_level info"
INVARIANTS_CHECK="--inv-check-period 1"

ARGS="--home $STARTING_VALIDATOR_HOME $LISTEN_ADDRESS $P2P_ADDRESS $RPC_ADDRESS $GRPC_ADDRESS $GRPC_WEB_ADDRESS $LOG_LEVEL $INVARIANTS_CHECK"

# shellcheck disable=SC2086
$BIN $ARGS start
