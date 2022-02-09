#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

S_PROPOSE_SETTINGS='{"activity_inputs":[[["Free","Free"]]],"transition_constraints":[[{"transition_limit":1,"cond":null}],[{"transition_limit":4,"cond":null}]],"binds":[{"U128":"1000000000000000000000000"}],"obj_validators":[[{"Primitive":0}]],"validator_exprs":[{"args":[{"User":1},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"GtE"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"storage_key":"wf_send_near_1"}'
S_TEMPLATE_SETTINGS='null'

PUUID=$(near call $DCID propose '{"template_id":2,"template_settings_id":0,"propose_settings":'$S_PROPOSE_SETTINGS',"template_settings": '$S_TEMPLATE_SETTINGS'}' --amount 1 --gas $TGAS_100 --accountId $CID1 | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 2}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'