#!/bin/bash

## TODO: Automation

### Workflow template data for Workflow provider

GREEN='\033[0;32m'
NC='\033[0m'

# ------------ SETTINGS ------------ #

WFSTART=1
WFCOUNT=10
WID=$1

if [ -z $WID ]; then
    WID='workflow-provider.v1.neardao.testnet';
fi



# ------------ DATA ------------ #

# WFT - BASIC_PACKAGE1
WF1='{"code":"basic_pkg1","version":"1","auto_exec":true,"need_storage":false,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"wf_add","actions":[{"exec_condition":null,"validators":[],"action_data":{"fn_call":{"id":{"dynamic":[{"src":{"input":"provider_id"}},"wf_template"]},"tgas":30,"deposit":null,"binds":[],"must_succeed":true}},"input_source":"prop_settings","postprocessing":{"instructions":["store_workflow"]},"optional":false}],"automatic":true,"terminal":"automatic","postprocessing":null,"is_sync":false}},{"activity":{"code":"media_add","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"media_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":true,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"near_send","actions":[{"exec_condition":null,"validators":[],"action_data":{"send_near":[{"src":{"input":"receiver_id"}},{"src":{"input":"amount"}}]},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":true,"terminal":"automatic","postprocessing":null,"is_sync":false}},{"activity":{"code":"ft_send","actions":[{"exec_condition":null,"validators":[],"action_data":{"fn_call":{"id":{"standard_dynamic":[{"src":{"input":"token_id"}},"ft_transfer"]},"tgas":15,"deposit":{"value":{"u64":1}},"binds":[],"must_succeed":true}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":true,"terminal":"automatic","postprocessing":null,"is_sync":false}}],"expressions":[],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{}},"end":[1,2,3,4]}'
WF1FNS='[["workflow-provider.v1.neardao.testnet","wf_template"]]'
WF1FNMETA='[[{"arg_names":["id"],"arg_types":[{"u64":false}]}]]'
WF1STDFNS='["ft_transfer"]'

