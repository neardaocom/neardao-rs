use library::workflow::instance::Instance;
use serde_json::json;
use workspaces::{Contract, DevNetwork, Worker};

use crate::utils::{parse_view_result, view_outcome_pretty};

use super::types::{
    consts::DAO_VIEW_INSTANCE,
    proposal::{Proposal, ViewInstance, ViewProposal, Votes},
};

pub(crate) async fn proposal<T>(
    worker: &Worker<T>,
    dao: &Contract,
    id: u32,
) -> anyhow::Result<ViewProposal>
where
    T: DevNetwork,
{
    let args = json!({ "id": id }).to_string().into_bytes();
    let outcome = dao.view(&worker, "proposal", args).await?;
    view_outcome_pretty::<ViewProposal>("dao view proposal", &outcome);
    let proposal = parse_view_result::<ViewProposal>(&outcome).unwrap_or_default();
    Ok(proposal)
}

pub(crate) async fn votes<T>(worker: &Worker<T>, dao: &Contract, id: u32) -> anyhow::Result<Votes>
where
    T: DevNetwork,
{
    let proposal = proposal(worker, dao, id)
        .await?
        .expect("failed to get proposal");
    let votes = Proposal::from(proposal.0).votes;
    Ok(votes)
}

pub(crate) async fn workflow_instance<T>(
    worker: &Worker<T>,
    dao: &Contract,
    proposal_id: u32,
) -> anyhow::Result<Option<Instance>>
where
    T: DevNetwork,
{
    let args = json!({ "proposal_id": proposal_id })
        .to_string()
        .into_bytes();
    let outcome = dao.view(&worker, DAO_VIEW_INSTANCE, args).await?;
    view_outcome_pretty::<ViewInstance>("dao view instance", &outcome);
    let instance = parse_view_result::<ViewInstance>(&outcome)
        .flatten()
        .map(|(i, _)| i);
    Ok(instance)
}
