use serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::{Account, AccountId, DevNetwork, Worker};

use crate::utils::{outcome_pretty, parse_view_result, view_outcome_pretty};

pub async fn dao_download_new_version_and_start_migration<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = caller
        .call(&worker, dao, "download_migration_and_upgrade")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("dao download new version binaries", &outcome);
    assert!(
        outcome.is_success(),
        "dao download new version binaries failed"
    );

    let args = json!({}).to_string().into_bytes();
    let outcome = caller
        .call(&worker, dao, "start_migration")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("dao start migration", &outcome);
    assert!(outcome.is_success(), "dao start migration failed");
    Ok(())
}

pub async fn dao_migrate_data<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = caller
        .call(&worker, dao, "migrate_data")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("dao migrate data", &outcome);
    assert!(outcome.is_success(), "dao migrate data");
    Ok(())
}

pub async fn dao_upgrade<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = caller
        .call(&worker, dao, "upgrade")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("dao upgrade", &outcome);
    assert!(outcome.is_success(), "dao upgrade");
    Ok(())
}

pub async fn dao_add_dummy_data<T>(
    worker: &Worker<T>,
    caller: &Account,
    dao: &AccountId,
    data: Vec<&str>,
) -> anyhow::Result<()>
where
    T: DevNetwork,
{
    let args = json!({ "data": data }).to_string().into_bytes();
    let outcome = caller
        .call(&worker, dao, "add_dummy_data")
        .args(args)
        .max_gas()
        .transact()
        .await?;
    outcome_pretty::<()>("dao add upgrade data", &outcome);
    assert!(outcome.is_success(), "dao add upgrade data");
    Ok(())
}

pub async fn dao_view_dummy_data<T>(
    worker: &Worker<T>,
    dao: &AccountId,
) -> anyhow::Result<(Vec<TestDataPrev>, Vec<VersionedNonMigrableTestDataPrev>)>
where
    T: DevNetwork,
{
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(dao, "view_dummy_data", args).await?;
    view_outcome_pretty::<(Vec<TestDataPrev>, Vec<VersionedNonMigrableTestDataPrev>)>(
        "dao view dummy data",
        &outcome,
    );
    let data =
        parse_view_result::<(Vec<TestDataPrev>, Vec<VersionedNonMigrableTestDataPrev>)>(&outcome)
            .unwrap();
    Ok(data)
}

pub async fn dao_view_dummy_data_after_migration<T>(
    worker: &Worker<T>,
    dao: &AccountId,
) -> anyhow::Result<(Vec<TestDataNew>, Vec<VersionedNonMigrableTestDataNew>)>
where
    T: DevNetwork,
{
    println!("check after migration and upgrade");
    let args = json!({}).to_string().into_bytes();
    let outcome = worker.view(dao, "view_dummy_data", args).await?;
    view_outcome_pretty::<(Vec<TestDataNew>, Vec<VersionedNonMigrableTestDataNew>)>(
        "dao view upgrade after migration data",
        &outcome,
    );
    let data =
        parse_view_result::<(Vec<TestDataNew>, Vec<VersionedNonMigrableTestDataNew>)>(&outcome)
            .unwrap();
    Ok(data)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TestDataPrev {
    string_data: String,
    //garbage: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TestDataNew {
    string_data: String,
    new_string_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VersionedNonMigrableTestDataPrev {
    Current(NonMigrableTestDataPrev),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VersionedNonMigrableTestDataNew {
    Current(NonMigrableTestDataPrev),
    New(NonMigrableTestDataNew),
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct NonMigrableTestDataPrev {
    string_data: String,
    //garbage: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct NonMigrableTestDataNew {
    string_data: String,
    new_string_data: String,
}
