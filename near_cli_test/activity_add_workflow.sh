#!/bin/bash

WF_ID=6
near call $DCID workflow_add '{"proposal_id":1,"workflow_id":'$WF_ID'}' --accountId $CID1 --gas $MAX_GAS