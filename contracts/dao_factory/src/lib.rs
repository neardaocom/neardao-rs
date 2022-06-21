use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
    Promise,
};
use near_sdk::{Gas, IntoStorageKey};

///include binary code of dao contract
const NEWEST_DAO_VERSION: &[u8] = include_bytes!("../../../res/dao_opt.wasm");
const MIGRATION_BLOB: &[u8] = &[];

/// Gas spent on the call & account creation.
const CREATE_CALL_GAS: Gas = Gas(150_000_000_000_000);
/// Gas allocated on the callback.
const ON_CREATE_CALL_GAS: Gas = Gas(30_000_000_000_000);

const DEPOSIT_CREATE: u128 = 10_000_000_000_000_000_000_000_000;
const MAX_DAO_VERSIONS: u8 = 5;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum MigrationType {
    OnlyMigration,
    NewMigrationBin,
    NewUpgradeBin,
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
        dao_info: DaoInfo,
    ) -> bool;
    fn on_delete(&mut self, account: String) -> bool;
}

#[derive(BorshStorageKey, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub enum StorageKeys {
    Daos,
    CurrentVersion,
    LatestVersion,
    V1Migration,
    V1Upgrade,
    V2Migration,
    V2Upgrade,
    V3Migration,
    V3Upgrade,
    V4Migration,
    V4Upgrade,
    V5Migration,
    V5Upgrade,
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub daos: UnorderedMap<AccountId, DaoInfo>,
    pub tags: Vec<String>,
    pub latest_migration_version_idx: u8,
    pub latest_upgrade_version_idx: u8,
    pub version_count: u8,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(tags: Vec<String>) -> Self {
        env::storage_write(
            &StorageKeys::CurrentVersion.into_storage_key(),
            NEWEST_DAO_VERSION,
        );
        Self {
            daos: UnorderedMap::new(StorageKeys::Daos),
            latest_upgrade_version_idx: 1,
            latest_migration_version_idx: 1,
            version_count: 1,
            tags,
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate(r#type: MigrationType) -> Self {
        let mut factory: Contract = env::state_read().expect("failed to read contract state");
        if r#type != MigrationType::OnlyMigration {
            let is_upgrade = r#type == MigrationType::NewUpgradeBin;
            let key = factory.update_version_and_get_slot(is_upgrade);
            if is_upgrade {
                env::storage_write(&key.into_storage_key(), NEWEST_DAO_VERSION);
            } else {
                env::storage_write(&key.into_storage_key(), MIGRATION_BLOB);
            }
            if factory.latest_migration_version_idx == factory.latest_upgrade_version_idx {
                env::storage_write(
                    &StorageKeys::LatestVersion.into_storage_key(),
                    &factory.version_count.to_le_bytes(),
                );
            }
        }
        factory
    }

    pub fn get_dao_list(&self, from_index: u64, limit: u64) -> Vec<(AccountId, DaoInfo)> {
        let keys = self.daos.keys_as_vector();
        let values = self.daos.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, self.daos.len()))
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap()))
            .collect()
    }

    pub fn get_dao_info(&self, account: &AccountId) -> Option<DaoInfo> {
        self.daos.get(account)
    }

    pub fn get_stats(self) -> FactoryStats {
        FactoryStats {
            latest_dao_version: self.version_count,
            migration_versions: self.latest_migration_version_idx,
            upgrade_versions: self.latest_upgrade_version_idx,
            versions_stored: MAX_DAO_VERSIONS,
        }
    }

    #[payable]
    pub fn create(&mut self, name: String, info: DaoInfo, args: Base64VecU8) -> Promise {
        assert!(env::attached_deposit() >= DEPOSIT_CREATE);
        let account_id: AccountId = format!("{}.{}", name, env::current_account_id())
            .try_into()
            .expect("account is invalid");
        require!(
            self.get_dao_info(&account_id).is_none(),
            "dao already exists"
        );
        let promise = Promise::new(account_id.clone())
            .create_account()
            .deploy_contract(NEWEST_DAO_VERSION.to_vec())
            .transfer(env::attached_deposit());

        promise
            .function_call(
                "new".to_string(),
                args.into(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS - ON_CREATE_CALL_GAS,
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(ON_CREATE_CALL_GAS)
                    .on_create(
                        account_id,
                        U128(env::attached_deposit()),
                        env::predecessor_account_id(),
                        info,
                    ),
            )
    }

    pub fn on_create(
        &mut self,
        account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
        dao_info: DaoInfo,
    ) -> bool {
        if near_sdk::is_promise_success() {
            self.daos.insert(&account_id, &dao_info);
            log!("Created DAO account: {}", account_id);
            true
        } else {
            Promise::new(predecessor_account_id).transfer(attached_deposit.0);
            false
        }
    }

    pub fn get_tags(self) -> Vec<String> {
        self.tags
    }

    #[private]
    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.tags.extend(tags)
    }

    /// Removes all version blobs so we can delete factory account
    #[private]
    pub fn clean_self() {
        env::storage_remove(&StorageKeys::V1Migration.into_storage_key());
        env::storage_remove(&StorageKeys::V1Upgrade.into_storage_key());
        env::storage_remove(&StorageKeys::V2Migration.into_storage_key());
        env::storage_remove(&StorageKeys::V2Upgrade.into_storage_key());
        env::storage_remove(&StorageKeys::V3Migration.into_storage_key());
        env::storage_remove(&StorageKeys::V3Upgrade.into_storage_key());
        env::storage_remove(&StorageKeys::V4Migration.into_storage_key());
        env::storage_remove(&StorageKeys::V4Upgrade.into_storage_key());
        env::storage_remove(&StorageKeys::V5Migration.into_storage_key());
        env::storage_remove(&StorageKeys::V5Upgrade.into_storage_key());
    }
}

