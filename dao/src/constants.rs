//TODO: With each upgrade +1 !!! TODO safe auto-inc mechanism
pub const VERSION: u8 = 1;

pub const GROUP_PREFIX: &[u8; 3] = b"grp";
pub const GROUP_RELEASE_PREFIX: &[u8; 3] = b"rml";

pub const ACC_REF_FINANCE: &str = "pstu.testnet"; //"ref-finance.near";
pub const ACC_SKYWARD_FINANCE: &str = "supertest.testnet"; //"skyward.near";
pub const ACC_WNEAR: &str = "wrap.testnet"; //"wrap.near";

pub const GAS_MIN_DOWNLOAD_LIMIT: u64 = 200_000_000_000_000;
pub const GAS_MIN_UPGRADE_LIMIT: u64 = 100_000_000_000_000;
pub const GAS_ADD_PROPOSAL: u64 = 100_000_000_000_000;
pub const GAS_FINISH_PROPOSAL: u64 = 100_000_000_000_000;
pub const TGAS: u64 = 1_000_000_000_000;

pub const DEPOSIT_ADD_PROPOSAL: u128 = 500_000_000_000_000_000_000_000; // 0.5 N
pub const DEPOSIT_VOTE: u128 = 1_000_000_000_000_000_000_000; // 0.00100 N
pub const DEPOSIT_STANDARD_STORAGE: u128 = 1_250_000_000_000_000_000_000; // 0.00125 N

pub const METADATA_MAX_DECIMALS: u8 = 24;

pub const MAX_FT_TOTAL_SUPPLY: u32 = 1_000_000_000;

// Must match count of proposal variants, awating for std::mem:variant_count to be stable
pub const PROPOSAL_KIND_COUNT: u8 = 8;

pub const DEFAULT_DOC_CAT: &str = "basic";

pub const CID_MAX_LENGTH: u8 = 64;

pub const MIN_VOTING_DURATION_SEC: u32 = 300;