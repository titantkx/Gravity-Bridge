# Compose test setup

This directory contains setup for test whole gravity system with 1 evm chain and 1 titan chain. Config each node in separate service in docker compose.

## Folder structure

- evm: evm chain node setup
- gravity: gravity node setup
- titan: titan chain node setup
- shared: shared information between nodes
- logs: contains logs from nodes

## Scripts

- [prepare-test.sh](prepare-test.sh): rebuild and start testing environment. Include `evm`, `gravity` and `titan` nodes docker container. Rerun this script to reset the environment.
- [run-test.sh](run-test.sh): run test cases.
  - e.g. `./run-test.sh HAPPY_PATH_V2` will run test case `HAPPY_PATH_V2`.
- [run-test-all.sh](run-test-all.sh): automatically run all repeatable test cases.
- [build-orchestrator-test.sh](build-orchestrator-test.sh): build docker image for orchestrator test.

## How to run

1. Prepare test environment by running `./prepare-test.sh`.
2. Run test cases by running `./run-test.sh <test-case-name>`. Or run all repeatable test cases by running `./run-test-all.sh`.