impl Contract {
    pub fn update_version_and_get_slot(&mut self, upgrade: bool) -> StorageKeys {
        if upgrade {
            self.latest_upgrade_version_idx += 1;
        } else {
            self.latest_migration_version_idx += 1;
        }

        if self.latest_migration_version_idx == self.latest_upgrade_version_idx {
            self.version_count += 1;
        }
        require!(
            self.latest_migration_version_idx
                .checked_sub(self.latest_upgrade_version_idx)
                .unwrap_or(2)
                <= 1,
            "load next upgrade bin first"
        );
        let key = if upgrade {
            match self.latest_upgrade_version_idx % 5 {
                1 => StorageKeys::V1Upgrade,
                2 => StorageKeys::V2Upgrade,
                3 => StorageKeys::V3Upgrade,
                4 => StorageKeys::V4Upgrade,
                0 => StorageKeys::V5Upgrade,
                _ => unreachable!(),
            }
        } else {
            match self.latest_migration_version_idx % 5 {
                1 => StorageKeys::V1Migration,
                2 => StorageKeys::V2Migration,
                3 => StorageKeys::V3Migration,
                4 => StorageKeys::V4Migration,
                0 => StorageKeys::V5Migration,
                _ => unreachable!(),
            }
        };
        key
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn get_next_version_keys(current_version: u8) -> (StorageKeys, StorageKeys) {
    let key_migration = match current_version % 5 {
        0 => StorageKeys::V1Migration,
        1 => StorageKeys::V2Migration,
        2 => StorageKeys::V3Migration,
        3 => StorageKeys::V4Migration,
        4 => StorageKeys::V5Migration,
        _ => unreachable!(),
    };
    let key_upgrade = match current_version % 5 {
        0 => StorageKeys::V1Upgrade,
        1 => StorageKeys::V2Upgrade,
        2 => StorageKeys::V3Upgrade,
        3 => StorageKeys::V4Upgrade,
        4 => StorageKeys::V5Upgrade,
        _ => unreachable!(),
    };
    (key_migration, key_upgrade)
}

/// Sends wasm blobs back to caller (dao) based on provided dao version
/// Dao must implement necessary store methods to be able to save the blobs.
/// Prepaid gas should be 200+ TGas
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    const GAS_DOWNLOAD_ACTION: Gas = Gas(100_000_000_000_000);
    env::setup_panic_hook();

    let version: u8 = *env::input().unwrap().get(0).unwrap();
    let caller = env::predecessor_account_id();
    let method_store_migration_bin = "store_migration_bin";
    let method_store_upgrade_bin = "store_upgrade_bin";

    log!("Got version: {:?}", version);
    let last_version_stored: u8 = u8::from_le(
        env::storage_read(&StorageKeys::LatestVersion.into_storage_key())
            .expect("no upgrade stored yet")[0],
    );
    require!(
        last_version_stored > version,
        "next version is not available"
    );
    require!(
        last_version_stored - version <= MAX_DAO_VERSIONS,
        "upgrade is not possible"
    );

    let (key_migration, key_upgrade) = get_next_version_keys(version);

    let migration_bin = env::storage_read(key_migration.into_storage_key().as_slice())
        .expect("migration code not found");
    let upgrade_bin = env::storage_read(key_upgrade.into_storage_key().as_slice())
        .expect("upgrade code not found");
    let promise_id = env::promise_batch_create(&caller);
    env::promise_batch_action_function_call(
        promise_id,
        method_store_migration_bin,
        migration_bin.as_slice(),
        0,
        GAS_DOWNLOAD_ACTION,
    );
    env::promise_batch_action_function_call(
        promise_id,
        method_store_upgrade_bin,
        upgrade_bin.as_slice(),
        0,
        GAS_DOWNLOAD_ACTION,
    );
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoInfo {
    pub founded_s: u64,
    pub name: String,
    pub description: String,
    pub ft_name: String,
    pub ft_amount: u32,
    pub tags: Vec<u8>,
}

#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct FactoryStats {
    latest_dao_version: u8,
    migration_versions: u8,
    upgrade_versions: u8,
    versions_stored: u8,
}

#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env};

