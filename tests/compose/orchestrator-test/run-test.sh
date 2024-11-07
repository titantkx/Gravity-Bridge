#!/bin/bash

TEST_TYPE=$1
set -eu

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
TEST_TYPE=$TEST_TYPE PATH=$PATH:$HOME/.cargo/bin cargo run --release --bin test-runner
