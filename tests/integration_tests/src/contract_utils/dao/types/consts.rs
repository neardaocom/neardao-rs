// TODO: Figure out some meta structures about contracts and move this there.

use near_sdk::{ONE_NEAR, ONE_YOCTO};

pub const DAO_TPL_ID_WF_ADD: u16 = 1;
pub const DAO_TPL_ID_SKYWARD: u16 = 2;

pub const PROVIDER_TPL_ID_SKYWARD: u16 = 1;

pub const DEPOSIT_PROPOSE_WF_ADD: u128 = ONE_NEAR;
pub const DEPOSIT_VOTE_WF_ADD: u128 = ONE_YOCTO;

pub const DAO_VIEW_INSTANCE: &str = "wf_instance";
pub const DAO_VIEW_TEMPLATES: &str = "wf_templates";
pub const DAO_VIEW_WORKFLOW_STORAGE: &str = "storage_bucket_data_all";

pub const PROVIDER_VIEW_TEMPLATE: &str = "wf_template";
