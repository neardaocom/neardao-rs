#!/bin/bash

### Workflow template data for Workflow provider

GREEN='\033[0;32m'
NC='\033[0m'

# ------------ SETTINGS ------------ #

WFSTART=1
WFCOUNT=12
WID=$1

if [ -z $WID ]; then
    WID='wf-provider.neardao.testnet';
fi

# ------------ DATA ------------ #

# WFT - Workflow Add
WF1='{"name":"wf_add","version":1,"activities":[null,{"code":"wf_add","exec_condition":null,"action":"WorkflowAdd","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"U16":false}],"activity_inputs":[[{"Bind":0}]],"postprocessing":null}],"obj_validators":[[{"Primitive":0}]],"validator_exprs":[{"args":[{"User":0},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF1FNS='[]'
WF1FNMETA='[]'

# WFT - Near Send
WF2='{"name":"wf_near_send","version":1,"activities":[null,{"code":"near_send","exec_condition":null,"action":"TreasurySendNear","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"String":false},{"U128":false}],"activity_inputs":[["Free","Free"]],"postprocessing":null}],"obj_validators":[[{"Primitive":0},{"Primitive":0}]],"validator_exprs":[{"args":[{"User":0},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}},{"args":[{"User":1},{"Bind":1}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"GtE"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF2FNS='[]'
WF2FNMETA='[]'

## WFT - FT Send
WF3='{"name":"wf_treasury_send_ft","version":1,"activities":[null,{"code":"treasury_send_ft","exec_condition":null,"action":"TreasurySendFt","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"String":false},{"String":false},{"Bool":false},{"U128":false},{"String":true},{"String":false}],"activity_inputs":[["Free","Free","Free","Free","Free","Free"]],"postprocessing":null}],"obj_validators":[[]],"validator_exprs":[],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF3FNS='[]'
WF3FNMETA='[]'
#
## WFT - Group Add
#WF4='{"name":"wf_group_add","version":1,"activities":[null,{"code":"add_group","exec_condition":null,"action":"GroupAdd","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"Object":1},{"VecObject":2},{"Object":3}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'
#WF4FNS='[]'
#WF4FNMETA='[]'
#
# WFT - Group Members Add
WF5='{"name":"wf_group_add_members","version":1,"activities":[null,{"code":"group_add_members","exec_condition":null,"action":"GroupAddMembers","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"U16":false},{"VecObject":1}],"activity_inputs":[["Free","Free"]],"postprocessing":null}],"obj_validators":[[{"Primitive":0}]],"validator_exprs":[{"args":[{"User":0},{"Bind":0}],"expr":{"Boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"Rel":"Eqs"}}],"terms":[{"Arg":1},{"Arg":0}]}}}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
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
WF10='{"name":"wf_ft_distribute","version":1,"activities":[null,{"code":"ft_distribute","exec_condition":null,"action":"FtDistribute","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"U16":false},{"U32":false},"VecString"],"activity_inputs":[["Free","Free","Free"]],"postprocessing":null}],"obj_validators":[[]],"validator_exprs":[],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF10FNS='[]'
WF10FNMETA='[]'
#
## WFT - Media Add
WF11='{"name":"wf_media_add","version":1,"activities":[null,{"code":"media_add","exec_condition":null,"action":"MediaAdd","fncall_id":null,"tgas":0,"deposit":"0","arg_types":[{"Object":1}],"activity_inputs":[["Free"]],"postprocessing":null}],"obj_validators":[[]],"validator_exprs":[],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF11FNS='[]'
WF11FNMETA='[]'

## TODO find out listing_fee_near for mainnet account
## WFT - Skyward
WF12='{"name":"wf_skyward","version":1,"activities":[null,{"code":"register_tokens","exec_condition":null,"action":"FnCall","fncall_id":["demo-skyward.petrstudynka.testnet","register_tokens"],"tgas":10,"deposit":"20000000000000000000000","arg_types":[{"Object":0}],"activity_inputs":[[{"Expression":{"args":[{"Const":0},{"User":0}],"expr":{"Fn":"ArrayMerge"}}}]],"postprocessing":{"storage_key":"pp_1","op_type":{"SaveUserValue":[0,0]},"instructions":[]}},{"code":"storage_deposit","exec_condition":null,"action":"FnCall","fncall_id":["self","storage_deposit"],"tgas":10,"deposit":"1250000000000000000000","arg_types":[{"Object":0}],"activity_inputs":[[{"Bind":0}]],"postprocessing":{"storage_key":"pp_2","op_type":{"SaveValue":{"Bool":true}},"instructions":[]}},{"code":"storage_deposit","exec_condition":null,"action":"FnCall","fncall_id":["wrap.testnet","storage_deposit"],"tgas":10,"deposit":"1250000000000000000000","arg_types":[{"Object":0}],"activity_inputs":[[{"Bind":0}]],"postprocessing":{"storage_key":"pp_3","op_type":{"SaveValue":{"Bool":true}},"instructions":[]}},{"code":"ft_transfer_call","exec_condition":null,"action":"FnCall","fncall_id":["self","ft_transfer_call"],"tgas":60,"deposit":"1","arg_types":[{"Object":0}],"activity_inputs":[["Free",{"Bind":2},{"Bind":0},{"Bind":1}]],"postprocessing":{"storage_key":"pp_4","op_type":{"SaveBind":2},"instructions":[]}},{"code":"sale_create","exec_condition":null,"action":"FnCall","fncall_id":["demo-skyward.petrstudynka.testnet","sale_create"],"tgas":50,"deposit":"3000000000000000000000000","arg_types":[{"Object":0}],"activity_inputs":[[{"Object":1}],["Free","Free",{"Const":0},{"VecObject":2},"Free","Free","Free"],[{"Const":0},{"Storage":"pp_4"},"Free"]],"postprocessing":null}],"obj_validators":[[],[],[],[],[]],"validator_exprs":[],"transitions":[[1],[2,3,4],[3,4],[4],[5]],"binds":[],"start":[0],"end":[5]}'
WF12FNS='[["demo-skyward.petrstudynka.testnet","register_tokens"],["self","storage_deposit"],["wrap.testnet","storage_deposit"],["self","ft_transfer_call"],["demo-skyward.petrstudynka.testnet","sale_create"]]'
WF12FNMETA='[[{"arg_names":["token_account_ids"],"arg_types":["VecString"]}],[{"arg_names":["account_id"],"arg_types":[{"String":false}]}],[{"arg_names":["account_id"],"arg_types":[{"String":false}]}],[{"arg_names":["memo","amount","receiver_id","msg"],"arg_types":[{"String":true},{"U128":false},{"String":false},{"String":false}]}],[{"arg_names":["sale"],"arg_types":[{"Object":1}]},{"arg_names":["title","url","permissions_contract_id","out_tokens","in_token_account_id","start_time","duration"],"arg_types":[{"String":false},{"String":true},{"String":true},{"VecObject":2},{"String":false},{"U64":false},{"U64":false}]},{"arg_names":["token_account_id","balance","referral_bpt"],"arg_types":[{"String":false},{"U128":false},{"U16":true}]}]]'

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