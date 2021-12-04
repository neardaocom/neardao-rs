#!/bin/bash

source ./near_cli_test/constants.sh

UPGRADE=false

if [[ $1 == "t" || $1 == "true" ]]; then
  UPGRADE=true;
fi

near deploy \
  --wasmFile res/dao_factory.wasm \
  --initFunction "migrate" \
  --initArgs '{"dao_version_update":'$UPGRADE'}' \
  --initGas $MAX_GAS \
  --accountId $CID