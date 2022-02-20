#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

STORAGE_KEY="wf_add_wf_1"

WF_SETTINGS_BASIC_WF='[{"allowed_proposers":[{"Group":1}],"allowed_voters":"TokenHolder","activity_rights":[[{"GroupLeader":1}]],"transition_constraints":[[{"transition_limit":1,"cond":null}]],"scenario":"TokenWeighted","duration":25,"quorum":51,"approve_threshold":20,"spam_threshold":80,"vote_only_once":true,"deposit_propose":"1","deposit_vote":"1000","deposit_propose_return":0}]'
WF_SETTINGS_SEND_NEAR='[{"allowed_proposers":[{"Group":1}],"allowed_voters":"TokenHolder","activity_rights":[[{"GroupLeader":1}]],"transition_constraints":[[{"transition_limit":1,"cond":null}]],"scenario":"TokenWeighted","duration":25,"quorum":51,"approve_threshold":20,"spam_threshold":80,"vote_only_once":true,"deposit_propose":"1000","deposit_vote":"1000","deposit_propose_return":0}]'
WF_SETTINGS_SKYWARD='[{"allowed_proposers":[{"Group":1}],"allowed_voters":"TokenHolder","activity_rights":[[{"GroupLeader":1}],[{"GroupLeader":1}],[{"GroupLeader":1}],[{"GroupLeader":1}],[{"GroupLeader":1}]],"transition_constraints":[[{"transition_limit":1,"cond":null}],[{"transition_limit":1,"cond":null},{"transition_limit":1,"cond":null},{"transition_limit":1,"cond":null}],[{"transition_limit":1,"cond":null},{"transition_limit":1,"cond":null}],[{"transition_limit":1,"cond":null}],[{"transition_limit":1,"cond":null}]],"scenario":"TokenWeighted","duration":25,"quorum":51,"approve_threshold":20,"spam_threshold":80,"vote_only_once":true,"deposit_propose":"1","deposit_vote":"1000","deposit_propose_return":0}]'
WF_SETTINGS_BOUNTY='[{"allowed_proposers":[{"Group":1}],"allowed_voters":"TokenHolder","activity_rights":[["Anyone"],["Anyone"],[{"Group":1}],[{"Group":1}],[{"Group":1}],[{"GroupLeader":1}]],"transition_constraints":[[{"transition_limit":1,"cond":null}],[{"transition_limit":5,"cond":null},{"transition_limit":5,"cond":null}],[{"transition_limit":4,"cond":null}],[{"transition_limit":4,"cond":{"args":[{"Storage":"pp_3"}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":0},{"Value":{"Bool":false}}]}}}},{"transition_limit":4,"cond":{"args":[{"Storage":"pp_1"},{"User":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":0},{"Arg":1}]}}}},{"transition_limit":1,"cond":{"args":[{"Storage":"pp_3"}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":0},{"Value":{"Bool":true}}]}}}}],[{"transition_limit":1,"cond":null}],[{"transition_limit":1,"cond":null}]],"scenario":"TokenWeighted","duration":25,"quorum":51,"approve_threshold":20,"spam_threshold":80,"vote_only_once":true,"deposit_propose":"1","deposit_vote":"1000","deposit_propose_return":0}]'

# WF ID from provider
WF_ID=6
S_PROPOSE_SETTINGS='{"binds":[{"U16":'$WF_ID'}],"storage_key":"'$STORAGE_KEY'"}'
S_TEMPLATE_SETTINGS=$WF_SETTINGS_BASIC_WF

PUUID=$(near call $DCID propose '{"content": null, "desc":"test","template_id":1,"template_settings_id":0,"propose_settings":'$S_PROPOSE_SETTINGS',"template_settings":'$S_TEMPLATE_SETTINGS'}' --amount 1 --gas $TGAS_100 --accountId $CID1 | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'