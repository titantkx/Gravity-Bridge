#!/bin/bash

set -eux

./build-orchestrator-test.sh

# start up evm, gravity and titan
docker compose up --build -d --wait evm gravity titan
docker compose stop orchestrator

# setup
# setup for Mac apple silicon compatibility
PLATFORM_CMD=""
if [[ "$OSTYPE" == "darwin"* ]]; then
  if [[ -n $(sysctl -a | grep brand | grep "Apple") ]]; then
    echo "Setting --platform=linux/amd64 for Mac apple silicon compatibility"
    PLATFORM_CMD="--platform=linux/amd64"
  fi
fi

# Remove existing container instance
set +e
docker rm -f gravity-with-titan-orchestrator-test
set -e

docker run --name gravity-with-titan-orchestrator-test $PLATFORM_CMD --network gravity-with-titan_net -it orchestrator-test /bin/bash /setup.sh
