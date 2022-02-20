#!/bin/bash


near call $DCID media_add '{"proposal_id":2}' --accountId $CID1 --gas $MAX_GAS

#checks
near view $DCID media_list ''