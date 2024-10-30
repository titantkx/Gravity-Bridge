#!/bin/bash

set -eu

source /set-env.sh

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

ENV_SET='DEPLOY_CONTRACTS=1'
# TEST_TYPE=$TEST_TYPE

# shellcheck disable=SC2090
# shellcheck disable=SC2086
env $ENV_SET PATH="$PATH":"$HOME"/.cargo/bin cargo run --release --bin test-runner
