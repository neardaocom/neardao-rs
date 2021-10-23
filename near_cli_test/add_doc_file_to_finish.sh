#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

# First file with new tags
PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "Testing doc file.", "tags":["test","doc","files"], "description_cid": "someblabla"}, "tx_input": { "AddDocFile": {"cid": "5fb2ae525103eb24bb18d1077942666012345678123450","metadata": { "Curr": {"name":"whitepaper","ext": ".pdf", "description":"first whitepaper", "tags": [], "category":0, "valid": true, "v":"1.0" } }, "new_tags": ["first", "testing", "important"], "new_category": null} }}' --amount $DEPOSIT_ADD_PROPOSAL --gas $TGAS_100 --accountId $CID4 | tail -n1 | tr -d '[:space:]')

# Second file with new category and same tags
#PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "Testing doc file.", "tags":["test","doc","files"], "description_cid": "must be up to 64 chars"}, "tx_input": { "AddDocFile": {"cid": "5fb2ae525103eb24bb18d1077942666012345678123451","metadata":  { "Curr": {"name":"whitepaper","ext": ".pdf", "description":"second whitepaper", "tags": [1,2], "category":0, "valid": true, "v":"1.0" } }, "new_tags": [], "new_category": "other"} }}' --amount $DEPOSIT_ADD_PROPOSAL --gas $TGAS_100 --accountId $CID4 | tail -n1 | tr -d '[:space:]')

# Third file with new category and tags with different lowercase
#PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "Testing doc file.", "tags":["test","doc","files"],  "description_cid": null}, "tx_input": { "AddDocFile": {"cid": "5fb2ae525103eb24bb18d1077942666012345678123452","metadata": { "Curr": {"name":"whitepaper","ext": ".pdf", "description":"third whitepaper", "tags": [1], "category":0, "valid": true, "v":"1.0" } }, "new_tags": ["IMPORTANT"], "new_category": "governance"} }}' --amount $DEPOSIT_ADD_PROPOSAL --gas $TGAS_100 --accountId $CID4 | tail -n1 | tr -d '[:space:]')

echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE --accountId $CID3
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE --accountId $CID4
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'
near view $DCID doc_files