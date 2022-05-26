#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn upgrade() {
    use crate::{
        constants::GAS_UPGRADE,
        core::{Contract, StorageKeys},
        settings::Settings,
    };
    use library::workflow::types::ActivityRight;
    use near_sdk::IntoStorageKey;
    use near_sdk::{env, log};

    env::setup_panic_hook();

    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().unwrap().into();

    assert!(contract.check_rights(&[ActivityRight::Group(1)], &caller));

    let current_acc = env::current_account_id();
    let method_name = "deploy_upgrade_bin";
    let key = StorageKeys::NewVersionUpgradeBin.into_storage_key();
    log!("running upgrade");

    let code = env::storage_read(key.as_slice()).expect("Failed to read code from storage.");
    let promise_id = env::promise_batch_create(&current_acc);
    env::promise_batch_action_deploy_contract(promise_id, code.as_slice());
    env::promise_batch_action_function_call(promise_id, method_name, &[], 0, GAS_UPGRADE / 2);
}
