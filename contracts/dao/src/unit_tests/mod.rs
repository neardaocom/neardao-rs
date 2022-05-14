#![allow(unused, dead_code)]
#![cfg(test)]

use std::{collections::HashMap, convert::TryFrom};

use data::workflow::basic::wf_add::WfAdd1;
use library::{
    workflow::{
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, ObjectMetadata, VoteScenario},
    },
    FnCallId, MethodName,
};
use near_sdk::{test_utils::VMContextBuilder, AccountId};

use crate::{
    core::Contract,
    group::{GroupInput, GroupMember, GroupSettings},
    settings::Settings,
    tags::TagInput,
    DurationSec,
};

//mod dao; // Require refactoring to match new structure
mod group;
mod reward;
mod tick;
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
const FACTORY_ACC: &str = "neardao.testnet";
const DAO_ACC: &str = "dao_instance.neardao.testnet";
const OWNER_ACC_FULLNAME: &str = "dao_instance.dao_factory.testnet";

const DAO_NAME: &str = "dao";
const DAO_DESC: &str = "dao description";

const FOUNDER_1: &str = "founder_1.testnet";
const FOUNDER_2: &str = "founder_2.testnet";
const FOUNDER_3: &str = "founder_3.testnet";
const FOUNDER_4: &str = "founder_4.testnet";
const FOUNDER_5: &str = "founder_5.testnet";

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
        .signer_account_id(as_account_id(FACTORY_ACC))
        .predecessor_account_id(as_account_id(FACTORY_ACC))
        .current_account_id(as_account_id(DAO_ACC))
        .account_balance(10u128.pow(24)); //10 nears
    builder
}

pub(crate) fn get_contract(
    token_id: AccountId,
    staking_id: AccountId,
    total_supply: u32,
    settings: Settings,
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
        as_account_id(TOKEN_ACC),
        as_account_id(STAKING_ACC),
        TOKEN_TOTAL_SUPPLY,
        get_default_dao_config(),
        get_default_groups(),
        get_default_tags(),
        get_default_standard_fncalls(),
        get_default_standard_fncall_metadata(),
        get_default_fncalls(),
        get_default_fncall_metadata(),
        get_default_templates(),
        get_default_template_settings(),
        0,
    )
}

pub(crate) fn decimal_const() -> u128 {
    10u128.pow(METADATA_DECIMALS as u32)
}

pub(crate) fn get_default_dao_config() -> Settings {
    Settings {
        name: DAO_NAME.into(),
        purpose: "test".into(),
        tags: vec![0, 1, 2],
        dao_admin_account_id: as_account_id(DAO_ADMIN_ACC),
        dao_admin_rights: vec!["all".into()],
        workflow_provider: as_account_id(WF_PROVIDER_ACC),
    }
}

pub(crate) fn get_default_groups() -> Vec<GroupInput> {
    let mut members = vec![
        GroupMember {
            account_id: as_account_id(FOUNDER_1),
            tags: vec![0],
        },
        GroupMember {
            account_id: as_account_id(FOUNDER_2),
            tags: vec![1],
        },
        GroupMember {
            account_id: as_account_id(FOUNDER_3),
            tags: vec![2],
        },
    ];
    let mut groups = Vec::with_capacity(1);
    let mut member_roles = HashMap::new();
    member_roles.insert("leader".into(), vec![as_account_id(FOUNDER_1)]);
    member_roles.insert(
        "other".into(),
        vec![as_account_id(FOUNDER_2), as_account_id(FOUNDER_3)],
    );
    groups.push(GroupInput {
        settings: GroupSettings {
            name: "council".into(),
            leader: Some(as_account_id(FOUNDER_1)),
            parent_group: 0,
        },
        members,
        member_roles,
    });

    groups
}

pub(crate) fn get_default_tags() -> Vec<TagInput> {
    vec![]
}
pub(crate) fn get_default_fncalls() -> Vec<FnCallId> {
    WfAdd1::template(WF_PROVIDER_ACC.into()).1
}
pub(crate) fn get_default_standard_fncalls() -> Vec<MethodName> {
    vec![]
}

pub(crate) fn get_default_standard_fncall_metadata() -> Vec<Vec<ObjectMetadata>> {
    vec![]
}

pub(crate) fn get_default_fncall_metadata() -> Vec<Vec<ObjectMetadata>> {
    WfAdd1::template(WF_PROVIDER_ACC.into()).2
}
pub(crate) fn get_default_templates() -> Vec<Template> {
    vec![WfAdd1::template(WF_PROVIDER_ACC.into()).0]
}
pub(crate) fn get_default_template_settings() -> Vec<Vec<TemplateSettings>> {
    vec![vec![WfAdd1::template_settings(None)]]
}

pub(crate) fn as_account_id(name: &str) -> AccountId {
    AccountId::new_unchecked(name.to_string())
}

pub(crate) fn dummy_propose_settings() -> ProposeSettings {
    ProposeSettings {
        global: None,
        binds: vec![None, None],
        storage_key: None,
    }
}

pub(crate) fn dummy_template_settings() -> TemplateSettings {
    TemplateSettings {
        allowed_proposers: vec![],
        allowed_voters: ActivityRight::Group(1),
        activity_rights: vec![],
        transition_limits: vec![],
        scenario: VoteScenario::Democratic,
        duration: 60,
        quorum: 10,
        approve_threshold: 50,
        spam_threshold: 80,
        vote_only_once: true,
        deposit_propose: None,
        deposit_vote: None,
        deposit_propose_return: 0,
        constants: None,
    }
}
