#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn upgrade() {
    use crate::{
        constants::GAS_UPGRADE,
        core::{Contract, StorageKeys},
        settings::Settings,
    };
    use library::workflow::types::ActivityRight;
    use near_sdk::{env, log};
    use near_sdk::{require, IntoStorageKey};

    env::setup_panic_hook();

    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().unwrap().into();

    require!(contract.check_rights(&[ActivityRight::Group(1)], &caller));

    // TODO: Add migration done flag.
    let migration_done = true;
    require!(migration_done, "migrate data first");

    let current_acc = env::current_account_id();
    let method_name = "deploy_upgrade_bin";
    let key = StorageKeys::NewVersionUpgradeBin.into_storage_key();

    let code = env::storage_read(key.as_slice()).expect("missing upgrade bin");
    let promise_id = env::promise_batch_create(&current_acc);
    env::promise_batch_action_deploy_contract(promise_id, code.as_slice());
    env::promise_batch_action_function_call(promise_id, method_name, &[], 0, GAS_UPGRADE / 2);
}
