use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

use near_sdk::json_types::{U128, U64};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use workspaces::{
    network::{Sandbox, Testnet},
    result::{CallExecutionDetails, ViewResultDetails},
    Worker,
};

pub const DAO_FT_TOTAL_SUPPLY: u128 = 1_000_000_000;
pub const DEFAULT_DECIMALS: u128 = 10u128.pow(24);

pub(crate) type DurationSec = u64;
pub(crate) type TimestampSec = u64;
pub(crate) type WrappedBalance = U128;
pub(crate) type WrappedDuration = U64;
pub(crate) type WrappedTimestamp = U64;
pub(crate) type MethodName = String;
pub(crate) type FnCallId = (near_sdk::AccountId, MethodName);

pub(crate) const MAX_GAS: u64 = 300 * 10u64.pow(12);

pub(crate) const ROOT_PATH: &str = env!("CARGO_MANIFEST_DIR");
pub(crate) const DAO: &str = "dao.wasm";
pub(crate) const DAO_FACTORY: &str = "dao_factory.wasm";
pub(crate) const WF_PROVIDER: &str = "workflow_provider.wasm";
pub(crate) const STAKING: &str = "staking.wasm";
pub(crate) const TOKEN: &str = "fungible_token.wasm";
pub(crate) const FT_FACTORY: &str = "ft_factory.wasm";

// 3rd party contracts.
pub(crate) const SKYWARD: &str = "05022022_skyward.wasm";
pub(crate) const WNEAR: &str = "w_near.wasm";

macro_rules! wasm_bin_getters {
    ( $($fnname:ident => $const:expr)*) => {
        $(
            /// Returns path of wasm blob.
            pub(crate) fn $fnname() -> String {
                format!("{}/../../res/{}",ROOT_PATH,$const)
            }
        )*
    };
    (EXTERNAL $($fnname:ident => $const:expr)*) => {
        $(
            /// Returns path of external wasm blob.
            pub(crate) fn $fnname() -> String {
                format!("{}/../res_3rd_party/{}",ROOT_PATH,$const)
            }
        )*
    };
}

wasm_bin_getters!(
    get_dao_wasm => DAO
    get_dao_factory_wasm => DAO_FACTORY
    get_wf_provider_wasm => WF_PROVIDER
    get_staking_wasm => STAKING
    get_fungible_token_wasm => TOKEN
    get_ft_factory_wasm => FT_FACTORY
);

wasm_bin_getters!(
    EXTERNAL
    get_skyward_wasm => SKYWARD
    get_wnear_wasm => WNEAR
);

pub(crate) fn outcome_pretty<T>(name: &str, outcome: &CallExecutionDetails)
where
    T: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    let result_data: Option<T> = outcome.json().unwrap_or_default();

    println!(
        r#"
    -------- OUT: {} --------
    success: {:?},
    total TGAS burnt: {},
    NEARs burnt: {},
    returned_data: {:?},
    logs: {:?},
    "#,
        name,
        outcome.is_success(),
        outcome.total_gas_burnt / 10u64.pow(12),
        (outcome.total_gas_burnt / 10u64.pow(12)) as f64 / 10f64.powf(4.0),
        result_data,
        outcome.logs(),
    )
}

pub(crate) fn view_outcome_pretty<T>(name: &str, outcome: &ViewResultDetails)
where
    T: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    let result_data: Option<T> = outcome.json().unwrap_or_default();

    println!(
        r#"
    -------- VIEW OUT: {} --------
    returned_data: {:?},
    logs: {:?},
    "#,
        name, result_data, outcome.logs,
    )
}

pub(crate) fn parse_call_outcome<T>(outcome: &CallExecutionDetails) -> Option<T>
where
    T: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    outcome.json().unwrap_or_default()
}

pub(crate) fn parse_view_result<T>(outcome: &ViewResultDetails) -> Option<T>
where
    T: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    outcome.json().unwrap_or_default()
}

pub(crate) fn generate_random_strings(count: usize, string_len: usize) -> Vec<String> {
    assert!(count > 0, "Called with zero");
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        let string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(string_len)
            .map(char::from)
            .collect();
        vec.push(string);
    }
    vec
}

#[async_trait]
pub trait Wait {
    async fn wait(&self, min_seconds: u64) -> anyhow::Result<()>;
}

#[async_trait]
impl Wait for Worker<Sandbox> {
    async fn wait(&self, min_seconds: u64) -> anyhow::Result<()> {
        self.fast_forward(min_seconds + 3).await?;
        Ok(())
    }
}

#[async_trait]
impl Wait for Worker<Testnet> {
    async fn wait(&self, min_seconds: u64) -> anyhow::Result<()> {
        sleep(Duration::from_secs(min_seconds)).await;
        Ok(())
    }
}
