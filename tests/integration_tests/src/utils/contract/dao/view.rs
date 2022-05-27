use library::workflow::instance::Instance;
use serde_json::json;
use workspaces::{AccountId, DevNetwork, Worker};

use crate::types::{ActionLog, Media};
use crate::utils::{parse_view_result, view_outcome_pretty};

use crate::constants::{DAO_VIEW_INSTANCE, DAO_VIEW_TEMPLATES, DAO_VIEW_WORKFLOW_STORAGE};
use crate::types::{
    TreasuryPartition, {Group, Roles, UserRoles}, {Proposal, Votes}, {Reward, Wallet},
    {Statistics, ViewInstance, ViewProposal, ViewTemplates, ViewWorkflowStorage},
};

pub async fn proposal<T>(
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

pub async fn votes<T>(worker: &Worker<T>, dao: &AccountId, id: u32) -> anyhow::Result<Votes>
where
    T: DevNetwork,
{
    let proposal = proposal(worker, dao, id)
        .await?
        .expect("failed to get proposal");
    let votes = Proposal::from(proposal.0).votes;
    Ok(votes)
}

pub async fn workflow_instance<T>(
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

pub async fn workflow_templates<T>(
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

pub async fn workflow_storage<T>(
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

pub async fn workflow_storage_buckets<T>(
    worker: &Worker<T>,
    dao: &AccountId,
) -> anyhow::Result<Vec<String>>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(dao, "storage_buckets", args).await?;
    view_outcome_pretty::<Vec<String>>("dao view workflow storage buckets", &outcome);
    let storage_buckets = parse_view_result::<Vec<String>>(&outcome)
        .expect("failed to parse workflow storage buckets");
    Ok(storage_buckets)
}

pub async fn debug_log<T>(worker: &Worker<T>, dao: &AccountId) -> anyhow::Result<Vec<String>>
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

pub async fn wf_log<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    proposal_id: u32,
) -> anyhow::Result<Option<Vec<ActionLog>>>
where
    T: DevNetwork,
{
    let args = json!({
        "proposal_id": proposal_id,
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&dao, "wf_log", args).await?;
    let title = format!("view wf log on dao: {}", dao.as_str());
    view_outcome_pretty::<Option<Vec<ActionLog>>>(&title, &outcome);
    let logs =
        parse_view_result::<Option<Vec<ActionLog>>>(&outcome).expect("failed to parse wf log");
    Ok(logs)
}

pub async fn get_timestamp<T>(worker: &Worker<T>, dao: &AccountId) -> anyhow::Result<u64>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(&dao, "current_timestamp", args).await?;
    view_outcome_pretty::<u64>("view current_timestamp", &outcome);
    let timestamp = parse_view_result::<u64>(&outcome).expect("failed to parse current_timestamp");
    Ok(timestamp)
}

pub async fn view_reward<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    reward_id: u16,
) -> anyhow::Result<Reward>
where
    T: DevNetwork,
{
    let args = json!({ "id": reward_id }).to_string().into_bytes();
    let outcome = worker.view(&dao, "reward", args).await?;
    view_outcome_pretty::<Reward>("view reward", &outcome);
    let reward = parse_view_result::<Reward>(&outcome).expect("failed to parse reward_id");
    Ok(reward)
}

pub async fn view_user_wallet<T>(
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
    let outcome = worker.view(&dao, "wallet", args).await?;
    view_outcome_pretty::<Wallet>("view user wallet", &outcome);
    let wallet = parse_view_result::<Wallet>(&outcome).expect("failed to parse wallet");
    Ok(wallet)
}

pub async fn view_user_roles<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    account_id: &AccountId,
) -> anyhow::Result<Option<UserRoles>>
where
    T: DevNetwork,
{
    let args = json!({
        "account_id": account_id.to_string()
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&dao, "user_roles", args).await?;
    let msg = format!("view user roles: {}", account_id.as_str());
    view_outcome_pretty::<Option<UserRoles>>(&msg, &outcome);
    let roles = parse_view_result::<Option<UserRoles>>(&outcome).flatten();
    Ok(roles)
}

pub async fn view_group_roles<T>(
    worker: &Worker<T>,
    dao: &AccountId,
    group_id: u16,
) -> anyhow::Result<Roles>
where
    T: DevNetwork,
{
    let args = json!({ "id": group_id }).to_string().into_bytes();
    let outcome = worker.view(&dao, "group_roles", args).await?;
    view_outcome_pretty::<Roles>("view group roles", &outcome);
    let roles = parse_view_result::<Roles>(&outcome).expect("failed to parse group roles");
    Ok(roles)
}

pub async fn statistics<T>(worker: &Worker<T>, dao: &AccountId) -> anyhow::Result<Statistics>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(&dao, "statistics", args).await?;
    view_outcome_pretty::<Statistics>("view dao statistics", &outcome);
    let stats = parse_view_result::<Statistics>(&outcome).expect("failed to parse dao statistics");
    Ok(stats)
}

pub async fn view_partitions<T>(
    worker: &Worker<T>,
    dao: &AccountId,
) -> anyhow::Result<Vec<(u16, TreasuryPartition)>>
where
    T: DevNetwork,
{
    let args = json!({
        "from_id": 0,
        "limit": 100,
    })
    .to_string()
    .into_bytes();
    let outcome = worker.view(&dao, "partition_list", args).await?;
    view_outcome_pretty::<Vec<(u16, TreasuryPartition)>>("view dao partition list", &outcome);
    let partitions = parse_view_result::<Vec<(u16, TreasuryPartition)>>(&outcome)
        .expect("failed to parse partition list");
    Ok(partitions)
}

pub async fn view_groups<T>(
    worker: &Worker<T>,
    dao: &AccountId,
) -> anyhow::Result<Vec<(u16, Group)>>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(&dao, "groups", args).await?;
    view_outcome_pretty::<Vec<(u16, Group)>>("view dao group list", &outcome);
    let groups =
        parse_view_result::<Vec<(u16, Group)>>(&outcome).expect("failed to parse group list");
    Ok(groups)
}

pub async fn view_media<T>(worker: &Worker<T>, dao: &AccountId) -> anyhow::Result<Vec<(u32, Media)>>
where
    T: DevNetwork,
{
    let args = json!({"from_id": 0, "limit": 100}).to_string().into_bytes();
    let outcome = worker.view(&dao, "media_list", args).await?;
    view_outcome_pretty::<Vec<(u32, Media)>>("view dao media list", &outcome);
    let media_list =
        parse_view_result::<Vec<(u32, Media)>>(&outcome).expect("failed to parse media list");
    Ok(media_list)
}
