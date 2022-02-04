#!/bin/bash

BEN=petrstudynka.testnet

# remove blobs from contracts
near call $CID clean_self --accountId $CID
near call $DCID clean_self --accountId $CID1
near call $DCID delete_self --accountId $CID1

# delete accounts
near delete $CID1 $BEN; near delete $CID2 $BEN; near delete $CID3 $BEN; near delete $CID4 $BEN; near delete $CID $BEN;
near delete $WID $BEN

# delete dev acc credentials so we can generate a new one
if [ -d ./neardev ]; then
    rm -rf ./neardev
fi