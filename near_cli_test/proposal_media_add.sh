#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

M_TEXT='{"Text":"text..."}'
M_LINK='{"Link":"link"}'
M_CID='{"CID":{"ipfs":"web3.storage.com","cid":"1234567890","mimetype":"image/jpeg"}}'

# Type CID
MEDIA='{"proposal_id":0,"name":"NearDAO logo","category":"logo","media_type":'$M_CID',"tags":[0,3],"version":"1.0","valid":true}'

WFT_ID=2
STORAGE_KEY="wf_media_add_1"
S_PROPOSE_SETTINGS='{"binds":[],"storage_key":"'$STORAGE_KEY'"}'
S_TEMPLATE_SETTINGS='null'

CONTENT='{"Media":'$MEDIA'}'

PUUID=$(near call $DCID propose '{"content":'$CONTENT', "desc":"test","template_id":'$WFT_ID',"template_settings_id":0,"propose_settings":'$S_PROPOSE_SETTINGS',"template_settings": '$S_TEMPLATE_SETTINGS'}' --amount 1 --gas $TGAS_100 --accountId $CID1 | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'
near view $DCID media_list ''