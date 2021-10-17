#!/bin/bash


source ./near_cli_test/init_env.sh

# init factory
near call $CID new '{"tags":["dao","test","podilnik"]}' --accountId $CID
near view $CID get_tags ''

# prepare args for dao into base64 and init dao vie factory
ARGS=`echo '{"name": "My first dao","total_supply": 1000000000,"init_distribution": 200000000,"ft_metadata": {"spec":"ft-1.0.0","name":"Example NEAR fungible token","symbol":"EXAMPLE","icon":"some_icon","reference":null,"reference_hash":null,"decimals":0},"config": {"lang":"en", "council_share": 25,"community_share": 10, "foundation_share": 15, "description":"Just for testing purposes","vote_spam_threshold": 60},"release_config": "Voting","vote_policy_configs": [{"proposal_kind": "Pay","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "AddMember","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "RemoveMember","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "RegularPayment","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "GeneralProposal","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "AddDocFile","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}, {"proposal_kind": "InvalidateFile","duration": 60000000000,"waiting_open_duration": 300000000000,"quorum": 20,"approve_threshold": 60,"vote_only_once": true}],"founders": ["'$CID1'", "'$CID2'", "'$CID3'"]}' | base64 -w 0`
near call $CID create '{"acc_name": "dao", "public_key":null,"dao_info": {"name": "My first dao","description": "Just for testing purposes", "ft_name": "BRO","ft_amount": 1000000000,"tags": [0,1,2]}, "args":"'$ARGS'"}' --accountId $CID --amount 5 --gas 100000000000000

near view $DCID statistics_ft ''
near view $DCID statistics_members ''
near call $CID add_tags '{"tags":["service","gaming","goverment"]}' --accountId $CID
near view $CID get_tags ''
near view $CID get_dao_list