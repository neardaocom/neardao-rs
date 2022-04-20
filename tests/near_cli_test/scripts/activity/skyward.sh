#!/bin/bash

# 1. activity
RECEIVER_1=$SID
METHOD_1='register_tokens'
ARGS_1='[[{"String":"wrap.testnet"}]]'
ARGS_COL_1='[]'

# 2. activity
RECEIVER_2='self'
METHOD_2='storage_deposit'
ARGS_2='[[]]'
ARGS_COL_2='[]'

# 3. activity
RECEIVER_3='wrap.testnet'
METHOD_3='storage_deposit'
ARGS_3='[[]]'
ARGS_COL_3='[]'

# 4. activity
RECEIVER_4='self'
METHOD_4='ft_transfer_call'
ARGS_4='[[{"String":"memo msg value"}]]'
ARGS_COL_4='[]'

# 5. activity
RECEIVER_5=$SID
METHOD_5='sale_create'
ARGS_5='[[],[]]' # Binded from proposal settings
ARGS_COL_5='[["Null","Null","Null"]]' # Requires placeholder values for now, will be fixed

RECEIVER=$RECEIVER_5
METHOD=$METHOD_5
ARGS=$ARGS_5
ARGS_COLLECTION=$ARGS_COL_5

near call $DCID fn_call '{"proposal_id":4,"fncall_receiver":"'$RECEIVER'","fncall_method":"'$METHOD'","arg_values":'$ARGS', "arg_values_collection":'$ARGS_COLLECTION'}' --accountId $CID1 --gas $TGAS_200

# checks
near view $DCID wf_instance '{"proposal_id": 4}'
near view $DCID storage_bucket_data_all '{"bucket_id":"wf_skyward_2"}'
#near view $SID get_sales '{"account_id": "'$DCID'"}'