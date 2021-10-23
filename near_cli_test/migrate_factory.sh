#!/bin/bash

UPGRADE=false

if [[ $1 == "t" || $1 == "true" ]]; then
  UPGRADE=true;
fi

near deploy \
  --wasmFile res/dao_factory.wasm \
  --initFunction "migrate" \
  --initArgs '{"dao_version_update":'$UPGRADE'}' \
  --accountId $CID