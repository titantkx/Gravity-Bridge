#!/bin/bash
set -eux

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOFOLDER=$DIR/../..

# change our directory sot hat the git arcive command works as expected
pushd "$REPOFOLDER"

# setup for Mac apple silicon Compatibility
PLATFORM_CMD=""
if [[ "$OSTYPE" == "darwin"* ]]; then
  if [[ -n $(sysctl -a | grep brand | grep "Apple") ]]; then
    echo "Setting --platform=linux/amd64 for Mac apple silicon compatibility"
    PLATFORM_CMD="--platform=linux/amd64"
  fi
fi
docker build --build-context repo=$REPOFOLDER --ulimit nofile=65536:65536 -t orchestrator-test $PLATFORM_CMD "$DIR"/orchestrator-test
