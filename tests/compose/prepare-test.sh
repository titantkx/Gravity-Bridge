#!/bin/bash

set -eux

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$DIR/../.."

$DIR/build-orchestrator-test.sh

# start up evm, gravity and titan
docker compose -f $DIR/docker-compose.yml down

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

docker run -d --name gravity-with-titan-orchestrator-test \
  $PLATFORM_CMD \
  --network gravity-with-titan_net \
  --mount type=bind,source="$DIR/shared",target=/shared \
  --mount type=bind,source="$REPO_DIR/orchestrator",target=/gravity/orchestrator \
  -it orchestrator-test

docker compose -f $DIR/docker-compose.yml up --build -d --wait evm gravity titan

docker exec gravity-with-titan-orchestrator-test /setup.sh
