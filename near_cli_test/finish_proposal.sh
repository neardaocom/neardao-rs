#!/bin/bash

near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $TGAS_100 --accountId $CID
near view $DCID proposal '{"proposal_id": '$PUUID'}'