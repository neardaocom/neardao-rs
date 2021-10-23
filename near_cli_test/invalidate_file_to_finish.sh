#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "This file is not valid anymore.", "tags":["test","invalidate"], "description_cid": "reason why i want this to be invalid here"}, "tx_input": { "InvalidateFile": {"cid": "5fb2ae525103eb24bb18d1077942666012345678123450"} }}' --amount $DEPOSIT_ADD_PROPOSAL --gas $TGAS_100 --accountId $CID4 | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID4
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'
near view $DCID doc_files