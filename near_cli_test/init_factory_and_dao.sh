#!/bin/bash

source ./near_cli_test/init_env.sh
source ./near_cli_test/constants.sh

# init factory
near call $CID new '{"tags":["dao","test","podilnik"]}' --accountId $CID
near view $CID get_tags ''

# prepare args for dao into base64 and init dao vie factory
#ARGS=`echo '{"total_supply": 1000000000,"founders_init_distribution": 10000000,"ft_metadata": {"spec":"ft-1.0.0","name":"Example NEAR fungible token","symbol":"EXAMPLE","icon":"some_icon","reference":null,"reference_hash":null,"decimals":0},"config": {"name": "My first dao", "lang":"en","slogan":"BEST DAO IN EU", "council_share": 25, "description":"Just for testing purposes","vote_spam_threshold": 60},"release_config": [["Council", {"Linear": {"from":null, "duration":600}}]], "vote_policy_configs": [{"proposal_kind": "Pay","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "AddMember","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "RemoveMember","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "GeneralProposal","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "AddDocFile","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "InvalidateFile","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "DistributeFT","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "RightForActionCall","duration": 45000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}],"founders": ["'$CID1'", "'$CID2'", "'$CID3'"]}' | base64`

ARGS=`echo '{"deposit_min_vote": "1","deposit_min_add_proposal": "1","total_supply": 1000000000,"ft_metadata": {"spec":"ft-1.0.0","name":"Example NEAR fungible token","symbol":"EXAMPLE","icon":"some_icon","reference":null,"reference_hash":null,"decimals":0},"settings": {"name": "My first dao", "purpose":"just testing","tags":[0,1,2], "dao_admin_account_id": "'$CID'", "dao_admin_rights":["all"],"workflow_provider": "market.neardao.near"},"vote_settings":[{"scenario":"Democratic", "duration": 300, "quorum": 30,"approve_threshold": 30,"spam_threshold":50},{"scenario": "TokenWeighted", "duration":400,"quorum":45,"approve_threshold":50,"spam_threshold":30}],"groups":[{"settings":{"name":"Council","leader":"pstu.testnet"},"members":[{"account_id":"pstu.testnet","tags":[0,1]},{"account_id":"somebody.testnet","tags":[2,3]},{"account_id":"nobody.testnet","tags":[3]}],"release":{"amount":100000000,"init_distribution":10000000,"start_from":0,"duration":1000000000,"model":"Linear"}},{"settings":{"name":"Cleaners","leader":"pstu.testnet"},"members":[{"account_id":"topclear.testnet","tags":[0]}],"release":{"amount":1000,"init_distribution":100,"start_from":0,"duration":1000000000,"model": "None"}}],"media":[],"tags":[{"category":"global","values":["test dao", "new", "top"]},{"category":"group","values":["CEO", "CTO", "no idea", "good guy"]},{"category":"media","values":["very important", "probably virus"]}], "function_calls": [{"name":"test","receiver": "'$DCID'"}],"function_call_metadata": [ [{"arg_names":["name1", "name2", "name3", "obj"], "arg_types":["String", "VecString", "VecU128",{"Object":1}]},{"arg_names":["nested_1_arr_8", "nested_1_obj"], "arg_types":["VecU8", { "Object":2}]}, {"arg_names":["nested_2_arr_u64", "bool_val"], "arg_types": ["VecU64","Bool"]}] ], "workflow_templates": []}' | base64`
near call $CID create '{"acc_name": "dao", "public_key":null,"dao_info": {"founded_s":9999, "name": "My first dao","description": "Just for testing purposes", "ft_name": "BRO","ft_amount": 1000000000,"tags": [0,1,2]}, "args":"'$ARGS'"}' --accountId $CID --amount $DEPOSIT_CREATE_DAO --gas $MAX_GAS

#near view $DCID statistics_ft ''
#near view $DCID statistics_members ''
#near view $DCID dao_config ''
#near view $DCID vote_policies ''
#near call $CID add_tags '{"tags":["service","gaming","goverment"]}' --accountId $CID
#near view $CID get_tags ''
# near view $CID get_dao_list '{"from_index":0, "limit": 100}'
# near view $DCID proposals '{"from_index":0, "limit": 100}'

##### MIGRATION VIEWS #####

#near view $DCID dao_config ''
#near view $CID get_stats
#near view $CID version_hash '{"version":0}'


###### NEW VIEW CALLS ######

#near view $DCID groups ''
#near view $DCID group_names ''
#near view $DCID group_members '{"id": 1}'
#near view $DCID dao_settings ''
#near view $DCID vote_settings ''
#near view $DCID tags '{"category": "global"}'
#near view $DCID tags '{"category": "group"}'
#near view $DCID tags '{"category": "media"}'
#
####### NEW CALLS ######
#near call $DCID group_create '{"proposal_id": 1, "settings":{"name":"Testers","leader":"machotester.near"},"members":[{"account_id":"machotester.near","tags":[0]}],"token_lock":{"amount":10,"init_distribution":1,"start_from":0,"duration":1000,"model":"Linear"}}' --accountId=$CID
#near call $DCID group_add_member '{"proposal_id":1,"id":3,"members":[{"account_id":"juniortester.near","tags":[3]},{"account_id":"mariotester.near","tags":[3]}]}' --accountId=$CID
#near call $DCID group_remove_member '{"proposal_id":1,"id":3,"member":"juniortester.near"}' --accountId=$CID
#near call $DCID group_update '{"proposal_id":1,"id":3,"settings":{"name":"TOP testers","leader":"pstu.testnet"}}' --accountId=$CID
#near call $DCID group_remove '{"proposal_id":1,"id":3}' --accountId=$CID

####### DEV TEST CALLS ######
#near call $DCID test '{"fncall_id": "test_'$DCID'", "names": [["name1", "name2", "name3", "obj"] , ["nested_1_arr_8", "nested_1_obj"], ["nested_2_arr_u64", "bool_val"]], "args": [["string value", ["string arr val 1", "string arr val 2","string arr val 3"], ["100000000000000000000000000", "200","300"]], [[4,5,6,7,8,9]], [["9007199254740993", "123", "456"], true] ]}'  --accountId=$CID