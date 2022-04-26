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

pub(crate) fn outcome_pretty(name: &str, outcome: CallExecutionDetails) {
    let result_data: String = outcome.json().unwrap_or_default();

    println!(
        r#"
    -------- OUT: {} --------
    sucess: {:?},
    total TGAS burnt: {},
    returned_data: {},
    logs: {:?},
    "#,
        name,
        outcome.is_success(),
        outcome.total_gas_burnt / 10u64.pow(12),
        result_data,
        outcome.logs(),
    )
}

pub(crate) fn view_outcome_pretty<T>(name: &str, outcome: ViewResultDetails)
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
