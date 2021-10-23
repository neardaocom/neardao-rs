#!/bin/bash


# Add key to DAO to FS storage
cp ~/.near-credentials/testnet/$CID.json ~/.near-credentials/testnet/$DCID.json
sed -i 's/dev-/dao.dev-/g' ~/.near-credentials/testnet/$DCID.json


# remove blobs from contracts
near call $CID clean_self --accountId $CID
near call $DCID clean_self --accountId $DCID

# delete accounts
near delete $DCID pstu.testnet; near delete $CID1 pstu.testnet; near delete $CID2 pstu.testnet; near delete $CID3 pstu.testnet; near delete $CID4 pstu.testnet; near delete $CID pstu.testnet; 

# delete dev acc credentials so we can generate a new one
if [ -d ./neardev ]; then
    rm -rf ./neardev
fi