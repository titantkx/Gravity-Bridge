#!/bin/bash

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$DIR/../.."

pushd $DIR/.. || exit

npx ts-node \
  contract-deployer.ts \
  --cosmos-node="http://localhost:26657" \
  --eth-node="http://localhost:8545" \
  --eth-privkey="0xb1bab011e03a9862664706fc3bbaa1b16651528e5f0e7fbfcbfdd8be302a13e7" \
  --contract="$REPO_DIR/solidity/artifacts/contracts/Gravity.sol/Gravity.json" \
  --contractERC721="$REPO_DIR/solidity/artifacts/contracts/GravityERC721.sol/GravityERC721.json" \
  --evm-prefix=ethereum \
  --test-mode=true
