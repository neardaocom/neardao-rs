#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

# First file with new tags
PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "Testing doc file.", "tags":["test","doc","files"], "transaction": { "AddDocFile": {"uuid": "5fb2ae525103eb24bb18d1077942666012345678123450","metadata": {"name":"whitepaper","ext": ".pdf", "description":"first whitepaper", "tags": [], "category":0, "valid": true, "v":"1.0" }, "new_tags": ["first", "testing", "important"], "new_category": null} }}}' --amount 0.5 --gas 100000000000000 --accountId $CID4 | tail -n1 | tr -d '[:space:]')

# Second file with new category and same tags
#PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "Testing doc file.", "tags":["test","doc","files"], "transaction": { "AddDocFile": {"uuid": "5fb2ae525103eb24bb18d1077942666012345678123451","metadata": {"name":"whitepaper","ext": ".pdf", "description":"second whitepaper", "tags": [1,2], "category":0, "valid": true, "v":"1.0" }, "new_tags": [], "new_category": "other"} }}}' --amount 0.5 --gas 100000000000000 --accountId $CID4 | tail -n1 | tr -d '[:space:]')

# Third file with new category and tags with different lowercase
#PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "Testing doc file.", "tags":["test","doc","files"], "transaction": { "AddDocFile": {"uuid": "5fb2ae525103eb24bb18d1077942666012345678123452","metadata": {"name":"whitepaper","ext": ".pdf", "description":"third whitepaper", "tags": [1], "category":0, "valid": true, "v":"1.0" }, "new_tags": ["IMPORTANT"], "new_category": "governance"} }}}' --amount 0.5 --gas 100000000000000 --accountId $CID4 | tail -n1 | tr -d '[:space:]')

echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas 10000000000000 --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas 10000000000000 --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas 10000000000000 --accountId $CID3
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas 10000000000000 --accountId $CID4
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas 100000000000000 --accountId $CID4

near view $DCID proposal '{"proposal_id": '$PUUID'}'
near view $DCID doc_files