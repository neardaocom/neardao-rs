
# TODO: Move to proposal settings
S_TEMPLATE_SETTINGS='{"allowed_proposers":[{"Group":1}],"allowed_voters": {"Group":1},"scenario":"Democratic","duration":60,"quorum":51,"approve_threshold":20,"spam_threshold":80,"vote_only_once":true,"deposit_propose":1,"deposit_vote":1000,"deposit_propose_return":0}'

near call $DCID workflow_add '{"proposal_id":1,"workflow_id":2,"workflow_settings":['$S_TEMPLATE_SETTINGS']}' --accountId $CID1 --gas $MAX_GAS