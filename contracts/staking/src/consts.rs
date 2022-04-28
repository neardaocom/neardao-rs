use near_sdk::{Balance, Gas, StorageUsage};

pub const U64_LEN: StorageUsage = 8;
pub const U128_LEN: StorageUsage = 16;
pub const ACCOUNT_MAX_LENGTH: StorageUsage = 64;
pub const DAO_KEY_PREFIX: &[u8; 4] = b"dao_";
pub const ACCOUNT_STATS_SIZE: StorageUsage = 2 * U128_LEN + U64_LEN;

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);

/// Amount of gas for delegate action.
pub const GAS_FOR_DELEGATE: Gas = Gas(10_000_000_000_000);

/// Amount of gas for register action.
pub const GAS_FOR_REGISTER: Gas = Gas(10_000_000_000_000);

/// Amount of gas for undelegate action.
pub const GAS_FOR_UNDELEGATE: Gas = Gas(10_000_000_000_000);

/// Measured value for item in LM where K is String.
pub const LOOKUP_MAP_ITEM_STORAGE: StorageUsage = 45;
/// 0.2 NEAR
pub const MIN_STORAGE: StorageUsage = 20_000;
pub const MIN_STORAGE_FOR_DAO: StorageUsage = 61;
pub const ACCOUNT_STATS_STORAGE: StorageUsage = LOOKUP_MAP_ITEM_STORAGE + 2 * U128_LEN + U64_LEN;

pub const MIN_REGISTER_DEPOSIT: Balance = 155 * 10u128.pow(21);

pub mod error_messages {
    pub const ERR_NOT_ENOUGH_AMOUNT: &str = "Not enough amount";
}
