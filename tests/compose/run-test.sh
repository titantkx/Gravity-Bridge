#!/bin/bash
TEST_TYPE=$1
KEEP_CONTAINER=${KEEP_ORCHESTRATOR_TEST_RUNNING:-false}
# Number of gravity validator nodes
export NODES=${NODES:-3}
set -eu

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ -z "${TEST_TYPE}" ]]; then
  echo "No TEST_TYPE provided, HAPPY_PATH should run"
fi

# check if `gravity-with-titan-orchestrator-test` container not exists
if [[ -z $(docker ps -a --format '{{.Names}}' | grep gravity-with-titan-orchestrator-test) ]]; then
  echo "Container gravity-with-titan-orchestrator-test not found => run prepare-test.sh"
  "$DIR"/prepare-test.sh "$NODES"
fi

# check if any in `gravity-with-titan-titan` `gravity-with-titan-gravity` `gravity-with-titan-evm` is not running
if [[ -z $(docker ps --format '{{.Names}}' | grep gravity-with-titan-titan) ]] ||
  [[ -z $(docker ps --format '{{.Names}}' | grep gravity-with-titan-gravity) ]] ||
  [[ -z $(docker ps --format '{{.Names}}' | grep gravity-with-titan-evm) ]]; then
  echo "Containers gravity-with-titan-titan, gravity-with-titan-gravity, gravity-with-titan-evm not found => run prepare-test.sh"
  "$DIR"/prepare-test.sh "$NODES"
fi

# check if container is not running
if [[ -z $(docker ps --format '{{.Names}}' | grep gravity-with-titan-orchestrator-test) ]]; then
  echo "Container gravity-with-titan-orchestrator-test is not running => start it"
  docker start gravity-with-titan-orchestrator-test
fi

# if not `KEEP_CONTAINER` allow to pass the error of the test. Because this script expects to run only one test => error log will print at last
if [[ "$KEEP_CONTAINER" != "true" ]]; then
  set +e
fi
# Run test entry point script
docker exec gravity-with-titan-orchestrator-test /bin/sh -c "/run-test.sh $TEST_TYPE"
set -e

# if `KEEP_CONTAINER` is not set to true, stop the container
if [[ "$KEEP_CONTAINER" != "true" ]]; then
  echo "Stopping container gravity-with-titan-orchestrator-test"
  docker stop gravity-with-titan-orchestrator-test
fi
