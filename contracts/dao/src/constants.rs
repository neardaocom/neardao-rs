use near_sdk::Gas;

//TODO: With each upgrade auto inc +1
pub const VERSION: u8 = 1;

pub const GAS_DOWNLOAD_NEW_VERSION: Gas = Gas(250_000_000_000_000);
pub const GAS_UPGRADE: Gas = Gas(250_000_000_000_000);

pub const GROUP_RELEASE_PREFIX: &[u8; 3] = b"rml";
pub const STORAGE_BUCKET_PREFIX: &[u8; 3] = b"bkt";
pub const TGAS: Gas = Gas(1_000_000_000_000);
pub const METADATA_MAX_DECIMALS: u8 = 24;
pub const MIN_VOTING_DURATION_SEC: u32 = 300;

pub const GLOBAL_BUCKET_IDENT: &str = "global";

pub const EVENT_CALLER_KEY: &str = "event_caller";

// DAO consts ids.
pub const C_DAO_ID: u8 = 0;
pub const C_CURRENT_TIMESTAMP_SECS: u8 = 1;
pub const C_PREDECESSOR: u8 = 2;

pub const LATEST_REWARD_ACTIVITY_ID: u8 = 4;
