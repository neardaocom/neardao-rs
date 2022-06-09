#![allow(unused, dead_code)]
#![cfg(test)]

use std::{collections::HashMap, convert::TryFrom};

use data::workflow::basic::basic_package::WfBasicPkg1;
use library::{
    locking::{LockInput, UnlockMethod, UnlockPeriodInput, UnlockingInput},
    workflow::{
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::{ActivityRight, ObjectMetadata, VoteScenario},
    },
    FnCallId, MethodName,
};
use near_sdk::{test_utils::VMContextBuilder, AccountId};

use crate::{
    constants::LATEST_REWARD_ACTIVITY_ID,
    contract::Contract,
    group::{GroupInput, GroupMember, GroupSettings},
    media::Media,
    role::{MemberRoles, Roles, UserRoles},
    settings::{AdminRight, Settings},
    tags::TagInput,
    treasury::{Asset, PartitionAssetInput, TreasuryPartitionInput},
    wallet::{ClaimableReward, Wallet, WithdrawStats},
    RewardId, RoleId,
};

//mod dao; // Require refactoring to match new structure
mod group;
mod reward;
pub mod treasury;
mod voting;
pub mod workflow;

pub const DURATION_1Y_S: u32 = 31_536_000;
pub const DURATION_2Y_S: u32 = 63_072_000;
pub const DURATION_3Y_S: u32 = 94_608_000;

pub const RELEASE_TIME: u64 = 63_072_000_000_000_000;
pub const DURATION_ONE_WEEK: u64 = 604_800_000_000_000;
pub const DURATION_1Y: u64 = 31_536_000_000_000_000;
pub const DURATION_2Y: u64 = 63_072_000_000_000_000;
pub const DURATION_3Y: u64 = 94_608_000_000_000_000;

const TOKEN_ACC: &str = "some.token.neardao.testnet";
const VOTE_TOKEN_ACC: &str = "dao-vote-token.neardao.testnet";
const DAO_ADMIN_ACC: &str = "admin.neardao.testnet";
const STAKING_ACC: &str = "staking.neardao.testnet";
const WF_PROVIDER_ACC: &str = "wf-provider.neardao.testnet";
const RESOURCE_PROVIDER_ACC: &str = "resource.neardao.testnet";
const SCHEDULER_ACC: &str = "scheduler.neardao.testnet";
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

const ACC_1: &str = "some_account_1.testnet";
const ACC_2: &str = "some_account_2.testnet";
const ACC_3: &str = "some_account_3.testnet";

const GROUP_1_NAME: &str = "council";
const GROUP_2_NAME: &str = "holy_men";

const GROUP_1_ROLE_1: &str = "role for group leader";
const GROUP_1_ROLE_2: &str = "other";

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
    treasury_partitions: Vec<TreasuryPartitionInput>,
    media: Vec<Media>,
) -> Contract {
    Contract::new(
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
        treasury_partitions,
        media,
    )
}

