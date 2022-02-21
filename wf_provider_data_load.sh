#!/bin/bash

### Workflow template data for Workflow provider

GREEN='\033[0;32m'
NC='\033[0m'

# ------------ SETTINGS ------------ #

WFSTART=1
WFCOUNT=13
WID=$1

if [ -z $WID ]; then
    WID='wf-provider.neardao.testnet';
fi

# ------------ DATA ------------ #

# WFT - Workflow Add
WF1='{"name":"wf_add","version":1,"activities":[null,{"code":"wf_add","exec_condition":null,"action":"WorkflowAdd","action_data":null,"arg_types":[{"U16":false}],"activity_inputs":[[{"Bind":0}]],"postprocessing":null,"must_succeed":true}],"obj_validators":[[{"Primitive":0}]],"validator_exprs":[{"args":[{"User":0},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF1FNS='[]'
WF1FNMETA='[]'

# WFT - Near Send
WF2='{"name":"wf_near_send","version":1,"activities":[null,{"code":"near_send","exec_condition":null,"action":"TreasurySendNear","action_data":null,"arg_types":[{"String":false},{"U128":false}],"activity_inputs":[["Free","Free"]],"postprocessing":null,"must_succeed":true}],"obj_validators":[[{"Primitive":0},{"Primitive":0}]],"validator_exprs":[{"args":[{"User":0},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}},{"args":[{"User":1},{"Bind":1}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"GtE"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF2FNS='[]'
WF2FNMETA='[]'

## WFT - FT Send
WF3='{"name":"wf_treasury_send_ft","version":1,"activities":[null,{"code":"treasury_send_ft","exec_condition":null,"action":"TreasurySendFt","action_data":null,"arg_types":[{"String":false},{"String":false},{"U128":false},{"String":true}],"activity_inputs":[["Free","Free","Free","Free"]],"postprocessing":null,"must_succeed":true}],"obj_validators":[[{"Primitive":0},{"Primitive":0},{"Primitive":0}]],"validator_exprs":[{"args":[{"User":0},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}},{"args":[{"User":1},{"Bind":1}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}},{"args":[{"User":2},{"Bind":2}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"GtE"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF3FNS='[]'
WF3FNMETA='[]'
#
## WFT - Group Add
#WF4='{"name":"wf_group_add","version":1,"activities":[null,{"code":"add_group","exec_condition":null,"action":"GroupAdd","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"Object":1},{"VecObject":2},{"Object":3}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'
#WF4FNS='[]'
#WF4FNMETA='[]'
#
# WFT - Group Members Add
WF5='{"name":"wf_group_members_add","version":1,"activities":[null,{"code":"group_members_add","exec_condition":null,"action":"GroupAddMembers","action_data":null,"arg_types":[{"U16":false},{"VecObject":1}],"activity_inputs":[["Free","Free"]],"postprocessing":null,"must_succeed":true}],"obj_validators":[[{"Primitive":0}]],"validator_exprs":[{"args":[{"User":0},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF5FNS='[]'
WF5FNMETA='[]'
#
## WFT - Group Member Remove
#WF6='{"name":"wf_group_remove_member","version":1,"activities":[null,{"code":"group_remove_member","exec_condition":null,"action":"GroupRemoveMember","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"U16":false},{"String":false}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'
#WF6FNS='[]'
#WF6FNMETA='[]'
#
## WFT - Group Remove
#WF7='{"name":"wf_group_remove","version":1,"activities":[null,{"code":"group_remove","exec_condition":null,"action":"GroupRemove","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"U16":false}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'
#WF7FNS='[]'
#WF7FNMETA='[]'
#
## WFT - Tag Add
#WF8='{"name":"wf_tag_add","version":1,"activities":[null,{"code":"tag_add","exec_condition":null,"action":"TagAdd","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"String":false},"VecString"],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'
#WF8FNS='[]'
#WF8FNMETA='[]'
#
## WFT - Tag Edit
#WF9='{"name":"wf_tag_edit","version":1,"activities":[null,{"code":"wf_tag_edit","exec_condition":null,"action":"TagRemove","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"String":false},{"U16":false},{"String":false}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'
#WF9FNS='[]'
#WF9FNMETA='[]'
#
## WFT - FT Distribute
WF10='{"name":"wf_ft_distribute","version":1,"activities":[null,{"code":"ft_distribute","exec_condition":null,"action":"FtDistribute","action_data":null,"arg_types":[{"U16":false},{"U32":false},"VecString"],"activity_inputs":[["Free","Free","Free"]],"postprocessing":null,"must_succeed":true}],"obj_validators":[[]],"validator_exprs":[],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF10FNS='[]'
WF10FNMETA='[]'
#
## WFT - Media Add
WF11='{"name":"wf_media_add","version":1,"activities":[null,{"code":"media_add","exec_condition":null,"action":"MediaAdd","action_data":null,"arg_types":[{"Object":1}],"activity_inputs":[["Free"]],"postprocessing":null,"must_succeed":true}],"obj_validators":[[]],"validator_exprs":[],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF11FNS='[]'
WF11FNMETA='[]'

