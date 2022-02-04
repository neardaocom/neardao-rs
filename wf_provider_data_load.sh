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
WF1='{"name":"wf_add","version":1,"activities":[null,{"exec_condition":null,"action":"WorkflowAdd","fncall_id":null,"gas":0,"deposit":0,"arg_types":[{"U16":false},{"Object":0}],"postprocessing":null}],"transitions":[[1]],"binds":[],"start":[0],"end":[1]}'

# WFT - Near Send
WF2='{"name":"payout","version":1,"activities":[null,{"exec_condition":null,"action":"NearSend","fncall_id":null,"gas":0,"deposit":0,"arg_types":[{"U16":false},{"Object":0}],"postprocessing":null}],"transitions":[[1],[1]],"binds":[],"start":[0],"end":[1]}'

# ------------ LOAD ------------ #

echo "\n\n------ Loading workflow templates to provider: ${GREEN} $WID ${NC} ------\n\n"
for ((i=$WFSTART;i<=$WFCOUNT;i++))
  do 
    args="WF$i"
    echo "${GREEN}------ Adding WF $i ------${NC}\n"
    near call $WID workflow_add '{"workflow": '${!args}'}' --accountId=$WID
 done