#![allow(unused, dead_code)]
#![cfg(test)]

use std::convert::TryFrom;

use library::{
    workflow::{settings::TemplateSettings, template::Template, types::ObjectMetadata},
    FnCallId, MethodName,
};
use near_sdk::{test_utils::VMContextBuilder, AccountId};

use crate::{
    core::Contract,
    group::{GroupInput, GroupMember, GroupSettings, GroupTokenLockInput},
    settings::DaoSettings,
    tags::TagInput,
    token_lock::{UnlockMethod, UnlockPeriodInput},
    DurationSec,
};

//mod dao; // Require refactoring to match new structure
mod group;
mod tick;
mod unlocking;
mod voting;

pub const DURATION_1Y_S: u32 = 31_536_000;
pub const DURATION_2Y_S: u32 = 63_072_000;
pub const DURATION_3Y_S: u32 = 94_608_000;

pub const RELEASE_TIME: u64 = 63_072_000_000_000_000;
pub const DURATION_ONE_WEEK: u64 = 604_800_000_000_000;
pub const DURATION_1Y: u64 = 31_536_000_000_000_000;
pub const DURATION_2Y: u64 = 63_072_000_000_000_000;
pub const DURATION_3Y: u64 = 94_608_000_000_000_000;

const TOKEN_ACC: &str = "some.token.neardao.testnet";
const DAO_ADMIN_ACC: &str = "admin.neardao.testnet";
const STAKING_ACC: &str = "staking.neardao.testnet";
const WF_PROVIDER_ACC: &str = "wf-provider.neardao.testnet";
const ISSUER_ACC: &str = "dao_factory";
const OWNER_ACC: &str = "dao_instance";
const OWNER_ACC_FULLNAME: &str = "dao_instance.dao_factory";

const DAO_NAME: &str = "dao";
const DAO_DESC: &str = "dao description";

const FOUNDER_1: &str = "founder_1";
const FOUNDER_2: &str = "founder_2";
const FOUNDER_3: &str = "founder_3";
const FOUNDER_4: &str = "founder_4";
const FOUNDER_5: &str = "founder_5";

/// Max possible FT supply.
const TOKEN_TOTAL_SUPPLY: u32 = 1_000_000_000;
const INIT_DISTRIBUTION: u32 = 200_000_000;
const METADATA_DECIMALS: u8 = 24;

const DURATION_WAITING: u64 = 10_000_000_000;

#[macro_export]
macro_rules! create_val_to_percent_closure {
    ($e:expr, $t:ty) => {
        |p| ($e as u128 * p / 100) as $t
    };
}

pub(crate) fn get_context_builder() -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .block_timestamp(0)
        .signer_account_id(AccountId::try_from(ISSUER_ACC.to_string()).unwrap()) // Who started the transaction - DaoFactory in our case
        .predecessor_account_id(AccountId::try_from(ISSUER_ACC.to_string()).unwrap()) // Previous cross-contract caller, its called directly from DaoFactory so its same as signer
        .current_account_id(AccountId::try_from(OWNER_ACC.to_string()).unwrap()) // Account owning this smart contract
        .account_balance(10u128.pow(24)); //10 nears
    builder
}

pub(crate) fn get_contract(
    token_id: AccountId,
    staking_id: AccountId,
    total_supply: u32,
    settings: DaoSettings,
    groups: Vec<GroupInput>,
    tags: Vec<TagInput>,
    standard_function_calls: Vec<String>,
    standard_function_call_metadata: Vec<Vec<ObjectMetadata>>,
    function_calls: Vec<FnCallId>,
    function_call_metadata: Vec<Vec<ObjectMetadata>>,
    workflow_templates: Vec<Template>,
    workflow_template_settings: Vec<Vec<TemplateSettings>>,
    tick_interval: DurationSec,
) -> Contract {
    Contract::new(
        token_id,
        staking_id,
        total_supply,
        24,
        settings,
        groups,
        tags,
        standard_function_calls,
        standard_function_call_metadata,
        function_calls,
        function_call_metadata,
        workflow_templates,
        workflow_template_settings,
        tick_interval,
    )
}

pub(crate) fn get_default_contract() -> Contract {
    get_contract(
        AccountId::new_unchecked(TOKEN_ACC.into()),
        AccountId::new_unchecked(STAKING_ACC.into()),
        TOKEN_TOTAL_SUPPLY,
        get_default_dao_config(),
        get_default_groups(),
        get_default_tags(),
        get_default_standard_fncalls(),
        get_default_standard_fncall_metadata(),
        get_default_fncalls(),
        get_default_fncall_metadata(),
        get_default_templates(),
        get_efault_template_settings(),
        0,
    )
}

pub(crate) fn decimal_const() -> u128 {
    10u128.pow(METADATA_DECIMALS as u32)
}

pub(crate) fn get_default_dao_config() -> DaoSettings {
    DaoSettings {
        name: DAO_NAME.into(),
        purpose: "test".into(),
        tags: vec![0, 1, 2],
        dao_admin_account_id: DAO_ADMIN_ACC.to_string().try_into().unwrap(),
        dao_admin_rights: vec!["all".into()],
        workflow_provider: WF_PROVIDER_ACC.to_string().try_into().unwrap(),
    }
}

pub(crate) fn get_default_groups() -> Vec<GroupInput> {
    let mut members = vec![
        GroupMember {
            account_id: FOUNDER_1.to_string().try_into().unwrap(),
            tags: vec![0],
        },
        GroupMember {
            account_id: FOUNDER_2.to_string().try_into().unwrap(),
            tags: vec![1],
        },
        GroupMember {
            account_id: FOUNDER_3.to_string().try_into().unwrap(),
            tags: vec![2],
        },
    ];
    let mut groups = Vec::with_capacity(1);
    groups.push(GroupInput {
        settings: GroupSettings {
            name: "council".into(),
            leader: Some(FOUNDER_1.to_string().try_into().unwrap()),
            parent_group: 0,
        },
        members: members,
        token_lock: Some(GroupTokenLockInput {
            amount: 100_000_000,
            start_from: 0,
            duration: 3600,
            init_distribution: 10_000_000,
            unlock_interval: 60,
            periods: vec![UnlockPeriodInput {
                kind: UnlockMethod::Linear,
                duration: 3600,
                amount: 90_000_000,
            }],
        }),
    });

    groups
}

pub(crate) fn get_default_tags() -> Vec<TagInput> {
    vec![]
}
pub(crate) fn get_default_fncalls() -> Vec<FnCallId> {
    vec![]
}
pub(crate) fn get_default_standard_fncalls() -> Vec<MethodName> {
    vec![]
}

pub(crate) fn get_default_standard_fncall_metadata() -> Vec<Vec<ObjectMetadata>> {
    vec![]
}

pub(crate) fn get_default_fncall_metadata() -> Vec<Vec<ObjectMetadata>> {
    vec![]
}
pub(crate) fn get_default_templates() -> Vec<Template> {
    vec![]
}
pub(crate) fn get_efault_template_settings() -> Vec<Vec<TemplateSettings>> {
    vec![]
}
