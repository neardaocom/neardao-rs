/// Triggers new version migration download from factory.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_migration_and_upgrade() {
    use crate::{
        constants::{GAS_DOWNLOAD_NEW_VERSION, VERSION},
        core::Contract,
        settings::Settings,
    };
    use library::workflow::types::ActivityRight;
    use near_sdk::{env, require};

    env::setup_panic_hook();

    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().unwrap().into();

    require!(contract.check_rights(&[ActivityRight::Group(1)], &caller));

    let admin_acc = settings.dao_admin_account_id;
    let method_name = "download_new_version";

    env::promise_create(
        admin_acc,
        method_name,
        &[VERSION],
        0,
        GAS_DOWNLOAD_NEW_VERSION,
    );
}

/// Method called by dao admin as response to download_migration_and_upgrade method.
/// Saves provided dao binary in storage under "NewVersionMigrationBin".
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn store_migration_bin() {
    use crate::{
        core::{Contract, StorageKeys},
        settings::Settings,
    };
    use library::workflow::types::ActivityRight;
    use near_sdk::env;
    use near_sdk::{require, IntoStorageKey};

    env::setup_panic_hook();

    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().unwrap().into();

    require!(settings.dao_admin_account_id == caller);

    env::storage_write(
        &StorageKeys::NewVersionMigrationBin.into_storage_key(),
        &env::input().unwrap(),
    );
}

/// Method called by dao admin as response to download_migration_and_upgrade method.
/// Saves provided dao binary in storage under "NewVersionUpgradeBin".
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn store_upgrade_bin() {
    use crate::{
        core::{Contract, StorageKeys},
        settings::Settings,
    };
    use library::workflow::types::ActivityRight;
    use near_sdk::env;
    use near_sdk::{require, IntoStorageKey};

    env::setup_panic_hook();

    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().unwrap().into();

    require!(settings.dao_admin_account_id == caller);

    env::storage_write(
        &StorageKeys::NewVersionUpgradeBin.into_storage_key(),
        &env::input().unwrap(),
    );
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn start_migration() {
    use crate::{
        constants::GAS_UPGRADE,
        core::{Contract, StorageKeys},
        settings::Settings,
    };
    use library::workflow::types::ActivityRight;
    use near_sdk::IntoStorageKey;
    use near_sdk::{env, require};

    env::setup_panic_hook();

    let caller = env::predecessor_account_id();
    let contract: Contract = env::state_read().unwrap();
    let settings: Settings = contract.settings.get().unwrap().into();

    require!(contract.check_rights(&[ActivityRight::Group(1)], &caller));
    require!(
        env::storage_has_key(&StorageKeys::NewVersionUpgradeBin.into_storage_key()),
        "missing upgrade bin"
    );

    let current_acc = env::current_account_id();
    let method_name = "deploy_migration_bin";
    let key = StorageKeys::NewVersionMigrationBin.into_storage_key();

    let code = env::storage_read(key.as_slice()).expect("missing migration bin");
    let promise = env::promise_batch_create(&current_acc);
    env::promise_batch_action_deploy_contract(promise, code.as_slice());
    env::promise_batch_action_function_call(promise, method_name, &[], 0, GAS_UPGRADE);
}
