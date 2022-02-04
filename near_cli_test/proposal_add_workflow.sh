#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

# transition from start to add 1x, from add to add 2x
S_PROPOSE_SETTINGS='{"activity_rights":[[],[{"GroupLeader":1}]],"activity_inputs":[["Free"]],"transition_constraints":[[{"transition_limit":1,"cond":null}],[{"transition_limit":2,"cond":null}]],"binds":[],"validators":[], "storage_key":"wf_add_wf_1"}'

PUUID=$(near call $DCID propose '{"template_id":1,"template_settings_id":0,"propose_settings":'$S_PROPOSE_SETTINGS'}' --amount 1 --gas $TGAS_100 --accountId $CID1 | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 2}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'