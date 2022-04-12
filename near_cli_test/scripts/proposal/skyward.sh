#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

# WF ID from provider
WFT_ID=2
STORAGE_KEY="wf_skyward_2"
S_PROPOSE_SETTINGS='{"binds":[{"String":"wrap.testnet"},{"U128":"1000"},{"String":"NearDAO auction"},{"String":"www.neardao.com"},{"U64":"1653304093000000000"},{"U64":"604800000000000"}],"storage_key":"'$STORAGE_KEY'"}'
S_TEMPLATE_SETTINGS='null'


PUUID=$(near call $DCID propose '{"content": null, "desc":"test","template_id":'$WFT_ID',"template_settings_id":0,"propose_settings":'$S_PROPOSE_SETTINGS',"template_settings":'$S_TEMPLATE_SETTINGS'}' --amount 1 --gas $TGAS_100 --accountId $CID1 | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'