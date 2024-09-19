#!/bin/bash
set -ex
# the directory of this script, useful for allowing this script
# to be run with any PWD
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $DIR
bash run-tests.sh # Happy path
export NO_IMAGE_BUILD=1
bash run-tests.sh VALSET_STRESS
bash run-tests.sh BATCH_STRESS
bash run-tests.sh HAPPY_PATH_V2
bash run-tests.sh ORCHESTRATOR_KEYS
bash run-tests.sh VALSET_REWARDS
bash run-tests.sh TXCANCEL
bash run-tests.sh INVALID_EVENTS
bash run-tests.sh UNHALT_BRIDGE
bash run-tests.sh PAUSE_BRIDGE
bash run-tests.sh ETHEREUM_BLACKLIST
bash run-tests.sh AIRDROP_PROPOSAL
bash run-tests.sh SIGNATURE_SLASHING
bash run-tests.sh IBC_METADATA
bash run-tests.sh ERC721_HAPPY_PATH
bash run-tests.sh IBC_AUTO_FORWARD
bash run-tests.sh ETHEREUM_KEYS
bash run-tests.sh BATCH_TIMEOUT
# bash run-tests.sh VESTING
bash run-tests.sh SEND_TO_ETH_FEES
if [ ! -z "$ALCHEMY_ID" ]; then
  bash run-tests.sh RELAY_MARKET $ALCHEMY_ID
  bash run-tests.sh ARBITRARY_LOGIC $ALCHEMY_ID
else
  echo "Alchemy API key not set under variable ALCHEMY_ID, not running ARBITRARY_LOGIC nor RELAY_MARKET"
fi
# `SLASHING_DELEGATION` will make validator get slashed and jailed
bash run-tests.sh SLASHING_DELEGATION
# `VALIDATOR_OUT` test will make validator get slashed and jailed
bash run-tests.sh VALIDATOR_OUT
# move EVIDENCE to the end because it will change validator set (jail one validator) make `UNHALT_BRIDGE` fail
bash run-tests.sh EVIDENCE
# `DEPOSIT_OVERFLOW` test must be the last one because it will break bridge, because we fake event from ethereum make nonce mismatch between ethereum contract and gravity bridge
bash run-tests.sh DEPOSIT_OVERFLOW

echo "All tests succeeded!"
