use library::workflow::instance::Instance;
use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{AccountId, Contract, DevNetwork, Worker};

use crate::utils::{parse_view_result, view_outcome_pretty};

use super::types::{
    consts::{DAO_VIEW_INSTANCE, DAO_VIEW_TEMPLATES, DAO_VIEW_WORKFLOW_STORAGE},
    proposal::{Proposal, Votes},
    reward::{Reward, Wallet},
    view::{UserRoles, ViewInstance, ViewProposal, ViewTemplates, ViewWorkflowStorage},
};

pub(crate) async fn proposal<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    id: u32,
) -> anyhow::Result<ViewProposal>
where
    T: DevNetwork,
{
    let args = json!({ "id": id }).to_string().into_bytes();
    let outcome = worker.view(dao, "proposal", args).await?;
    view_outcome_pretty::<ViewProposal>("dao view proposal", &outcome);
    let proposal = parse_view_result::<ViewProposal>(&outcome).unwrap_or_default();
    Ok(proposal)
}

pub(crate) async fn votes<T>(worker: &Worker<T>, dao: &AccountId, id: u32) -> anyhow::Result<Votes>
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
    dao: &AccountId,
    proposal_id: u32,
) -> anyhow::Result<Option<Instance>>
where
    T: DevNetwork,
{
    let args = json!({ "proposal_id": proposal_id })
        .to_string()
        .into_bytes();
    let outcome = worker.view(dao, DAO_VIEW_INSTANCE, args).await?;
    view_outcome_pretty::<ViewInstance>("dao view instance", &outcome);
    let instance = parse_view_result::<ViewInstance>(&outcome).flatten();
    Ok(instance)
}

pub(crate) async fn workflow_templates<T>(
    worker: &Worker<T>,
    dao: &AccountId,
) -> anyhow::Result<ViewTemplates>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(dao, DAO_VIEW_TEMPLATES, args).await?;
    view_outcome_pretty::<ViewTemplates>("dao view templates", &outcome);
    let instance =
        parse_view_result::<ViewTemplates>(&outcome).expect("failed to parse workflow templates");
    Ok(instance)
}

pub(crate) async fn workflow_storage<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    workflow_storage_key: String,
) -> anyhow::Result<ViewWorkflowStorage>
where
    T: DevNetwork,
{
    let args = json!({ "bucket_id": workflow_storage_key })
        .to_string()
        .into_bytes();
    let outcome = worker.view(dao, DAO_VIEW_WORKFLOW_STORAGE, args).await?;
    view_outcome_pretty::<ViewWorkflowStorage>("dao view workflow storage", &outcome);
    let storage = parse_view_result::<ViewWorkflowStorage>(&outcome)
        .expect("failed to parse workflow storage");
    Ok(storage)
}

pub(crate) async fn ft_balance_of<T>(
    worker: &Worker<T>,
    ft_contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<U128>
where
    T: DevNetwork,
{
    let args = json!({ "account_id": account_id.to_string() })
        .to_string()
        .into_bytes();
    let outcome = ft_contract.view(&worker, "ft_balance_of", args).await?;
    let title = format!(
        "view ft_balance_of account: {} on contract: {}",
        account_id.as_str(),
        ft_contract.id().as_str(),
    );
    view_outcome_pretty::<U128>(&title, &outcome);
    let amount = parse_view_result::<U128>(&outcome).expect("failed to parse ft_balance_of amount");
    Ok(amount)
}

pub(crate) async fn storage_balance_of<T, U>(
    worker: &Worker<T>,
    contract: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<U>
where
    T: DevNetwork,
    U: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    let args = json!({ "account_id": account_id.to_string() })
        .to_string()
        .into_bytes();
    let outcome = contract.view(&worker, "storage_balance_of", args).await?;
    let title = format!(
        "view storage_balance_of account: {} on contract: {}",
        account_id.as_str(),
        contract.id().as_str(),
    );
    view_outcome_pretty::<U>(&title, &outcome);
    let amount =
        parse_view_result::<U>(&outcome).expect("failed to parse storage_balance_of amount");
    Ok(amount)
}

pub(crate) async fn storage_minimum_balance<T>(
    worker: &Worker<T>,
    contract: &Contract,
) -> anyhow::Result<U128>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = contract
        .view(&worker, "storage_minimum_balance", args)
        .await?;
    let title = format!(
        "view storage_minimum_balance on contract: {}",
        contract.id().as_str(),
    );
    view_outcome_pretty::<U128>(&title, &outcome);
    let amount = parse_view_result::<U128>(&outcome)
        .expect("failed to parse storage_minimum_balance amount");
    Ok(amount)
}

pub(crate) async fn debug_log<T>(worker: &Worker<T>, dao: &AccountId) -> anyhow::Result<Vec<String>>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(&dao, "debug_log", args).await?;
    let title = format!("view debug log on dao: {}", dao.as_str(),);
    view_outcome_pretty::<Vec<String>>(&title, &outcome);
    let logs = parse_view_result::<Vec<String>>(&outcome).expect("failed to parse debug log");
    Ok(logs)
}

pub(crate) async fn get_timestamp<T>(worker: &Worker<T>, dao: &AccountId) -> anyhow::Result<u64>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(&dao, "current_timestamp", args).await?;
    view_outcome_pretty::<u64>("view current_timestamp", &outcome);
    let timestamp = parse_view_result::<u64>(&outcome).expect("failed to parse current_timestamp");
    Ok(timestamp)
}

pub(crate) async fn view_reward<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    reward_id: u16,
) -> anyhow::Result<Reward>
where
    T: DevNetwork,
{
    let args = json!({ "reward_id": reward_id }).to_string().into_bytes();
    let outcome = worker.view(&dao, "view_reward", args).await?;
    view_outcome_pretty::<Reward>("view reward", &outcome);
    let reward = parse_view_result::<Reward>(&outcome).expect("failed to parse reward_id");
    Ok(reward)
}

pub(crate) async fn view_user_wallet<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    account_id: &AccountId,
) -> anyhow::Result<Wallet>
where
    T: DevNetwork,
{
    let args = json!({
        "account_id": account_id.to_string()
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&dao, "view_wallet", args).await?;
    view_outcome_pretty::<Wallet>("view user wallet", &outcome);
    let wallet = parse_view_result::<Wallet>(&outcome).expect("failed to parse wallet");
    Ok(wallet)
}

pub(crate) async fn view_user_roles<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    account_id: &AccountId,
) -> anyhow::Result<UserRoles>
where
    T: DevNetwork,
{
    let args = json!({
        "account_id": account_id.to_string()
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&dao, "view_user_roles", args).await?;
    view_outcome_pretty::<UserRoles>("view user roles", &outcome);
    let roles = parse_view_result::<UserRoles>(&outcome).expect("failed to parse user roles");
    Ok(roles)
}

/* pub(crate) async fn view_partition_list<T>(
    worker: &Worker<T>,
    dao: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<Vec<(u16, TreasuryPartition)>>
where
    T: DevNetwork,
{
    let args = json!({
        "account_id": account_id.to_string()
    })
    .to_string()
    .into_bytes();
    let outcome = dao.view(&worker, "view_partition_list", args).await?;
    view_outcome_pretty::<Vec<(u16, u16)>>("view partition list", &outcome);
    let roles = parse_view_result::<Vec<(u16, u16)>>(&outcome).expect("failed to parse partition list");
    Ok(roles)
} */
