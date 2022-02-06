#!/bin/bash

#source ./near_cli_test/constants.sh
#source ./near_cli_test/init_env.sh
#near call $CID new '{"tags":["dao","test","podilnik"]}' --accountId $CID

##
### init factory
#near view $CID get_tags ''

# prepare args for dao into base64 and init dao vie factory
#ARGS=`echo '{"total_supply": 1000000000,"founders_init_distribution": 10000000,"ft_metadata": {"spec":"ft-1.0.0","name":"Example NEAR fungible token","symbol":"EXAMPLE","icon":"some_icon","reference":null,"reference_hash":null,"decimals":0},"config": {"name": "My first dao", "lang":"en","slogan":"BEST DAO IN EU", "council_share": 25, "description":"Just for testing purposes","vote_spam_threshold": 60},"release_config": [["Council", {"Linear": {"from":null, "duration":600}}]], "vote_policy_configs": [{"proposal_kind": "Pay","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "AddMember","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "RemoveMember","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "GeneralProposal","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "AddDocFile","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "InvalidateFile","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "DistributeFT","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "RightForActionCall","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}],"founders": ["'$CID1'", "'$CID2'", "'$CID3'"]}' | base64`

S_FT_META='{"spec":"ft-1.0.0","name":"Example NEAR fungible token","symbol":"EXAMPLE","icon":"some_icon","reference":null,"reference_hash":null,"decimals":0}'
S_SETTINGS='{"name":"My first dao","purpose":"just testing","tags":[0,1,2],"dao_admin_account_id":"'$CID'","dao_admin_rights":["TODO"],"workflow_provider":"'$WID'"}'
S_GROUPS='[{"settings":{"name":"council","leader":"'$CID1'"},"members":[{"account_id":"'$CID1'","tags":[1]},{"account_id":"'$CID2'","tags":[3,4]},{"account_id":"'$CID3'","tags":[4]}],"release":{"amount":100000000,"init_distribution":10000000,"start_from":0,"duration":1000000000,"model":"Linear"}}]'
S_MEDIA='[]'
S_TAGS='[{"category":"global","values":["test dao", "new", "top"]},{"category":"group","values":["CEO", "CTO", "no idea", "good guy"]},{"category":"media","values":["very important", "probably virus"]}]'
S_FNCALLS='[{"name":"test","receiver": "'$DCID'"}]'
S_FNCALL_META='[ [{"arg_names":["name1", "nullable_obj", "name2", "name3", "obj"], "arg_types":[{"String": false}, {"NullableObject": 1}, "VecString", "VecU128",{"Object":2}]}, {"arg_names": ["test"], "arg_types": [ {"U8":true} ]},{"arg_names":["nested_1_arr_8", "nested_1_obj"], "arg_types":["VecU8",{"Object":3}]},{"arg_names":["nested_2_arr_u64", "bool_val"],"arg_types":["VecU64",{"Bool":false}]}] ]'
S_WFT='[{"name":"wf_add","version":1,"activities":[null,{"code":"wf_add","exec_condition":null,"action":"WorkflowAdd","fncall_id":null,"gas":0,"deposit":0,"arg_types":[{"U16":false},{"Object":0}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}]'
S_WFS='[[{"allowed_proposers":[{"Group":1}],"allowed_voters":"TokenHolder","scenario":"TokenWeighted","activity_rights":[[],[{"GroupLeader":1}]], "duration": 300, "quorum": 30,"approve_threshold": 51,"vote_only_once":true,"spam_threshold":80,"deposit_propose":1000000000000000000000000,"deposit_vote":1000, "deposit_propose_return": 50}]]'

ARGS=`echo '{"total_supply":1000000000,"ft_metadata":'$S_FT_META',"settings":'$S_SETTINGS',"groups":'$S_GROUPS',"media":'$S_MEDIA',"tags":'$S_TAGS',"function_calls":'$S_FNCALLS',"function_call_metadata":'$S_FNCALL_META',"workflow_templates":'$S_WFT',"workflow_template_settings":'$S_WFS'}' | base64`
near call $CID create '{"acc_name": "dao", "dao_info": {"founded_s":9999, "name": "My first dao","description": "Just for testing purposes", "ft_name": "BRO","ft_amount": 1000000000,"tags": [0,1,2]}, "args":"'$ARGS'"}' --accountId $CID --amount $DEPOSIT_CREATE_DAO --gas $MAX_GAS

