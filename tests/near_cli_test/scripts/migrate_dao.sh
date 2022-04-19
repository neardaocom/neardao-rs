#!/bin/bash

# Add key to DAO to FS storage
cp ~/.near-credentials/testnet/$CID.json ~/.near-credentials/testnet/$DCID.json
sed -i 's/dev-/dao.dev-/g' ~/.near-credentials/testnet/$DCID.json


near deploy \
  --wasmFile res/dao.wasm \
  --initFunction "migrate" \
  --initArgs "{}" \
  --accountId $DCID