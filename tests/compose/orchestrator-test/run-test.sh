#!/bin/bash

TEST_TYPE=$1
set -eux

source /set-env.sh

FILE=/contracts
if test -f "$FILE"; then
  echo "Contracts already deployed, running tests"
else
  echo "Contracts not deployed yet, running setup"
  /setup.sh
fi

set +e
killall -9 test-runner
set -e

pushd /gravity/orchestrator/test_runner
RUST_BACKTRACE=full TEST_TYPE=$TEST_TYPE RUST_LOG=INFO PATH=$PATH:$HOME/.cargo/bin cargo run --release --bin test-runner
