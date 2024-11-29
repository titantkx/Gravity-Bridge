#!/bin/bash

# Number of gravity validator nodes
export NODES=${1:-3}

set -eux

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$DIR/../.."

# remove evm, gravity and titan
docker compose -f $DIR/docker-compose.yml down

# check wasm contract exists
if [ ! -f $REPO_DIR/cosmwasm/artifacts/cosmwasm.wasm ]; then
  echo "Missing $REPO_DIR/cosmwasm/artifacts/cosmwasm.wasm"
  exit 1
fi

$DIR/build-orchestrator-test.sh

# Remove existing container instance
set +e
docker rm -f gravity-with-titan-orchestrator-test
set -e

# setup
# setup for Mac apple silicon compatibility
PLATFORM_CMD=""
if [[ "$OSTYPE" == "darwin"* ]]; then
  if [[ -n $(sysctl -a | grep brand | grep "Apple") ]]; then
    echo "Setting --platform=linux/amd64 for Mac apple silicon compatibility"
    PLATFORM_CMD="--platform=linux/amd64"
  fi
fi

docker compose -f $DIR/docker-compose.yml up --build -d --wait evm gravity titan

# deploy tkx exchange wasm contract
docker exec -it gravity-with-titan-titan-1 /deploy-contract.sh

docker run -d --cpus 5 --name gravity-with-titan-orchestrator-test \
  $PLATFORM_CMD \
  --network gravity-with-titan_net \
  --mount type=bind,source="$DIR/shared",target=/shared \
  --mount type=bind,source="$DIR/logs/ibc-relayer",target=/ibc-relayer-logs \
  --mount type=bind,source="$REPO_DIR/tests",target=/gravity/tests \
  --mount type=bind,source="$REPO_DIR/orchestrator",target=/gravity/orchestrator \
  -it orchestrator-test

docker exec -it gravity-with-titan-orchestrator-test /setup.sh