    use super::*;
    #[test]
    fn rotate_slots() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());

        let mut factory = Contract::new(vec![]);
        assert_eq!(factory.version_count, 1);
        assert_eq!(factory.latest_migration_version_idx, 1);
        assert_eq!(factory.latest_upgrade_version_idx, 1);

        assert_eq!(
            factory.update_version_and_get_slot(false),
            StorageKeys::V2Migration
        );
        assert_eq!(factory.version_count, 1);
        assert_eq!(factory.latest_migration_version_idx, 2);
        assert_eq!(factory.latest_upgrade_version_idx, 1);
        assert_eq!(
            factory.update_version_and_get_slot(true),
            StorageKeys::V2Upgrade
        );
        assert_eq!(factory.version_count, 2);
        assert_eq!(factory.latest_migration_version_idx, 2);
        assert_eq!(factory.latest_upgrade_version_idx, 2);

        assert_eq!(
            factory.update_version_and_get_slot(false),
            StorageKeys::V3Migration
        );
        assert_eq!(factory.version_count, 2);
        assert_eq!(factory.latest_migration_version_idx, 3);
        assert_eq!(factory.latest_upgrade_version_idx, 2);

        assert_eq!(
            factory.update_version_and_get_slot(true),
            StorageKeys::V3Upgrade
        );
        assert_eq!(factory.version_count, 3);
        assert_eq!(factory.latest_migration_version_idx, 3);
        assert_eq!(factory.latest_upgrade_version_idx, 3);

        assert_eq!(
            factory.update_version_and_get_slot(false),
            StorageKeys::V4Migration
        );
        assert_eq!(factory.version_count, 3);
        assert_eq!(factory.latest_migration_version_idx, 4);
        assert_eq!(factory.latest_upgrade_version_idx, 3);

        assert_eq!(
            factory.update_version_and_get_slot(true),
            StorageKeys::V4Upgrade
        );
        assert_eq!(factory.version_count, 4);
        assert_eq!(factory.latest_migration_version_idx, 4);
        assert_eq!(factory.latest_upgrade_version_idx, 4);

        assert_eq!(
            factory.update_version_and_get_slot(false),
            StorageKeys::V5Migration
        );
        assert_eq!(factory.version_count, 4);
        assert_eq!(factory.latest_migration_version_idx, 5);
        assert_eq!(factory.latest_upgrade_version_idx, 4);

        assert_eq!(
            factory.update_version_and_get_slot(true),
            StorageKeys::V5Upgrade
        );
        assert_eq!(factory.version_count, 5);
        assert_eq!(factory.latest_migration_version_idx, 5);
        assert_eq!(factory.latest_upgrade_version_idx, 5);

        assert_eq!(
            factory.update_version_and_get_slot(false),
            StorageKeys::V1Migration
        );
        assert_eq!(factory.version_count, 5);
        assert_eq!(factory.latest_migration_version_idx, 6);
        assert_eq!(factory.latest_upgrade_version_idx, 5);

        assert_eq!(
            factory.update_version_and_get_slot(true),
            StorageKeys::V1Upgrade
        );
        assert_eq!(factory.version_count, 6);
        assert_eq!(factory.latest_migration_version_idx, 6);
        assert_eq!(factory.latest_upgrade_version_idx, 6);
    }

    #[test]
    #[should_panic]
    fn rotate_slots_invalid_use() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());

        let mut factory = Contract::new(vec![]);
        assert_eq!(factory.version_count, 1);
        assert_eq!(factory.latest_migration_version_idx, 1);
        assert_eq!(factory.latest_upgrade_version_idx, 1);

        assert_eq!(
            factory.update_version_and_get_slot(false),
            StorageKeys::V2Migration
        );
        assert_eq!(factory.version_count, 1);
        assert_eq!(factory.latest_migration_version_idx, 2);
        assert_eq!(factory.latest_upgrade_version_idx, 1);
        factory.update_version_and_get_slot(false);
    }

    #[test]
    fn get_next_version() {
        assert_eq!(
            get_next_version_keys(1),
            (StorageKeys::V2Migration, StorageKeys::V2Upgrade)
        );
        assert_eq!(
            get_next_version_keys(2),
            (StorageKeys::V3Migration, StorageKeys::V3Upgrade)
        );
        assert_eq!(
            get_next_version_keys(3),
            (StorageKeys::V4Migration, StorageKeys::V4Upgrade)
        );
        assert_eq!(
            get_next_version_keys(4),
            (StorageKeys::V5Migration, StorageKeys::V5Upgrade)
        );
        assert_eq!(
            get_next_version_keys(5),
            (StorageKeys::V1Migration, StorageKeys::V1Upgrade)
        );
        assert_eq!(
            get_next_version_keys(6),
            (StorageKeys::V2Migration, StorageKeys::V2Upgrade)
        );
    }
}
