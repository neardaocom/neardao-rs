#![cfg(target_arch = "wasm32")]

use near_sdk::Gas;

pub const GAS_DOWNLOAD_NEW_VERSION: Gas = Gas(250_000_000_000_000);
pub const GAS_UPGRADE: Gas = Gas(250_000_000_000_000);

/// Triggers new version download from upgrade provider.
#[no_mangle]
pub extern "C" fn download_upgrade() {
    use crate::{Contract, Settings};
    use near_sdk::{env, require};

    env::setup_panic_hook();
    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().expect("No settings.");
    require!(settings.owner_id == caller, "No rights.");
    let upgrade_provider = settings
        .upgrade_provider
        .expect("Upgrade provider is not defined.");
    let method_name = "download_new_version";
    env::promise_create(
        upgrade_provider,
        method_name,
        &[],
        0,
        GAS_DOWNLOAD_NEW_VERSION,
    );
}

/// Method called by upgrade provider as response to download_upgrade method.
/// Saves provided upgrade binary in storage under "NewVersionCode".
#[no_mangle]
pub extern "C" fn store_upgrade_bin() {
    use crate::{Contract, Settings, StorageKeys};
    use near_sdk::env;
    use near_sdk::{require, IntoStorageKey};

    env::setup_panic_hook();
    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().expect("No settings.");
    require!(
        settings
            .upgrade_provider
            .expect("Upgrade provider is not defined.")
            == caller
    );
    env::storage_write(
        &StorageKeys::NewVersionCode.into_storage_key(),
        &env::input().unwrap(),
    );
}

#[no_mangle]
pub extern "C" fn start_upgrade() {
    use crate::{Contract, Settings, StorageKeys};
    use near_sdk::IntoStorageKey;
    use near_sdk::{env, require};

    env::setup_panic_hook();
    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().expect("No settings.");
    require!(settings.owner_id == caller, "No rights.");
    let current_acc = env::current_account_id();
    let method_name = "deploy_upgrade";
    let key = StorageKeys::NewVersionCode.into_storage_key();
    let code = env::storage_read(key.as_slice()).expect("Missing upgrade bin.");
    let promise = env::promise_batch_create(&current_acc);
    env::promise_batch_action_deploy_contract(promise, code.as_slice());
    env::promise_batch_action_function_call(promise, method_name, &[], 0, GAS_UPGRADE);
}
