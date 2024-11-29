#!/bin/bash

# Number of gravity validator nodes
export NODES=${1:-3}

set -eux

export KEEP_ORCHESTRATOR_TEST_RUNNING=true

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# happy path
"$DIR"/run-test.sh
"$DIR"/run-test.sh IBC_METADATA
"$DIR"/run-test.sh HAPPY_PATH_V2
"$DIR"/run-test.sh HAPPY_PATH_V2_NATIVE
# this test is disabled because it only can run once (hardcode tokenId)
# "$DIR"/run-test.sh ERC721_HAPPY_PATH
"$DIR"/run-test.sh IBC_AUTO_FORWARD
"$DIR"/run-test.sh IBC_AUTO_FORWARD_RETRY
"$DIR"/run-test.sh IBC_AUTO_FORWARD_TKX
"$DIR"/run-test.sh IBC_AUTO_SEND_ETH
"$DIR"/run-test.sh ETHEREUM_KEYS
"$DIR"/run-test.sh ORCHESTRATOR_KEYS
"$DIR"/run-test.sh BATCH_TIMEOUT
"$DIR"/run-test.sh SEND_TO_ETH_FEES
"$DIR"/run-test.sh ETHEREUM_BLACKLIST
"$DIR"/run-test.sh VALSET_REWARDS
"$DIR"/run-test.sh TXCANCEL
"$DIR"/run-test.sh VALSET_STRESS
"$DIR"/run-test.sh AIRDROP_PROPOSAL
"$DIR"/run-test.sh BATCH_STRESS

echo "Stopping container gravity-with-titan-orchestrator-test"
docker stop gravity-with-titan-orchestrator-test

echo "Stopping chain container"
docker compose -f "$DIR"/docker-compose.yml stop orchestrator evm gravity titan
