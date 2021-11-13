#!/bin/bash

source ./near_cli_test/init_env.sh
source ./near_cli_test/constants.sh

# init factory
near call $CID new '{"tags":["dao","test","podilnik"]}' --accountId $CID
near view $CID get_tags ''

# prepare args for dao into base64 and init dao vie factory
ARGS=`echo '{"total_supply": 1000000000,"founders_init_distribution": 10000000,"ft_metadata": {"spec":"ft-1.0.0","name":"Example NEAR fungible token","symbol":"EXAMPLE","icon":"some_icon","reference":null,"reference_hash":null,"decimals":0},"config": {"name": "My first dao", "lang":"en","slogan":"BEST DAO IN EU", "council_share": 25,"community_share": 10, "foundation_share": 15, "description":"Just for testing purposes","vote_spam_threshold": 60},"release_config": [["Council", {"Linear": {"from":null, "duration":600}}], ["Community", {"Linear": {"from":1636804800, "duration":3600}}], ["Foundation", "None"]],"vote_policy_configs": [{"proposal_kind": "Pay","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "AddMember","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "RemoveMember","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "RegularPayment","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "GeneralProposal","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "AddDocFile","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true},{"proposal_kind": "InvalidateFile","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "DistributeFT","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}],"founders": ["'$CID1'", "'$CID2'", "'$CID3'"]}' | base64 -w 0`
near call $CID create '{"acc_name": "dao", "public_key":null,"dao_info": {"founded_s":9999, "name": "My first dao","description": "Just for testing purposes", "ft_name": "BRO","ft_amount": 1000000000,"tags": [0,1,2]}, "args":"'$ARGS'"}' --accountId $CID --amount $DEPOSIT_CREATE_DAO --gas $MAX_GAS

near view $DCID statistics_ft ''
near view $DCID statistics_members ''
near view $DCID dao_config ''
near view $DCID vote_policies ''
#near call $CID add_tags '{"tags":["service","gaming","goverment"]}' --accountId $CID
#near view $CID get_tags ''
near view $CID get_dao_list '{"from_index":0, "limit": 100}'
near view $DCID proposals '{"from_index":0, "limit": 100}'