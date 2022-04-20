use workspaces::result::CallExecutionDetails;

pub(crate) const ROOT_PATH: &str = env!("CARGO_MANIFEST_DIR");
pub(crate) const DAO_WASM: &str = "dao.wasm";
pub(crate) const DAO_FACTORY_WASM: &str = "dao_factory.wasm";
pub(crate) const WF_PROVIDER: &str = "workflow_provider";

macro_rules! wasm_bin_getters {
    ( $($fnname:ident => $const:expr)*) => {
        $(
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
);

pub(crate) fn outcome_pretty(name: &str, outcome: CallExecutionDetails) {
    let result_data: String = outcome.json().unwrap_or("Failed to parse data".into());

    println!(
        r#"
    -------- OUT: {} --------
    sucess: {:?},
    total TGAS burnt: {},
    returned_data: {},
    "#,
        name,
        outcome.is_success(),
        outcome.total_gas_burnt / 10u64.pow(12),
        result_data,
    )
}
