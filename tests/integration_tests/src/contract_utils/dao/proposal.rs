use library::{
    data::workflows::integration::skyward::{Skyward1, Skyward1ProposeOptions},
    workflow::settings::{ProposeSettings, TemplateSettings},
};
use serde_json::json;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

use crate::{
    contract_utils::dao::{
        types::proposal::{ProposalCreateInput, ProposalState},
        view::{proposal, votes},
    },
    utils::outcome_pretty,
};

use super::types::proposal::Proposal;

pub(crate) async fn create_proposal<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &Contract,
    used_template_id: u16,
    proposal_settings: ProposeSettings,
    template_settings: Option<TemplateSettings>,
    deposit: u128,
) -> anyhow::Result<u32>
where
    T: DevNetwork,
{
    let proposal_input =
        ProposalCreateInput::default(used_template_id, proposal_settings, template_settings);
    let args = serde_json::to_string(&proposal_input)
        .expect("failed to serialize propose settings object")
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
    let proposal_id: u32 = outcome.json().expect("failed to parse proposal_id.");
    let proposal = proposal(worker, dao, proposal_id).await?;
    assert!(proposal.is_some());

    Ok(proposal_id)
}

pub(crate) async fn vote_proposal<T>(
    worker: &Worker<T>,
    mut voters: Vec<(&Account, u8)>,
    dao: &Contract,
    proposal_id: u32,
    deposit: u128,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    voters.sort_by(|(a, _), (b, _)| a.id().cmp(b.id()));
    voters.dedup_by(|(a, _), (b, _)| a.id() == b.id());
    let voters_len = voters.len();
    for (voter, vote) in voters {
        let args = json!({
            "id": proposal_id,
            "vote": vote,
        })
        .to_string()
        .into_bytes();
        let outcome = voter
            .call(&worker, dao.id(), "proposal_vote")
            .args(args)
            .max_gas()
            .deposit(deposit)
            .transact()
            .await?;
        outcome_pretty("dao vote_proposal", &outcome);
        assert!(outcome.is_success(), "dao vote failed");
    }
    let actual_votes = votes(worker, dao, proposal_id).await?;
    assert_eq!(actual_votes.len(), voters_len, "vote count does not equal");

    Ok(())
}

pub(crate) async fn finish_proposal<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &Contract,
    id: u32,
    expected_state: ProposalState,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({
        "id": id,
    })
    .to_string()
    .into_bytes();
    let outcome = caller
        .call(&worker, dao.id(), "proposal_finish")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty("dao finish_proposal", &outcome);
    assert!(outcome.is_success(), "dao finish_proposal failed");

    let proposal = proposal(worker, dao, id)
        .await?
        .expect("failed to get proposal");
    let state = Proposal::from(proposal.0).state;
    assert_eq!(
        state, expected_state,
        "actual proposal state is different from expected"
    );

    Ok(())
}
/// Default proposal settings for proposal wf add skyward.
pub(crate) fn ps_skyward(
    token_id: &AccountId,
    token_amount: u128,
    auction_start: u128,
    auction_duration: u128,
    storage_key: Option<&str>,
) -> ProposeSettings {
    Skyward1::propose_settings(
        Some(Skyward1ProposeOptions {
            token_account_id: token_id.to_string(),
            token_amount,
            auction_start,
            auction_duration,
        }),
        storage_key,
    )
}
