#!/bin/bash

### Workflow template data for Workflow provider

## call example: sh wf_provider_data_load.sh $WID

GREEN='\033[0;32m'
NC='\033[0m'

# ------------ SETTINGS ------------ #

WFSTART=1
WFCOUNT=2
WID=$1

if [ -z $WID ]; then
    WID='wf-provider.neardao.testnet';
fi

# ------------ DATA ------------ #

# WFT - Workflow Add
WF1='{"name":"wf_add","version":1,"activities":[null,{"code":"wf_add","exec_condition":null,"action":"WorkflowAdd","fncall_id":null,"tgas":0,"deposit":0,"arg_types":[{"U16":false},{"Object":0}],"postprocessing":null}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'
WF1FNS='[]'
WF1FNMETA='[]'

# WFT - Near Send
WF2='{"name":"wf_near_send","version":1,"activities":[null,{"code":"near_send","exec_condition":null,"action":"NearSend","fncall_id":null,"tgas":null,"deposit":null,"arg_types":[{"String":false},{"U128":false}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'
WF2FNS='[]'
WF2FNMETA='[]'
# ------------ LOAD ------------ #

echo "\n\n------ Loading workflow templates to provider: ${GREEN} $WID ${NC} ------\n\n"
for ((i=$WFSTART;i<=$WFCOUNT;i++))
  do 
    wf_val="WF$i"
    wf_fns="WF"$i"FNS"
    wf_fnmeta="WF"$i"FNMETA"
    echo "${GREEN}------ Adding WF $i ------${NC}\n"
    near call $WID workflow_add '{"workflow": '${!wf_val}', "fncalls": '${!wf_fns}', "fncall_metadata": '${!wf_fnmeta}'}' --accountId=$WID
 done