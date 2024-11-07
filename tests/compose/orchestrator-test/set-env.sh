#!/bin/bash

export HERMES_CONFIG=/gravity/tests/assets/ibc-relayer-titan-compose-config.toml

export COSMOS_NODE_GRPC=http://gravity:9090
export COSMOS_NODE_ABCI=http://gravity:26657
export IBC_NODE_GRPC=http://titan:9090
export IBC_NODE_ABCI=http://titan:26657
export ETH_NODE=http://evm:8545
export IBC_CHAIN_ID=titan_18887-1
export IBC_ADDRESS_PREFIX=titan
export IBC_ADDRESS_TYPE=ethermint
export IBC_STAKING_TOKEN=atkx
export IBC_GAS_PRICE=1000000000
export IBC_STAKING_DECIMALS=18

export RUST_BACKTRACE=full
export NO_GAS_OPT=1
export RUST_LOG=INFO,relayer=INFO,orchestrator=DEBUG,cosmos_gravity=DEBUG,deep_space=TRACE