# near view $CID get_dao_list '{"from_index":0, "limit": 100}'
# near view $DCID proposals '{"from_index":0, "limit": 100}'

##### MIGRATION VIEWS #####

#near view $DCID dao_config ''
#near view $CID get_stats
#near view $CID version_hash '{"version":0}'


###### NEW VIEW CALLS ######

#near state $DCID
#near view $DCID groups ''
#near view $DCID stats_ft ''
#near view $DCID wf_template '{"id":1}'
#near view $DCID wf_templates ''
#near view $DCID check_condition '{"proposal_id": 1, "args": [], "activity_id": 1, "transition_id":null}'
#near view $DCID wf_instance '{"proposal_id":1}'
#near view $DCID group_names ''
#near view $DCID group_members '{"id": 1}'
#near view $DCID dao_settings ''
#near view $DCID tags '{"category": "global"}'
#near view $DCID tags '{"category": "group"}'
#near view $DCID tags '{"category": "media"}'
#near view $DCID storage_buckets ''
#near view $DCID storage_bucket_data_all '{"bucket_id": "workflow"}'
#near view $DCID storage_bucket_data '{"bucket_id": "workflow", "data_id": "action_1_result"}'
#near view $DCID storage_bucket_data '{"bucket_id": "workflow", "data_id": "action_2_result"}'
#near view $DCID storage_bucket_data '{"bucket_id": "workflow", "data_id": "action_3_result"}'
#
####### NEW CALLS ######
#near call $DCID group_create '{"proposal_id": 1, "settings":{"name":"Testers","leader":"machotester.near"},"members":[{"account_id":"machotester.near","tags":[0]}],"token_lock":{"amount":10,"init_distribution":1,"start_from":0,"duration":1000,"model":"Linear"}}' --accountId=$CID
#near call $DCID group_add_member '{"proposal_id":1,"id":3,"members":[{"account_id":"juniortester.near","tags":[3]},{"account_id":"mariotester.near","tags":[3]}]}' --accountId=$CID
#near call $DCID group_remove_member '{"proposal_id":1,"id":3,"member":"juniortester.near"}' --accountId=$CID
#near call $DCID group_update '{"proposal_id":1,"id":3,"settings":{"name":"TOP testers","leader":"pstu.testnet"}}' --accountId=$CID
#near call $DCID group_remove '{"proposal_id":1,"id":3}' --accountId=$CID

####### DEV TEST CALLS ######
#near call $DCID fn_call_validity_test '{"fncall_id": "test_'$DCID'", "names": [["name1", "nullable_obj", "name2", "name3", "obj"] , ["test"], ["nested_1_arr_8", "nested_1_obj"], ["nested_2_arr_u64", "bool_val"]], "args": [["string value", null, ["string arr val 1", "string arr val 2","string arr val 3"], ["100000000000000000000000000", "200","300"]], null, [[4,5,6,7,8,9]], [["9007199254740993", "123", "456"], true] ]}'  --accountId=$CID
#near call $DCID fn_call_validity_test '{"fncall_id": "test_'$DCID'", "names": [["name1", "nullable_obj", "name2", "name3", "obj"] , ["test"], ["nested_1_arr_8", "nested_1_obj"], ["nested_2_arr_u64", "bool_val"]], "args": [["string value", null, ["string arr val 1", "string arr val 2","string arr val 3"], ["100000000000000000000000000", "200","300"]], [null], [[4,5,6,7,8,9]], [["9007199254740993", "123", "456"], true] ]}'  --accountId=$CID

# init storage: 482301 bytes
#near call $DCID storage_add_bucket '{"bucket_id": "workflow"}' --accountId=$CID
#near call $DCID storage_add_data '{"bucket_id": "workflow", "data_id": "action_1_result", "data": { "String": "This is result from action 1"} }' --accountId=$CID
#near call $DCID storage_add_data '{"bucket_id": "workflow", "data_id": "action_2_result", "data": { "VecString": ["This is result from action 2", "and its stored as", "array of strings"]} }' --accountId=$CID
#near call $DCID storage_add_data '{"bucket_id": "workflow", "data_id": "action_3_result", "data": { "VecU8": [1,2,255,0,3,4]}}' --accountId=$CID
#near call $DCID storage_remove_data '{"bucket_id": "workflow", "data_id": "action_3_result"}' --accountId=$CID
#near call $DCID storage_remove_bucket '{"bucket_id": "workflow"}' --accountId=$CID