## TODO find out listing_fee_near for mainnet account
## WFT - Skyward
WF12='{"name":"wf_skyward","version":1,"activities":[null,{"code":"register_tokens","exec_condition":null,"action":"FnCall","action_data":{"FnCall":{"id":["demo-skyward.petrstudynka.testnet","register_tokens"],"tgas":30,"deposit":"20000000000000000000000"}},"arg_types":[{"Object":0}],"activity_inputs":[[{"Expression":{"args":[{"Const":0},{"Bind":0}],"expr":{"Fn":"ToArray"}}}]],"postprocessing":null,"must_succeed":true},{"code":"storage_deposit","exec_condition":null,"action":"FnCall","action_data":{"FnCall":{"id":["self","storage_deposit"],"tgas":10,"deposit":"1250000000000000000000"}},"arg_types":[{"Object":0}],"activity_inputs":[[{"BindTpl":0}]],"postprocessing":null,"must_succeed":false},{"code":"storage_deposit","exec_condition":null,"action":"FnCall","action_data":{"FnCall":{"id":["wrap.testnet","storage_deposit"],"tgas":10,"deposit":"1250000000000000000000"}},"arg_types":[{"Object":0}],"activity_inputs":[[{"BindTpl":0}]],"postprocessing":null,"must_succeed":false},{"code":"ft_transfer_call","exec_condition":null,"action":"FnCall","action_data":{"FnCall":{"id":["self","ft_transfer_call"],"tgas":100,"deposit":"1"}},"arg_types":[{"Object":0}],"activity_inputs":[["Free",{"Bind":1},{"BindTpl":0},{"BindTpl":1}]],"postprocessing":null,"must_succeed":true},{"code":"sale_create","exec_condition":null,"action":"FnCall","action_data":{"FnCall":{"id":["demo-skyward.petrstudynka.testnet","sale_create"],"tgas":50,"deposit":"3000000000000000000000000"}},"arg_types":[{"Object":0}],"activity_inputs":[[{"Object":1}],[{"Bind":2},{"Bind":3},{"Const":0},{"VecObject":2},{"Bind":0},{"Bind":4},{"Bind":5}],[{"Const":0},{"Bind":1},{"BindTpl":2}]],"postprocessing":{"storage_key":"pp_5","op_type":{"FnCallResult":{"U32":false}},"instructions":[]},"must_succeed":true}],"obj_validators":[[],[],[],[],[]],"validator_exprs":[],"transitions":[[1],[2,3,4],[3,4],[4],[5]],"binds":[{"String":"demo-skyward.petrstudynka.testnet"},{"String":"\\\"AccountDeposit\\\""},"Null"],"start":[0],"end":[5]}'
WF12FNS='[["demo-skyward.petrstudynka.testnet","register_tokens"],["self","storage_deposit"],["wrap.testnet","storage_deposit"],["self","ft_transfer_call"],["demo-skyward.petrstudynka.testnet","sale_create"]]'
WF12FNMETA='[[{"arg_names":["token_account_ids"],"arg_types":["VecString"]}],[{"arg_names":["account_id"],"arg_types":[{"String":false}]}],[{"arg_names":["account_id"],"arg_types":[{"String":false}]}],[{"arg_names":["memo","amount","receiver_id","msg"],"arg_types":[{"String":true},{"U128":false},{"String":false},{"String":false}]}],[{"arg_names":["sale"],"arg_types":[{"Object":1}]},{"arg_names":["title","url","permissions_contract_id","out_tokens","in_token_account_id","start_time","duration"],"arg_types":[{"String":false},{"String":true},{"String":true},{"VecObject":2},{"String":false},{"U64":false},{"U64":false}]},{"arg_names":["token_account_id","balance","referral_bpt"],"arg_types":[{"String":false},{"U128":false},{"U16":true}]}]]'