pub(crate) fn get_default_contract() -> Contract {
    let contract = get_contract(
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
        get_default_treasury_partitions(),
        get_default_media(),
    );
    assert_eq!(
        contract.total_members_count, 6,
        "invalid total unique members count"
    );
    assert!(contract.partition_last_id == 2);
    assert!(contract.treasury_partition.get(&1).is_some());
    assert!(contract.treasury_partition.get(&2).is_some());
    assert_eq!(contract.group_roles(1).unwrap(), default_group_1_roles());
    assert_eq!(contract.group_roles(2).unwrap(), default_group_2_roles());
    assert_user_roles(&contract, as_account_id(FOUNDER_1), Some(founder_1_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_2), Some(founder_2_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
    assert_group_role_members(&contract, 1, 1, vec![as_account_id(FOUNDER_1)]);
    contract
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
        dao_admin_rights: vec![AdminRight::Upgrade],
        workflow_provider: as_account_id(WF_PROVIDER_ACC),
        resource_provider: Some(as_account_id(RESOURCE_PROVIDER_ACC)),
        scheduler: Some(as_account_id(SCHEDULER_ACC)),
        token_id: as_account_id(TOKEN_ACC),
        staking_id: as_account_id(STAKING_ACC),
    }
}

pub(crate) fn get_default_groups() -> Vec<GroupInput> {
    let mut groups = Vec::with_capacity(2);
    let mut group_1_members = vec![
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
    let group_1_roles = vec![MemberRoles {
        name: GROUP_1_ROLE_1.into(),
        members: vec![as_account_id(FOUNDER_1)],
    }];
    groups.push(GroupInput {
        settings: GroupSettings {
            name: GROUP_1_NAME.into(),
            leader: Some(as_account_id(FOUNDER_1)),
            parent_group: 0,
        },
        members: group_1_members,
        member_roles: group_1_roles,
    });

    let mut group_2_members = vec![
        GroupMember {
            account_id: as_account_id(ACC_1),
            tags: vec![],
        },
        GroupMember {
            account_id: as_account_id(ACC_2),
            tags: vec![],
        },
        GroupMember {
            account_id: as_account_id(ACC_3),
            tags: vec![],
        },
        GroupMember {
            account_id: as_account_id(FOUNDER_2),
            tags: vec![],
        },
        GroupMember {
            account_id: as_account_id(FOUNDER_3),
            tags: vec![],
        },
    ];
    let group_2_roles: Vec<MemberRoles> = vec![];
    groups.push(GroupInput {
        settings: GroupSettings {
            name: GROUP_2_NAME.into(),
            leader: Some(as_account_id(ACC_1)),
            parent_group: 1,
        },
        members: group_2_members,
        member_roles: group_2_roles,
    });
    groups
}

pub fn founder_1_roles() -> UserRoles {
    UserRoles::new().add_role(1, 0).add_role(1, 1)
}
pub fn founder_2_roles() -> UserRoles {
    UserRoles::new().add_role(1, 0).add_role(2, 0)
}
pub fn founder_3_roles() -> UserRoles {
    UserRoles::new().add_role(1, 0).add_role(2, 0)
}

pub(crate) fn default_group_1_roles() -> Roles {
    let mut roles = Roles::new();
    roles.insert(GROUP_1_ROLE_1.into());
    roles
}

pub(crate) fn default_group_2_roles() -> Roles {
    let mut roles = Roles::new();
    roles
}

pub(crate) fn get_default_tags() -> Vec<TagInput> {
    vec![]
}
pub(crate) fn get_default_fncalls() -> Vec<FnCallId> {
    WfBasicPkg1::template(WF_PROVIDER_ACC.into()).1
}
pub(crate) fn get_default_standard_fncalls() -> Vec<MethodName> {
    vec![]
}

pub(crate) fn get_default_standard_fncall_metadata() -> Vec<Vec<ObjectMetadata>> {
    vec![]
}

pub(crate) fn get_default_fncall_metadata() -> Vec<Vec<ObjectMetadata>> {
    WfBasicPkg1::template(WF_PROVIDER_ACC.into()).2
}
pub(crate) fn get_default_templates() -> Vec<Template> {
    vec![WfBasicPkg1::template(WF_PROVIDER_ACC.into()).0]
}
pub(crate) fn get_default_template_settings() -> Vec<Vec<TemplateSettings>> {
    vec![vec![WfBasicPkg1::template_settings(None)]]
}

pub(crate) fn as_account_id(name: &str) -> AccountId {
    AccountId::new_unchecked(name.to_string())
}

pub(crate) fn dummy_propose_settings() -> ProposeSettings {
    ProposeSettings {
        constants: None,
        activity_constants: vec![None, None, None, None, None],
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

pub(crate) fn update_template_settings_vote_rights(
    contract: &mut Contract,
    template_id: u16,
    settings_id: u16,
    allowed_voters: ActivityRight,
) {
    let (template, mut settings) = contract.workflow_template.get(&template_id).unwrap();
    settings
        .get_mut(settings_id as usize)
        .unwrap()
        .allowed_voters = allowed_voters;
    contract
        .workflow_template
        .insert(&template_id, &(template, settings));
}

pub(crate) fn get_role_id(contract: &Contract, group_id: u16, role_name: &str) -> u16 {
    let group_roles = contract
        .group_roles
        .get(&group_id)
        .expect("group not found");
    let role_id = group_roles
        .iter()
        .find(|(key, name)| name.as_str() == role_name)
        .expect("role not found");
    *role_id.0
}

pub(crate) fn get_default_treasury_partitions() -> Vec<TreasuryPartitionInput> {
    vec![
        TreasuryPartitionInput {
            name: "near_partition".into(),
            assets: vec![PartitionAssetInput {
                asset_id: Asset::new_near(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 100,
                    lock: None,
                },
            }],
        },
        TreasuryPartitionInput {
            name: "vote_token_partition".into(),
            assets: vec![PartitionAssetInput {
                asset_id: Asset::new_ft(as_account_id(VOTE_TOKEN_ACC), 24),
                unlocking: UnlockingInput {
                    amount_init_unlock: 0,
                    lock: Some(LockInput {
                        amount_total_lock: TOKEN_TOTAL_SUPPLY,
                        start_from: 0,
                        duration: 1000,
                        periods: vec![UnlockPeriodInput {
                            r#type: UnlockMethod::Linear,
                            duration: 1000,
                            amount: TOKEN_TOTAL_SUPPLY,
                        }],
                    }),
                },
            }],
        },
    ]
}

pub fn get_default_media() -> Vec<Media> {
    vec![]
}

/// Convert timestamp seconds to miliseconds
/// Contract internally works with seconds.
fn tm(v: u64) -> u64 {
    v * 10u64.pow(9)
}

fn get_wallet(contract: &Contract, account_id: &AccountId) -> Wallet {
    let wallet: Wallet = contract
        .wallets
        .get(&account_id)
        .expect("wallet not found")
        .into();
    wallet
}

fn get_wallet_withdraw_stat<'a>(
    wallet: &'a Wallet,
    reward_id: u16,
    asset: &Asset,
) -> &'a WithdrawStats {
    let wallet_reward = wallet
        .wallet_reward(reward_id)
        .expect("wallet reward nout found");
    wallet_reward.withdraw_stat(asset)
}

fn claimable_rewards_sum(claimable_rewards: &[ClaimableReward], asset: &Asset) -> u128 {
    let mut sum = 0;
    for reward in claimable_rewards.into_iter() {
        if reward.asset == *asset {
            sum += reward.amount.0
        }
    }
    sum
}

fn assert_user_roles(
    contract: &Contract,
    account_id: AccountId,
    mut expected_roles: Option<UserRoles>,
) {
    let mut user_roles = contract.user_roles(account_id);
    let user_roles = user_roles.as_ref().map(|r| r.to_owned().sort());
    let expected_roles = expected_roles.as_ref().map(|r| r.to_owned().sort());
    assert_eq!(user_roles, expected_roles);
}

fn assert_group_role_members(
    contract: &Contract,
    group_id: u16,
    role_id: u16,
    mut expected_members: Vec<AccountId>,
) {
    let group = contract.group(group_id).unwrap();
    let mut actual_members = contract.get_group_members_with_role(group_id, &group, role_id);
    actual_members.sort();
    expected_members.sort();
    assert_eq!(actual_members, expected_members);
}

fn assert_group_rewards(
    contract: &Contract,
    group_id: u16,
    mut expected_reward_ids: Vec<(RewardId, RoleId)>,
) {
    let group = contract.group(group_id).expect("group not found");
    let mut reward_ids = group.group_reward_ids();
    reward_ids.sort();
    expected_reward_ids.sort();
    assert_eq!(reward_ids, expected_reward_ids);
}

fn assert_cache_reward_activity(contract: &Contract, expected_cache: Vec<(u8, Vec<u16>)>) {
    for id in 0..=LATEST_REWARD_ACTIVITY_ID {
        if let Some(pos) = expected_cache.iter().position(|(idx, _)| *idx == id) {
            let mut expected = expected_cache[pos].clone().1;
            expected.sort();
            let mut actual = contract.cache_reward_activity.get(&id).unwrap();
            actual.sort();
            assert_eq!(actual, expected);
        }
    }
}
