#!/bin/bash

# deploy dev account with factory contract
CID=$(near dev-deploy --wasmFile=res/dao_factory.wasm | sed -n 's/\(.*\Account id: \)\(dev-[0-9]*-[0-9]*\)\(.*\)/\2/p')
DCID="dao.$CID"; CID1="first.$CID"; CID2="second.$CID"; CID3="third.$CID"; CID4="fourth.$CID"; WID="wf-provider.$CID"
SID=demo-skyward.petrstudynka.testnet
RID=ref.petrstudynka.testnet

# deploy workflow provider
near create-account $WID --masterAccount $CID --initialBalance 15; near deploy --wasmFile res/workflow_provider.wasm --accountId $WID

# setup other accounts 
near create-account $CID1 --masterAccount $CID --initialBalance 15; \
near create-account $CID2 --masterAccount $CID --initialBalance 15; \
near create-account $CID3 --masterAccount $CID --initialBalance 15; \
near create-account $CID4 --masterAccount $CID --initialBalance 15