# WFT - SKYWARD1
WF2='{"code":"skyward1","version":"1","auto_exec":false,"need_storage":true,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"register_tokens","actions":[{"exec_condition":null,"validators":[],"action_data":{"fn_call":{"id":{"static":["skyward.v1.neardao.testnet","register_tokens"]},"tgas":30,"deposit":{"src":{"tpl":"deposit_register_tokens"}},"binds":[{"key":"token_account_ids","value":{"expr":{"args":[{"tpl":"account_wnear"},{"prop_settings":"offered_token"}],"expr_id":0}},"collection_data":null}],"must_succeed":true}},"input_source":"user","postprocessing":{"instructions":[{"store_value":["pp_1_result",{"bool":true}]}]},"optional":false}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":false}},{"activity":{"code":"storage_deposit","actions":[{"exec_condition":null,"validators":[],"action_data":{"fn_call":{"id":{"standard_static":["wnear.v1.neardao.testnet","storage_deposit"]},"tgas":10,"deposit":{"src":{"tpl":"deposit_storage"}},"binds":[{"key":"account_id","value":{"src":{"tpl":"account_skyward"}},"collection_data":null}],"must_succeed":false}},"input_source":"user","postprocessing":null,"optional":true},{"exec_condition":null,"validators":[],"action_data":{"fn_call":{"id":{"standard_dynamic":[{"src":{"prop_settings":"offered_token"}},"storage_deposit"]},"tgas":10,"deposit":{"src":{"tpl":"deposit_storage"}},"binds":[{"key":"account_id","value":{"src":{"tpl":"account_skyward"}},"collection_data":null}],"must_succeed":false}},"input_source":"user","postprocessing":null,"optional":true}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":false}},{"activity":{"code":"transfer_tokens","actions":[{"exec_condition":null,"validators":[],"action_data":{"fn_call":{"id":{"standard_dynamic":[{"src":{"prop_settings":"offered_token"}},"ft_transfer_call"]},"tgas":100,"deposit":{"src":{"tpl":"deposit_ft_transfer_call"}},"binds":[{"key":"receiver_id","value":{"src":{"tpl":"account_skyward"}},"collection_data":null},{"key":"amount","value":{"src":{"prop_settings":"offered_amount"}},"collection_data":null},{"key":"msg","value":{"src":{"tpl":"ft_transfer_call_msg"}},"collection_data":null},{"key":"memo","value":{"value":"null"},"collection_data":null}],"must_succeed":true}},"input_source":"user","postprocessing":{"instructions":[{"store_value":["pp_3_result",{"bool":true}]}]},"optional":false}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":false}},{"activity":{"code":"sale_create","actions":[{"exec_condition":null,"validators":[],"action_data":{"fn_call":{"id":{"static":["skyward.v1.neardao.testnet","sale_create"]},"tgas":50,"deposit":{"src":{"tpl":"deposit_sale_create"}},"binds":[{"key":"sale.permissions_contract_id","value":{"src":{"runtime":0}},"collection_data":null},{"key":"token_account_id","value":{"src":{"prop_settings":"offered_token"}},"collection_data":{"prefixes":["sale.out_tokens"],"collection_binding_type":{"force_same":1}}},{"key":"balance","value":{"src":{"prop_settings":"offered_amount"}},"collection_data":{"prefixes":["sale.out_tokens"],"collection_binding_type":{"force_same":1}}},{"key":"referral_bpt","value":{"value":"null"},"collection_data":{"prefixes":["sale.out_tokens"],"collection_binding_type":{"force_same":1}}},{"key":"sale.in_token_account_id","value":{"src":{"tpl":"account_wnear"}},"collection_data":null},{"key":"sale.start_time","value":{"src":{"action":"start_time"}},"collection_data":null},{"key":"sale.duration","value":{"src":{"action":"duration"}},"collection_data":null}],"must_succeed":true}},"input_source":"user","postprocessing":{"instructions":[{"store_fn_call_result":["skyward_auction_id",{"datatype":{"u64":false}}]}]},"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":false}}],"expressions":[{"fn":"array_merge"},{"boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"rel":"eqs"}}],"terms":[{"arg":0},{"value":{"bool":true}}]}}],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":2,"cond":{"expr":{"args":[{"storage":"pp_1_result"}],"expr_id":1}},"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":{"expr":{"args":[{"storage":"pp_1_result"}],"expr_id":1}},"time_from_cond":null,"time_to_cond":null}],[{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":4,"cond":{"expr":{"args":[{"storage":"pp_3_result"}],"expr_id":1}},"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{"deposit_sale_create":{"u128":"3000000000000000000000000"},"account_wnear":{"string":"wnear.v1.neardao.testnet"},"deposit_ft_transfer_call":{"u128":"1"},"deposit_storage":{"u128":"1250000000000000000000"},"ft_transfer_call_msg":{"string":"\\\"AccountDeposit\\\""},"account_skyward":{"string":"skyward.v1.neardao.testnet"},"deposit_register_tokens":{"u128":"20000000000000000000000"}}},"end":[4]}'
WF2FNS='[["skyward.v1.neardao.testnet","register_tokens"],["skyward.v1.neardao.testnet","sale_create"]]'
WF2FNMETA='[[{"arg_names":["token_account_ids"],"arg_types":["vec_string"]}],[{"arg_names":["sale"],"arg_types":[{"object":1}]},{"arg_names":["title","url","permissions_contract_id","out_tokens","in_token_account_id","start_time","duration"],"arg_types":[{"string":false},{"string":true},{"string":true},{"vec_object":2},{"string":false},{"u128":false},{"u128":false}]},{"arg_names":["token_account_id","balance","referral_bpt"],"arg_types":[{"string":false},{"u128":false},{"u64":true}]}]]'
WF2STDFNS='["storage_deposit","ft_transfer_call"]'

