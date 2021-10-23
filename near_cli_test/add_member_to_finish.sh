#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "I want to contribute to your company.", "tags":["test","first","money"], "description_cid": null}, "tx_input": { "AddMember": {"group": "Council", "account_id":"'$CID4'"}}}' --amount $DEPOSIT_ADD_PROPOSAL --gas $TGAS_100 --accountId $CID | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 2}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 2}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID4
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID statistics_ft ''
near view $DCID statistics_members ''
near view $DCID proposal '{"proposal_id": '$PUUID'}'