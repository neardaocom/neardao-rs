use std::convert::TryFrom;

use near_sdk::CryptoHash;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base58CryptoHash, Base58PublicKey, Base64VecU8, U128, ValidAccountId};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, serde_json::json, AccountId,
    BorshStorageKey, PanicOnDefault, Promise,
};
use near_sdk::{env::BLOCKCHAIN_INTERFACE, IntoStorageKey, PromiseOrValue, PromiseResult};

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};

near_sdk::setup_alloc!();

///include binary code of dao contract
const NEWEST_DAO_VERSION: &[u8] = include_bytes!("../../res/dao.wasm");

/// Gas spent on the call & account creation.
const CREATE_CALL_GAS: u64 = 75_000_000_000_000;

/// Gas allocated on the callback.
const ON_CREATE_CALL_GAS: u64 = 10_000_000_000_000;

const DEPOSIT_CREATE: u128 = 5_000_000_000_000_000_000_000_000;
const MAX_DAO_VERSIONS: u8 = 5;
const GAS_SEND_BIN_LIMIT: u64 = 100_000_000_000_000;

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
pub enum StorageKeys {
    Daos,
    V1,
    V2,
    V3,
    V4,
    V5,
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DaoFactoryContract {
    pub daos: UnorderedMap<AccountId, DaoInfo>,
    pub key: Base58PublicKey, //TODO vec<u8>
    pub tags: Vec<String>,
    pub latest_dao_version_idx: u8,
    pub version_count: u8,
}

#[near_bindgen]
impl DaoFactoryContract {
    #[init]
    pub fn new(tags: Vec<String>) -> Self {
        env::storage_write(&StorageKeys::V1.into_storage_key(), NEWEST_DAO_VERSION);

        Self {
            daos: UnorderedMap::new(StorageKeys::Daos),
            key: Base58PublicKey::try_from(env::signer_account_pk()).unwrap(),
            latest_dao_version_idx: 1,
            version_count: 1,
            tags,
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate(dao_version_update: bool) -> Self {
        let mut dao: DaoFactoryContract = env::state_read().expect("Failed to migrate");

        if dao_version_update {
            // Inc version counter and rotate storage slots
            if dao.latest_dao_version_idx == MAX_DAO_VERSIONS {
                dao.latest_dao_version_idx = 1;
            } else {
                dao.latest_dao_version_idx += 1;
            }
            dao.version_count += 1;

            // Store new dao version to storage
            let key = match dao.latest_dao_version_idx {
                1 => StorageKeys::V1,
                2 => StorageKeys::V2,
                3 => StorageKeys::V3,
                4 => StorageKeys::V4,
                5 => StorageKeys::V5,
                _ => unreachable!(),
            };

            env::storage_write(&key.into_storage_key(), NEWEST_DAO_VERSION);
        }

        dao
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
        }
    }

    pub fn version_hash(self, version: u8) -> Option<Base64VecU8> {
        
        // Check it was already uploaded or we still keep this version 
        if version > self.version_count || self.version_count - version > 4 && version != 0 {
            return None;
        }

        let mut key = None;

        // Assume caller meant specific version
        if version > 0 {
            key = match version % 5 {
                1 => Some(StorageKeys::V1),
                2 => Some(StorageKeys::V2),
                3 => Some(StorageKeys::V3),
                4 => Some(StorageKeys::V4),
                0 => Some(StorageKeys::V5),
                _ => unreachable!(),
            };
        }
            
        let code = match key {
            Some(k) => {
                env::storage_read(&k.into_storage_key()).unwrap()
            },
            None => {
                NEWEST_DAO_VERSION.to_vec()
            }
        };

        Some(Base64VecU8::from(env::sha256(&code)))
    }

    #[payable]
    pub fn create(
        &mut self,
        acc_name: AccountId,
        public_key: Option<Base58PublicKey>, //TODO refactor
        dao_info: DaoInfo,
        args: Base64VecU8,
    ) -> Promise {
        assert!(env::attached_deposit() >= DEPOSIT_CREATE);
        let account_id = format!("{}.{}", acc_name, env::current_account_id());
        log!("Creating account: {}", account_id);

        assert!(
            self.get_dao_info(&account_id).is_none(),
            "{}",
            "Dao already exists"
        );

        let promise = Promise::new(account_id.clone())
            .create_account()
            .deploy_contract(NEWEST_DAO_VERSION.to_vec())
            .transfer(env::attached_deposit())
            .add_full_access_key(self.key.clone().into());

        promise
            .function_call(
                b"new".to_vec(),
                args.into(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS - ON_CREATE_CALL_GAS,
            )
            .then(ext_self::on_create(
                acc_name,
                U128(env::attached_deposit()),
                env::predecessor_account_id(),
                dao_info,
                &env::current_account_id(),
                0,
                ON_CREATE_CALL_GAS,
            ))
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
        env::storage_remove(&StorageKeys::V1.into_storage_key());
        env::storage_remove(&StorageKeys::V2.into_storage_key());
        env::storage_remove(&StorageKeys::V3.into_storage_key());
        env::storage_remove(&StorageKeys::V4.into_storage_key());
        env::storage_remove(&StorageKeys::V5.into_storage_key());
    }
}

/// Sends wasm blob back to caller (dao) based on provided dao version
/// Dao must implement store_new_version method
/// Prepaid gas should be 100+ TGas
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_dao_bin() {
    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    log!("version: {:?}", *env::input().unwrap().get(0).unwrap());

    let predecessor = env::predecessor_account_id().into_bytes();
    let method_name = "store_new_version".as_bytes().to_vec();
    let key = match *env::input().unwrap().get(0).unwrap() as u8 % 5 {
        0 => StorageKeys::V1,
        1 => StorageKeys::V2,
        2 => StorageKeys::V3,
        3 => StorageKeys::V4,
        4 => StorageKeys::V5,
        _ => unreachable!(),
    }
    .into_storage_key();

    unsafe {
        BLOCKCHAIN_INTERFACE.with(|b| {
            b.borrow()
                .as_ref()
                .unwrap()
                .storage_read(key.len() as _, key.as_ptr() as _, 0);

            b.borrow().as_ref().unwrap().promise_create(
                predecessor.len() as _,
                predecessor.as_ptr() as _,
                method_name.len() as _,
                method_name.as_ptr() as _,
                u64::MAX as _,
                0,
                0 as _,
                GAS_SEND_BIN_LIMIT,
            )
        });
    }
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
}

#[cfg(test)]
mod tests {
    //TODO
}