# WFT - BOUNTY1
WF3='{"code":"bounty1","version":"1","auto_exec":false,"need_storage":true,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"event_checkin","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"event","code":"event_checkin","expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":{"instructions":[{"store_dyn_value":["account_id_applied",{"src":{"runtime":2}}]}]},"optional":false}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":true}},{"activity":{"code":"event_unrealized","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"event","code":"event_unrealized","expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":{"instructions":[{"delete_key":"account_id_applied"}]},"optional":false}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":true}},{"activity":{"code":"event_approve","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"event","code":"event_approve","expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":{"instructions":[{"store_dyn_value":["approved_by",{"src":{"runtime":2}}]},{"store_dyn_value":["checkin_accepted",{"src":{"input":"checkin_accepted"}}]}]},"optional":false}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":true}},{"activity":{"code":"event_done","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"event","code":"event_done","expected_input":[["result",{"string":false}]],"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":{"instructions":[{"store_dyn_value":["event_done_result",{"src":{"input":"result"}}]}]},"optional":false}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":true}},{"activity":{"code":"event_done_approve","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"event","code":"event_done_approve","expected_input":[["result_evaluation",{"string":false}]],"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":{"instructions":[{"store_dyn_value":["event_done_approved_by",{"src":{"runtime":2}}]},{"store_dyn_value":["event_done_result_evaluation",{"src":{"input":"result_evaluation"}}]}]},"optional":false}],"automatic":true,"terminal":"non_terminal","postprocessing":null,"is_sync":true}},{"activity":{"code":"send_near","actions":[{"exec_condition":null,"validators":[{"object":{"expression_id":0,"value":[{"src":{"input":"amount_near"}},{"src":{"prop_settings":"max_offered_near_amount"}}]}},{"object":{"expression_id":1,"value":[{"src":{"input":"receiver_id"}},{"src":{"storage":"account_id_applied"}}]}}],"action_data":{"send_near":[{"src":{"storage":"account_id_applied"}},{"src":{"input":"amount_near"}}]},"input_source":"user","postprocessing":null,"optional":false}],"automatic":true,"terminal":"automatic","postprocessing":null,"is_sync":false}}],"expressions":[{"boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"rel":"gt_e"}}],"terms":[{"arg":0},{"arg":1}]}},{"boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"rel":"eqs"}}],"terms":[{"arg":0},{"arg":1}]}},{"boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"rel":"eqs"}},{"operands_ids":[2,3],"op_type":{"rel":"eqs"}},{"operands_ids":[0,1],"op_type":{"log":"and"}}],"terms":[{"arg":0},{"value":{"bool":true}},{"arg":1},{"arg":2}]}},{"boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"rel":"eqs"}}],"terms":[{"arg":0},{"value":{"bool":false}}]}},{"boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"rel":"eqs"}}],"terms":[{"arg":0},{"arg":1}]}}],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":{"expr":{"args":[{"storage":"checkin_accepted"}],"expr_id":3}},"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":{"expr":{"args":[{"storage":"account_id_applied"},{"runtime":2}],"expr_id":1}},"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":{"expr":{"args":[{"storage":"checkin_accepted"},{"storage":"account_id_applied"},{"runtime":2}],"expr_id":2}},"time_from_cond":null,"time_to_cond":null}],[{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{}},"end":[6]}'
WF3FNS='[]'
WF3FNMETA='[]'
WF3STDFNS='[]'

# WFT - REWARD1
WF4='{"code":"reward1","version":"1","auto_exec":false,"need_storage":false,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"treasury_add_partition","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"treasury_add_partition","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"non_terminal","postprocessing":null,"is_sync":true}},{"activity":{"code":"reward_add_wage","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"reward_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"reward_add_user_activity","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"reward_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}}],"expressions":[],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{}},"end":[2,3]}'
WF4FNS='[]'
WF4FNMETA='[]'
WF4STDFNS='[]'

# WFT - TRADE1
WF5='{"code":"trade1","version":"1","auto_exec":false,"need_storage":true,"receiver_storage_keys":[{"id":"trade","sender_id":"sender_id","token_id":"token_id","amount":"received_token_amount"}],"activities":["init",{"activity":{"code":"send_near","actions":[{"exec_condition":{"expr":{"args":[{"prop_settings":"required_token_id"},{"storage":"token_id"},{"prop_settings":"required_token_amount"},{"storage":"received_token_amount"}],"expr_id":0}},"validators":[],"action_data":{"send_near":[{"src":{"storage":"sender_id"}},{"src":{"prop_settings":"offered_near_amount"}}]},"input_source":"user","postprocessing":null,"optional":false}],"automatic":true,"terminal":"automatic","postprocessing":null,"is_sync":false}}],"expressions":[{"boolean":{"operators":[{"operands_ids":[0,1],"op_type":{"rel":"eqs"}},{"operands_ids":[2,3],"op_type":{"rel":"eqs"}},{"operands_ids":[0,1],"op_type":{"log":"and"}}],"terms":[{"arg":0},{"arg":1},{"arg":2},{"arg":3}]}}],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{}},"end":[1]}'
WF5FNS='[]'
WF5FNMETA='[]'
WF5STDFNS='[]'

# WFT - Media1
WF6='{"code":"media1","version":"1","auto_exec":false,"need_storage":false,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"media_add","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"media_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}},{"activity":{"code":"media_update","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"media_update","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}}],"expressions":[],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null}],[]],"constants":{"map":{}},"end":[1,2]}'
WF6FNS='[]'
WF6FNMETA='[]'
WF6STDFNS='[]'

# WFT - Lock1
WF7='{"code":"lock1","version":"1","auto_exec":false,"need_storage":false,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"treasury_add_partition","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"treasury_add_partition","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"partition_add_asset","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"partition_add_asset_amount","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}}],"expressions":[],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null}],[],[{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{}},"end":[1,2,3]}'
WF7FNS='[]'
WF7FNMETA='[]'
WF7STDFNS='[]'

# WFT - Group1
WF8='{"code":"group1","version":"1","auto_exec":false,"need_storage":false,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"group_add","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_add_members","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_add_members","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove_members","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove_members","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove_roles","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove_roles","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove_member_roles","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove_member_roles","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}}],"expressions":[],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}],[],[],[],[],[]],"constants":{"map":{}},"end":[1,2,3,4,5,6]}'
WF8FNS='[]'
WF8FNMETA='[]'
WF8STDFNS='[]'

