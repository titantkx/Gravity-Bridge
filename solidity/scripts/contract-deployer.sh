#!/bin/bash

# this directy of this script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$DIR/../.."

pushd $DIR/.. || exit

export HARDHAT_NETWORK="http://localhost:8545"

npx ts-node \
  --files contract-deployer.ts \
  --cosmos-node="http://localhost:26657" \
  --contract="$REPO_DIR/solidity/artifacts/contracts/Gravity.sol/Gravity.json" \
  --contractERC721="$REPO_DIR/solidity/artifacts/contracts/GravityERC721.sol/GravityERC721.json" \
  --evm-prefix=ethereum \
  --test-mode=true
