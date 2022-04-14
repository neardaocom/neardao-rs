use near_sdk::Gas;

//TODO: With each upgrade auto inc +1
pub const VERSION: u8 = 1;

pub const GAS_DOWNLOAD_NEW_VERSION: Gas = Gas(200_000_000_000_000);
pub const GAS_UPGRADE: Gas = Gas(200_000_000_000_000);

pub const GROUP_RELEASE_PREFIX: &[u8; 3] = b"rml";
pub const STORAGE_BUCKET_PREFIX: &[u8; 3] = b"bkt";
pub const TGAS: Gas = Gas(1_000_000_000_000);
pub const METADATA_MAX_DECIMALS: u8 = 24;
pub const MAX_FT_TOTAL_SUPPLY: u32 = 1_000_000_000;
pub const MIN_VOTING_DURATION_SEC: u32 = 300;

pub const GLOBAL_BUCKET_IDENT: &str = "global";

// DAO CONSTS
pub const C_DAO_ACC_ID: u8 = 0;