# WFT - GroupPackage1
WF9='{"code":"group_package1","version":"1","auto_exec":false,"need_storage":false,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"group_add","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_add_members","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_add_members","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove_members","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove_members","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove_roles","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove_roles","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}},{"activity":{"code":"group_remove_member_roles","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"group_remove_member_roles","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"user","postprocessing":null,"optional":false}],"automatic":false,"terminal":"user","postprocessing":null,"is_sync":true}}],"expressions":[],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}],[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":4,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":5,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":6,"cond":null,"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{}},"end":[1,2,3,4,5,6]}'
WF9FNS='[]'
WF9FNMETA='[]'
WF9STDFNS='[]'

# WFT - Reward2
WF10='{"code":"reward2","version":"1","auto_exec":false,"need_storage":false,"receiver_storage_keys":[],"activities":["init",{"activity":{"code":"reward_add_wage","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"reward_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"reward_add_user_activity","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"reward_add","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}},{"activity":{"code":"reward_update","actions":[{"exec_condition":null,"validators":[],"action_data":{"action":{"name":"reward_update","code":null,"expected_input":null,"required_deposit":null,"binds":[]}},"input_source":"prop_settings","postprocessing":null,"optional":false}],"automatic":false,"terminal":"automatic","postprocessing":null,"is_sync":true}}],"expressions":[],"transitions":[[{"activity_id":1,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":2,"cond":null,"time_from_cond":null,"time_to_cond":null},{"activity_id":3,"cond":null,"time_from_cond":null,"time_to_cond":null}]],"constants":{"map":{}},"end":[1,2,3]}'
WF10FNS='[]'
WF10FNMETA='[]'
WF10STDFNS='[]'

## ------------ LOAD ------------ #

echo "\n\n------ Loading workflow templates to provider: ${GREEN} $WID ${NC} ------\n\n"
for ((i=$WFSTART;i<$WFSTART+$WFCOUNT;i++))
  do 
    wf_val="WF$i"
    wf_fns="WF"$i"FNS"
    wf_std_fns="WF"$i"STDFNS"
    wf_fnmeta="WF"$i"FNMETA"
    if [ ! -z "${!wf_val}" ]; then
      echo "${GREEN}------ Adding WF $i ------${NC}\n"
      near call $WID workflow_add '{"workflow": '${!wf_val}', "fncalls": '${!wf_fns}',"standard_fncalls": '${!wf_std_fns}', "fncall_metadata": '${!wf_fnmeta}'}' --accountId=$WID
    fi
 done

# STANDARD_FN_CALLS='{"fncalls":["ft_transfer","ft_transfer_call","nft_transfer","nft_transfer_call","storage_deposit","storage_withdraw","storage_unregister"],"fncall_metadata":[[{"arg_names":["receiver_id","amount","memo"],"arg_types":[{"string":false},{"u128":false},{"string":true}]}],[{"arg_names":["receiver_id","amount","memo","msg"],"arg_types":[{"string":false},{"u128":false},{"string":true},{"string":false}]}],[{"arg_names":["receiver_id","token_id","approval_id","memo"],"arg_types":[{"string":false},{"string":false},{"u64":true},{"string":true}]}],[{"arg_names":["receiver_id","token_id","approval_id","memo","msg"],"arg_types":[{"string":false},{"string":false},{"u64":true},{"string":true},{"string":false}]}],[{"arg_names":["account_id","registration_only"],"arg_types":[{"string":false},{"bool":true}]}],[{"arg_names":["amount"],"arg_types":[{"u128":true}]}],[{"arg_names":["force"],"arg_types":[{"string":false}]}]]}'
# WF_BASIC_PKG_TEMPLATE_SETTINGS='{"allowed_proposers":[{"group":1}],"allowed_voters":{"group":1},"activity_rights":[[],[{"group":1}],[{"group":1}],[{"group":1}],[{"group":1}]],"transition_limits":[[{"to":1,"limit":1},{"to":2,"limit":1},{"to":3,"limit":1},{"to":4,"limit":1}]],"scenario":"democratic","duration":60,"quorum":51,"approve_threshold":20,"spam_threshold":80,"vote_only_once":true,"deposit_propose":"1000000000000000000000000","deposit_vote":"1","deposit_propose_return":0,"constants":null}'
# near call workflow-provider.$CID wf_basic_package_add_settings '{"settings": '$WF_BASIC_PKG_TEMPLATE_SETTINGS'}' --accountId=workflow-provider.$CID
# near call workflow-provider.$CID standard_fncalls_add $STANDARD_FN_CALLS --accountId=workflow-provider.$CID