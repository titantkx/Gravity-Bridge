#!/bin/bash

set -eux

# Setup relayer files to avoid permissions issues later
set +e
mkdir /ibc-relayer-logs
touch /ibc-relayer-logs/hermes-logs
touch /ibc-relayer-logs/channel-creation
set -e

# move data in `shared` folder to correct location
cp -rf shared/validator-phrases /validator-phrases
cp -rf shared/ibc-validator-phrases /ibc-validator-phrases
cp -rf shared/orchestrator-phrases /orchestrator-phrases
cp -rf shared/vesting-phrases /vesting-phrases
cp -rf shared/validator-eth-keys /validator-eth-keys

# deploy the ethereum contracts
pushd /gravity/orchestrator/test_runner
# shellcheck disable=SC2089
ENV_SET='DEPLOY_CONTRACTS=1 RUST_BACKTRACE=full NO_GAS_OPT=1 RUST_LOG=INFO,relayer=DEBUG,orchestrator=DEBUG'
# TEST_TYPE=$TEST_TYPE
ENV_SET="$ENV_SET COSMOS_NODE_GRPC=http://gravity:9090"
ENV_SET="$ENV_SET COSMOS_NODE_ABCI=http://gravity:26657"
ENV_SET="$ENV_SET IBC_NODE_GRPC=http://titan:9190"
ENV_SET="$ENV_SET IBC_NODE_ABCI=http://titan:27657"
ENV_SET="$ENV_SET ETH_NODE=http://evm:8545"

# shellcheck disable=SC2090
# shellcheck disable=SC2086
env $ENV_SET PATH="$PATH":"$HOME"/.cargo/bin cargo run --release --bin test-runner
popd