## WFT - Bounty
WF13='{"name":"wf_bounty","version":1,"activities":[null,{"code":"event_checkin","exec_condition":null,"action":"Event","action_data":{"Event":{"code":"checkin","values":[{"String":false}],"deposit_from_bind":1}},"arg_types":[],"activity_inputs":[["Free"]],"postprocessing":{"storage_key":"pp_1","op_type":{"SaveUserValue":[0,0]},"instructions":[]},"must_succeed":true},{"code":"event_unrealized","exec_condition":null,"action":"Event","action_data":{"Event":{"code":"unrealized","values":[{"String":false}],"deposit_from_bind":null}},"arg_types":[],"activity_inputs":[["Free"]],"postprocessing":{"storage_key":"pp_2","op_type":{"RemoveActionStorage":"pp_1"},"instructions":[]},"must_succeed":true},{"code":"event_approve","exec_condition":null,"action":"Event","action_data":{"Event":{"code":"approve","values":[{"String":false},{"Bool":false}],"deposit_from_bind":null}},"arg_types":[],"activity_inputs":[["Free","Free"]],"postprocessing":{"storage_key":"pp_3","op_type":{"SaveUserValue":[0,1]},"instructions":[]},"must_succeed":true},{"code":"event_done","exec_condition":null,"action":"Event","action_data":{"Event":{"code":"done","values":[{"String":false},{"String":false}],"deposit_from_bind":null}},"arg_types":[],"activity_inputs":[["Free","Free"]],"postprocessing":{"storage_key":"pp_4","op_type":{"SaveUserValue":[0,1]},"instructions":[]},"must_succeed":true},{"code":"event_done_approve","exec_condition":null,"action":"Event","action_data":{"Event":{"code":"done_approve","values":[{"String":false},{"String":false}],"deposit_from_bind":null}},"arg_types":[],"activity_inputs":[["Free","Free"]],"postprocessing":{"storage_key":"pp_5","op_type":{"SaveUserValue":[0,1]},"instructions":[]},"must_succeed":true},{"code":"send_near","exec_condition":null,"action":"TreasurySendNear","action_data":null,"arg_types":[],"activity_inputs":[[{"Storage":"pp_1"},"Free"]],"postprocessing":null,"must_succeed":true}],"obj_validators":[[],[],[],[],[],[{"Primitive":0},{"Primitive":0}]],"validator_exprs":[{"args":[{"Bind":0},{"User":1}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"GtE"}}],"terms":[{"Arg":0},{"Arg":1}]}}},{"args":[{"Storage":"pp_1"},{"User":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":0},{"Arg":1}]}}}],"transitions":[[1],[2,3],[1],[1,2,4],[5],[6]],"binds":[],"start":[0],"end":[6]}'
WF13FNS='[]'
WF13FNMETA='[]'

## ------------ LOAD ------------ #

echo "\n\n------ Loading workflow templates to provider: ${GREEN} $WID ${NC} ------\n\n"
for ((i=$WFSTART;i<=$WFCOUNT;i++))
  do 
    wf_val="WF$i"
    wf_fns="WF"$i"FNS"
    wf_fnmeta="WF"$i"FNMETA"
    if [ ! -z "${!wf_val}" ]; then
      echo "${GREEN}------ Adding WF $i ------${NC}\n"
      near call $WID workflow_add '{"workflow": '${!wf_val}', "fncalls": '${!wf_fns}', "fncall_metadata": '${!wf_fnmeta}'}' --accountId=$WID
    fi  
 done