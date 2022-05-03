use library::{
    data::{
        basic_workflows::workflow_settings_skyward_test,
        skyward::{workflow_skyward_template_settings_data_1, SkywardTemplateUserOptions},
    },
    workflow::settings::{ProposeSettings, TemplateSettings},
};
use serde_json::json;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

use crate::{contract_utils::dao::types::proposal::ProposalCreateInput, utils::outcome_pretty};

pub(crate) async fn create_proposal<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &Contract,
    used_template_id: u16,
    proposal_settings: ProposeSettings,
    template_settings: Option<TemplateSettings>,
    deposit: u128,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let proposal_input =
        ProposalCreateInput::default(used_template_id, proposal_settings, template_settings);
    let args = serde_json::to_string(&proposal_input)
        .expect("Failed to serialize propose settings object")
        .into_bytes();
    let outcome = caller
        .call(&worker, dao.id(), "proposal_create")
        .args(args)
        .max_gas()
        .deposit(deposit)
        .transact()
        .await?;
    outcome_pretty("dao create_proposal", &outcome);
    assert!(outcome.is_success(), "dao create proposal failed");

    // TODO:
    // internal_check_proposal(worker, dao_contract, expected_proposal).await?;

    Ok(())
}

pub(crate) async fn vote_proposal<T>(
    worker: &Worker<T>,
    voters: Vec<(&Account, u8)>,
    dao: &Contract,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    for (voter, vote) in voters {
        let args = json!({
            "proposal_vote": vote,
        })
        .to_string()
        .into_bytes();
        let outcome = voter
            .call(&worker, dao.id(), "proposal_vote")
            .args(args)
            .max_gas()
            .transact()
            .await?;
        outcome_pretty("dao create_proposal", &outcome);
        assert!(outcome.is_success(), "dao vote failed");
    }

    Ok(())
}

/// Default template settings for proposal wf add skyward.
/// `id` is id of template on the provider.
pub(crate) fn ts_for_skyward(id: u8) -> TemplateSettings {
    workflow_settings_skyward_test()
}
/// Default proposal settings for proposal wf add skyward.
pub(crate) fn ps_wf_add_skyward(
    token_id: &AccountId,
    token_amount: u128,
    auction_start: u128,
    auction_duration: u128,
) -> ProposeSettings {
    workflow_skyward_template_settings_data_1(Some(SkywardTemplateUserOptions {
        token_account_id: token_id.to_string(),
        token_amount,
        auction_start,
        auction_duration,
    }))
    .1
}
