use rand::{distributions::Alphanumeric, thread_rng, Rng};
use workspaces::result::{CallExecutionDetails, ViewResultDetails};

pub(crate) const ROOT_PATH: &str = env!("CARGO_MANIFEST_DIR");
pub(crate) const DAO_WASM: &str = "dao.wasm";
pub(crate) const DAO_FACTORY_WASM: &str = "dao_factory.wasm";
pub(crate) const WF_PROVIDER: &str = "workflow_provider";
pub(crate) const STAKING: &str = "staking.wasm";
pub(crate) const TOKEN: &str = "fungible_token.wasm";

macro_rules! wasm_bin_getters {
    ( $($fnname:ident => $const:expr)*) => {
        $(
            /// Returns path of wasm blob.
            pub(crate) fn $fnname() -> String {
                format!("{}/../../res/{}",ROOT_PATH,$const)
            }
        )*
    };
}

wasm_bin_getters!(
    get_dao_wasm => DAO_WASM
    get_dao_factory_wasm => DAO_FACTORY_WASM
    get_wf_provider_wasm => WF_PROVIDER
    get_staking => STAKING
    get_fungible_token => TOKEN
);

pub(crate) fn outcome_pretty(name: &str, outcome: &CallExecutionDetails) {
    let result_data: String = outcome.json().unwrap_or_default();

    println!(
        r#"
    -------- OUT: {} --------
    sucess: {:?},
    total TGAS burnt: {},
    NEARs burnt: {},
    returned_data: {},
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
