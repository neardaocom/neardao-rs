#!/bin/bash

#

M_TEXT='{"Text":"text..."}'
M_LINK'{"Link":"link"}'
M_CID='{"CID":{"ipfs":"web3.storage.com","cid":"1234567890","mimetype":"image/jpeg"}}'

# Type CID
MEDIA='{"name":"NearDAO logo","category":"logo","media_type":'$M_CID',"tags":[0,3],"version":"1.0","valid":true}'

near call $DCID media_add '{"proposal_id":2,"workflow_id":'$WF_ID', "media":'$MEDIA'}' --accountId $CID1 --gas $MAX_GAS

#checks
near view $DCID media_list ''