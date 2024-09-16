#!/bin/bash
set -eux

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKERFOLDER=$DIR/dockerfile
REPOFOLDER=$DIR/..

# change our directory sot hat the git arcive command works as expected
pushd $REPOFOLDER

#docker system prune -a -f
# Build base container
git archive --format=tar.gz -o $DOCKERFOLDER/gravity.tar.gz --prefix=gravity/ HEAD
git archive --format=tar.gz -o $DOCKERFOLDER/module.tar.gz --prefix=gravity/module/ HEAD:module/
git archive --format=tar.gz -o $DOCKERFOLDER/solidity.tar.gz --prefix=gravity/solidity/ HEAD:solidity/
git archive --format=tar.gz -o $DOCKERFOLDER/orchestrator.tar.gz --prefix=gravity/orchestrator/ HEAD:orchestrator/
pushd $DOCKERFOLDER

# setup for Mac apple silicon Compatibility
PLATFORM_CMD=""
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ -n $(sysctl -a | grep brand | grep "Apple") ]]; then
        echo "Setting --platform=linux/amd64 for Mac apple silicon compatibility"
        PLATFORM_CMD="--platform=linux/amd64"
    fi
fi
docker build --ulimit nofile=65536:65536 -t gravity-base $PLATFORM_CMD .
