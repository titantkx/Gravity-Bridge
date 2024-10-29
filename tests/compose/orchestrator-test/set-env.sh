#!/bin/bash

export COSMOS_NODE_GRPC=http://gravity:9090
export COSMOS_NODE_ABCI=http://gravity:26657
export IBC_NODE_GRPC=http://titan:9190
export IBC_NODE_ABCI=http://titan:27657
export ETH_NODE=http://evm:8545

export RUST_BACKTRACE=full
export NO_GAS_OPT=1
export RUST_LOG=INFO,relayer=DEBUG,orchestrator=DEBUG
