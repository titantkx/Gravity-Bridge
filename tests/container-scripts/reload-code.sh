#!/bin/bash
set -eux

# Number of validators to start
NODES=$1

# Stop any currently running peggy and eth processes
pkill peggyd || true # allowed to fail
pkill geth || true # allowed to fail

# Wipe filesystem changes
for i in $(seq 1 $NODES);
do
    rm -rf "/validator$i"
done


cd /peggy/module/
make
make install
cd /peggy/
tests/container-scripts/setup-validators.sh $NODES
tests/container-scripts/run-testnet.sh $NODES

# deploy the ethereum contracts
pushd /peggy/orchestrator
DEPLOY_CONTRACTS=1 RUST_BACKTRACE=full RUST_LOG=INFO PATH=$PATH:$HOME/.cargo/bin cargo run --release --bin test-runner

# This keeps the script open to prevent Docker from stopping the container
# immediately if the nodes are killed by a different process
read -p "Press Return to Close..."