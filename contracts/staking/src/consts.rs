use near_sdk::{Gas, StorageUsage};

pub const U64_LEN: StorageUsage = 8;
pub const U128_LEN: StorageUsage = 16;
pub const ACCOUNT_MAX_LENGTH: StorageUsage = 64;
pub const DAO_KEY_PREFIX: &[u8; 4] = b"dao_";

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);

/// Amount of gas for delegate action.
pub const GAS_FOR_DELEGATE: Gas = Gas(10_000_000_000_000);

/// Amount of gas for register action.
pub const GAS_FOR_REGISTER: Gas = Gas(10_000_000_000_000);

/// Amount of gas for undelegate action.
pub const GAS_FOR_UNDELEGATE: Gas = Gas(10_000_000_000_000);

pub mod error_messages {
    pub const ERR_NOT_ENOUGH_AMOUNT: &str = "Not enough amount";
}

pub const STORAGE_DEPOSIT_MIN: StorageUsage = 200;
pub const STORAGE_PER_DAO: StorageUsage = 155;
pub const STORAGE_DEPOSIT_FOR_DAO: StorageUsage = 125;
pub const STORAGE_PER_DELEGATE: StorageUsage = 64 + 64 + 24 + 3;
