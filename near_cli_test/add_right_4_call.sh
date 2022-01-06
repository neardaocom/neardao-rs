#!/bin/bash

#source ./near_cli_test/init_factory_and_dao.sh

BEN=petrstudynka.testnet
RID=pstu.testnet
SID=supertest.testnet


near send $BEN $DCID 20 --accountId=$BEN

PUUID=$(near call $DCID add_proposal '{"proposal_input" : {"description": "Testing ref.", "tags":["test","first","money"], "description_cid": null}, "tx_input": { "RightForActionCall": {"to": {"Group": {"value" :"Council"}},"rights": ["RefFinance","SkywardFinance"], "time_from": null, "time_to": null}}}' --amount $DEPOSIT_ADD_PROPOSAL --gas $TGAS_100 --accountId $CID4 | tail -n1 | tr -d '[:space:]')
echo "Created proposal UUID: $PUUID"

near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID1
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID2
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID3
near call $DCID vote '{"proposal_id":'$PUUID',"vote_kind": 1}' --gas $TGAS_10 --amount $DEPOSIT_VOTE  --accountId $CID4
near call $DCID finish_proposal '{"proposal_id":'$PUUID'}' --gas $MAX_GAS --accountId $RID

near view $DCID proposal '{"proposal_id": '$PUUID'}'
near view $DCID statistics_members ''

######### Ref-Finance testnet setup #########
# near deploy $RID ref_exchange_release.wasm
# near call $RID new '{"owner_id": "'$RID'", "exchange_fee": 4, "referral_fee": 1}' --accountId=$RID
# near call $RID storage_deposit '' --accountId=$RID --depositYocto=1

######### TEST interaction with REF #########
# near call $DCID execute_privileged_action '{"action": "RefRegisterTokens"}' --accountId=$CID1 --gas $MAX_GAS
# POOL_ID=$(near call $DCID execute_privileged_action '{"action": "RefAddPool": {"fee": 25}}' --accountId=$CID2 --gas $MAX_GAS | tail -n1 | tr -d '[:space:]')
# near call $DCID execute_privileged_action '{"action": { "RefAddLiquidity": { "pool_id": '$POOL_ID', "amount_near": "10", "amount_ft": "10" }}}' --accountId=$CID3 --gas $MAX_GAS
# near call $DCID execute_privileged_action '{"action": { "RefWithdrawLiquidity": { "pool_id" : '$POOL_ID', "shares": "2000000000000000000000000", "min_ft": "1", "min_near": "1"}}}' --accountId=$CID1 --gas $MAX_GAS
# near call $DCID execute_privileged_action '{"action": { "RefWithdrawDeposit": {"token_id":"'$DCID'","amount":"10"}}}' --accountId=$CID1 --gas $MAX_GAS
# near call $DCID execute_privileged_action '{"action": { "RefWithdrawDeposit": {"token_id":"wrap.testnet","amount":"10"}}}' --accountId=$CID1 --gas $MAX_GAS

# near view $DCID ref_pools ''
# near view $RID get_pools '{"from_index": 44, "limit": 5}'
# near view $RID get_whitelisted_tokens
# near view $RID get_pool_shares '{"pool_id": '$POOL_ID', "account_id": "'$DCID'"}'
# near view $RID get_deposits '{"account_id": "'$DCID'"}'



######### SkywardFinance testnet setup #########
# near deploy $RID skyward.wasm
# near call $SID --accountId=$SID new '{"skyward_token_id": "'$SKYWARD_TOKEN_ID'","skyward_vesting_schedule": [], "listing_fee_near": "1000000000000000000000000", "w_near_token_id": "wrap.testnet"}'

######### TEST interaction with Skyward #########
# near call $DCID execute_privileged_action '{"action": { "SkyCreateSale": { "title": "NEARDAO TEST", "url":"neardao.com", "amount_ft": "10", "out_token_id": "wrap.testnet", "time_from": "1643649200000000000", "duration": "3600000000000" }}}' --accountId=$CID1 --gas $MAX_GAS

# near view $SID get_sales
# near view $SID get_sales_by_id '{"account_id": "dao.dev-1639757148730-51231357207842", "sale_ids": [1,3]}'