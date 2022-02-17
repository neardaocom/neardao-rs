#!/bin/bash

DEPOSIT=0

# 1. activity
CODE_1='checkin'
ARGS_1='[]'

# 2. activity
CODE_2='unrealized'
ARGS_2='[]'

# 3. activity
CODE_3='approve'
ARGS_3='[{"Bool":true}]'

# 4. activity
CODE_4='done'
ARGS_4='[{"String":"put in a link"}]'

# 5. activity
CODE_5='done_approve'
ARGS_5='[{"Bool":true},{"String":"All good - 5/5"}]'

CODE=$CODE_1
ARGS=$ARGS_1

near call $DCID event '{"proposal_id":2,"code":"'$CODE'","args":'$ARGS'}' --deposit $DEPOSIT --accountId $CID1 --gas $TGAS_200
#near call $DCID treasury_send_near '{"proposal_id":2,"receiver_id":"'$CID1'","amount":"5000000000000000000000000"}' --accountId $CID3 --gas $TGAS_100

# checks
near view $DCID storage_bucket_data_all '{"bucket_id":"wf_bounty_1"}'