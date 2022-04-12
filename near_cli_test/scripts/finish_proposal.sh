#!/bin/bash

near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $MAX_GAS --accountId $CID
near view $DCID proposal '{"proposal_id": '$PUUID'}'