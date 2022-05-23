use std::collections::HashMap;

use library::workflow::types::{ActivityRight, VoteScenario};
use near_sdk::{test_utils::accounts, testing_env, AccountId, MockedBlockchain};

use crate::{
    core::Contract,
    group::{Group, GroupInput, GroupMember, GroupSettings},
    reward::{Reward, RewardType},
    role::{MemberRoles, Roles, UserRoles},
    treasury::Asset,
    unit_tests::{
        as_account_id, assert_group_role_members, assert_user_roles, claimable_rewards_sum,
        decimal_const, default_group_1_roles, default_group_2_roles, founder_1_roles,
        founder_2_roles, founder_3_roles, get_default_contract, tm, FOUNDER_1, FOUNDER_2,
        FOUNDER_3, FOUNDER_4, FOUNDER_5, GROUP_1_ROLE_1,
    },
    wallet::Wallet,
};

use super::get_context_builder;

fn assert_group_members(contract: &Contract, group_id: u16, mut expected_members: Vec<AccountId>) {
    let group = contract.group(group_id).unwrap();
    let mut actual_members = group.get_members_accounts();
    actual_members.sort();
    expected_members.sort();
    assert_eq!(actual_members, expected_members);
}

fn assert_wallet(
    wallet: &Wallet,
    expected_reward_id: u16,
    expected_assets: Vec<Asset>,
    expected_time_added: u64,
    expected_time_removed: Option<u64>,
) {
    let wallet_reward = wallet
        .wallet_reward(expected_reward_id)
        .expect("wallet reward id not found");
    assert_eq!(wallet_reward.time_added(), expected_time_added);
    assert_eq!(wallet_reward.time_removed(), expected_time_removed);
    assert_eq!(
        wallet_reward
            .withdraw_stats()
            .into_iter()
            .map(|s| s.reward_asset().clone())
            .collect::<Vec<Asset>>(),
        expected_assets
    );
}

#[test]
fn group_add() {
    let ctx = get_context_builder();
    testing_env!(ctx.build());

    let mut contract = get_default_contract();

    assert_eq!(contract.total_members_count, 6);
    let new_group_members = vec![
        GroupMember {
            account_id: as_account_id(FOUNDER_4),
            tags: vec![0],
        },
        GroupMember {
            account_id: as_account_id(FOUNDER_5),
            tags: vec![1],
        },
    ];

    contract.group_add(GroupInput {
        settings: GroupSettings {
            name: "council_rest".into(),
            leader: Some(as_account_id(FOUNDER_4)),
            parent_group: 0,
        },
        members: new_group_members.clone(),
        member_roles: vec![],
    });

    assert_eq!(contract.total_members_count, 8);
}

#[test]
fn group_add_members_only_members() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    assert_user_roles(&contract, accounts(0), None);
    assert_user_roles(&contract, accounts(1), None);
    assert_user_roles(&contract, accounts(2), None);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    let new_group_members = vec![
        GroupMember {
            account_id: accounts(0),
            tags: vec![],
        },
        GroupMember {
            account_id: accounts(1),
            tags: vec![],
        },
        GroupMember {
            account_id: accounts(2),
            tags: vec![],
        },
    ];
    let new_member_roles = vec![];
    let members_count = new_group_members.len();
    contract.group_add_members(1, new_group_members, new_member_roles);
    assert_eq!(contract.total_members_count, 9);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    assert_eq!(group_roles, default_group_1_roles());
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
            accounts(0),
            accounts(1),
            accounts(2),
        ],
    );
    let expected_user_roles = UserRoles::new().add_role(1, 0);
    assert_user_roles(&contract, as_account_id(FOUNDER_1), Some(founder_1_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_2), Some(founder_2_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
    assert_user_roles(&contract, accounts(0), Some(expected_user_roles.clone()));
    assert_user_roles(&contract, accounts(1), Some(expected_user_roles.clone()));
    assert_user_roles(&contract, accounts(2), Some(expected_user_roles.clone()));
}

#[test]
fn group_add_members_only_roles() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    let new_group_members = vec![];
    let new_member_roles = vec![
        MemberRoles {
            name: "bilboswaggings".into(),
            members: vec![],
        },
        MemberRoles {
            name: "rustyboi".into(),
            members: vec![],
        },
        MemberRoles {
            name: "hamstalover".into(),
            members: vec![],
        },
    ];
    let members_count = new_group_members.len();
    contract.group_add_members(1, new_group_members, new_member_roles);
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_user_roles(&contract, as_account_id(FOUNDER_1), Some(founder_1_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_2), Some(founder_2_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
    let group_roles = contract.group_roles(1).unwrap();
    let mut expected_roles = default_group_1_roles();
    expected_roles.insert("bilboswaggings".into());
    expected_roles.insert("rustyboi".into());
    expected_roles.insert("hamstalover".into());
    assert_eq!(group_roles, expected_roles);
}

#[test]
fn group_add_members_members_and_new_roles() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    assert_user_roles(&contract, accounts(0), None);
    assert_user_roles(&contract, accounts(1), None);
    assert_user_roles(&contract, accounts(2), None);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    let new_group_members = vec![
        GroupMember {
            account_id: accounts(0),
            tags: vec![],
        },
        GroupMember {
            account_id: accounts(1),
            tags: vec![],
        },
        GroupMember {
            account_id: accounts(2),
            tags: vec![],
        },
    ];
    let new_member_roles = vec![
        MemberRoles {
            name: "bilboswaggings".into(),
            members: vec![accounts(0)],
        },
        MemberRoles {
            name: "rustyboi".into(),
            members: vec![accounts(1)],
        },
        MemberRoles {
            name: "hamstalover".into(),
            members: vec![],
        },
    ];
    let members_count = new_group_members.len();
    contract.group_add_members(1, new_group_members, new_member_roles);
    assert_eq!(contract.total_members_count, 9);
    assert_eq!(contract.group_last_id, 2);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
            accounts(0),
            accounts(1),
            accounts(2),
        ],
    );
    let expected_user_roles = UserRoles::new().add_role(1, 0);
    assert_user_roles(&contract, as_account_id(FOUNDER_1), Some(founder_1_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_2), Some(founder_2_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
    assert_user_roles(
        &contract,
        accounts(0),
        Some(expected_user_roles.clone().add_role(1, 2)),
    );
    assert_user_roles(
        &contract,
        accounts(1),
        Some(expected_user_roles.clone().add_role(1, 3)),
    );
    assert_user_roles(&contract, accounts(2), Some(expected_user_roles.clone()));
    let group_roles = contract.group_roles(1).unwrap();
    let mut expected_roles = default_group_1_roles();
    expected_roles.insert("bilboswaggings".into());
    expected_roles.insert("rustyboi".into());
    expected_roles.insert("hamstalover".into());
    assert_eq!(group_roles, expected_roles);
}

#[test]
fn group_add_members_roles_for_existing_members() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    assert_group_role_members(&contract, 1, 1, vec![as_account_id(FOUNDER_1)]);
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    let new_group_members = vec![];
    let new_member_roles = vec![
        MemberRoles {
            name: GROUP_1_ROLE_1.into(),
            members: vec![as_account_id(FOUNDER_2), as_account_id(FOUNDER_3)],
        },
        MemberRoles {
            name: "bilboswaggings".into(),
            members: vec![as_account_id(FOUNDER_1)],
        },
        MemberRoles {
            name: "rustyboi".into(),
            members: vec![as_account_id(FOUNDER_2)],
        },
        MemberRoles {
            name: "hamstalover".into(),
            members: vec![],
        },
    ];
    let members_count = new_group_members.len();
    contract.group_add_members(1, new_group_members, new_member_roles);
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_user_roles(
        &contract,
        as_account_id(FOUNDER_1),
        Some(founder_1_roles().add_role(1, 2)),
    );
    assert_user_roles(
        &contract,
        as_account_id(FOUNDER_2),
        Some(founder_2_roles().add_role(1, 1).add_role(1, 3)),
    );
    assert_user_roles(
        &contract,
        as_account_id(FOUNDER_3),
        Some(founder_3_roles().add_role(1, 1)),
    );
    assert_group_role_members(
        &contract,
        1,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_group_role_members(&contract, 1, 2, vec![as_account_id(FOUNDER_1)]);
    assert_group_role_members(&contract, 1, 3, vec![as_account_id(FOUNDER_2)]);
    let group_roles = contract.group_roles(1).unwrap();
    let mut expected_roles = default_group_1_roles();
    expected_roles.insert("bilboswaggings".into());
    expected_roles.insert("rustyboi".into());
    expected_roles.insert("hamstalover".into());
    assert_eq!(group_roles, expected_roles);
}

/// Roles are added to the group but members are ignored.
#[test]
fn group_add_roles_with_unknown_members() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    let new_group_members = vec![];
    let new_member_roles = vec![
        MemberRoles {
            name: "bilboswaggings".into(),
            members: vec![accounts(0)],
        },
        MemberRoles {
            name: "rustyboi".into(),
            members: vec![accounts(1), accounts(2)],
        },
        MemberRoles {
            name: "hamstalover".into(),
            members: vec![],
        },
    ];
    let members_count = new_group_members.len();
    contract.group_add_members(1, new_group_members, new_member_roles);
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_user_roles(&contract, as_account_id(FOUNDER_1), Some(founder_1_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_2), Some(founder_2_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
    assert_user_roles(&contract, accounts(0), None);
    assert_user_roles(&contract, accounts(1), None);
    assert_user_roles(&contract, accounts(2), None);
    let group_roles = contract.group_roles(1).unwrap();
    let mut expected_roles = default_group_1_roles();
    expected_roles.insert("bilboswaggings".into());
    expected_roles.insert("rustyboi".into());
    expected_roles.insert("hamstalover".into());
    assert_eq!(group_roles, expected_roles);
}

#[test]
fn group_remove_members() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    let group = contract.group(1).unwrap();
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    contract.group_remove_members(1, vec![as_account_id(FOUNDER_1), as_account_id(FOUNDER_3)]);
    let group = contract.group(1).unwrap();
    assert_group_members(&contract, 1, vec![as_account_id(FOUNDER_2)]);
    assert!(group.settings.leader.is_none());
    assert_eq!(contract.total_members_count, 5);
}

#[test]
fn group_remove_member_role_only_role() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    contract.group_remove_roles(1, vec![1]);
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    let group_roles = contract.group_roles(1).unwrap();
    let mut expected_group_1_roles = default_group_1_roles();
    expected_group_1_roles.remove(1);
    assert_eq!(group_roles, expected_group_1_roles.clone());
    assert_user_roles(
        &contract,
        as_account_id(FOUNDER_1),
        Some(founder_1_roles().remove_role(1, 1)),
    );
    assert_user_roles(&contract, as_account_id(FOUNDER_2), Some(founder_2_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
    contract.group_remove_roles(1, vec![0]);
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    let group_roles = contract.group_roles(1).unwrap();
    assert_eq!(group_roles, expected_group_1_roles.clone());
    assert_user_roles(
        &contract,
        as_account_id(FOUNDER_1),
        Some(founder_1_roles().remove_role(1, 1)),
    );
    assert_user_roles(&contract, as_account_id(FOUNDER_2), Some(founder_2_roles()));
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
}

#[test]
fn group_remove_member_role_with_members() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles);
    assert_group_role_members(&contract, 1, 1, vec![as_account_id(FOUNDER_1)]);
    contract.group_add_members(
        1,
        vec![],
        vec![MemberRoles {
            name: GROUP_1_ROLE_1.into(),
            members: vec![as_account_id(FOUNDER_2)],
        }],
    );
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_group_role_members(
        &contract,
        1,
        0,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_group_role_members(
        &contract,
        1,
        1,
        vec![as_account_id(FOUNDER_1), as_account_id(FOUNDER_2)],
    );
    contract.group_remove_member_roles(
        1,
        vec![MemberRoles {
            name: GROUP_1_ROLE_1.into(),
            members: vec![as_account_id(FOUNDER_1), as_account_id(FOUNDER_3)],
        }],
    );
    assert_group_members(
        &contract,
        1,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_group_role_members(
        &contract,
        1,
        0,
        vec![
            as_account_id(FOUNDER_1),
            as_account_id(FOUNDER_2),
            as_account_id(FOUNDER_3),
        ],
    );
    assert_group_role_members(&contract, 1, 1, vec![as_account_id(FOUNDER_2)]);
    assert_user_roles(
        &contract,
        as_account_id(FOUNDER_1),
        Some(founder_1_roles().remove_role(1, 1)),
    );
    assert_user_roles(
        &contract,
        as_account_id(FOUNDER_2),
        Some(founder_2_roles().add_role(1, 1)),
    );
    assert_user_roles(&contract, as_account_id(FOUNDER_3), Some(founder_3_roles()));
    let group_roles = contract.group_roles(1).unwrap();
    let expected_group_1_roles = default_group_1_roles();
    assert_eq!(group_roles, expected_group_1_roles.clone());
}

/// Test case description:
/// 1. A group is created.
/// 2. Reward is defined for the group.
/// 3. Some new member is added.
/// 4. Other group member is removed.
/// 5. Group is removed.
/// Bonus: Withdraw rewards.
#[test]
fn group_scenario() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert_eq!(contract.total_members_count, 6);
    assert_eq!(contract.group_last_id, 2);

    let founder_account = as_account_id(FOUNDER_1);
    let expected_role_founder = UserRoles::new().add_group_roles(1, vec![0, 1]);
    let expected_roles_artists = UserRoles::new();

    assert_user_roles(&contract, accounts(0), None);
    assert_user_roles(&contract, accounts(1), None);
    assert_user_roles(&contract, accounts(2), None);
    assert_user_roles(
        &contract,
        founder_account.clone(),
        Some(expected_role_founder),
    );
    assert!(contract.wallet(accounts(0)).is_none());
    assert!(contract.wallet(accounts(1)).is_none());
    assert!(contract.wallet(accounts(2)).is_none());
    assert!(contract.wallet(founder_account.clone()).is_none());

    // 1. Add group.
    let roles_acc_1 = contract.user_roles(accounts(0));
    assert!(roles_acc_1.is_none());
    assert!(contract.group(3).is_none());
    let new_group_members = vec![
        GroupMember {
            account_id: accounts(0),
            tags: vec![],
        },
        GroupMember {
            account_id: accounts(1),
            tags: vec![],
        },
        GroupMember {
            account_id: accounts(2),
            tags: vec![],
        },
        GroupMember {
            account_id: founder_account.clone(),
            tags: vec![],
        },
    ];
    let members_count = new_group_members.len();
    contract.group_add(GroupInput {
        settings: GroupSettings {
            name: "artists".into(),
            leader: Some(accounts(0)),
            parent_group: 0,
        },
        members: new_group_members.clone(),
        member_roles: vec![],
    });
    assert_eq!(contract.total_members_count as usize, 6 + members_count - 1); // Note: FOUNDER_1 is already in the council group.
    assert_eq!(contract.group_last_id, 3);
    let group = contract.group(3).unwrap();
    assert_eq!(group.settings.leader.clone(), Some(accounts(0)));
    assert_eq!(group.group_reward_ids(), vec![]);
    assert_group_members(
        &contract,
        3,
        vec![
            accounts(0),
            accounts(1),
            accounts(2),
            as_account_id(FOUNDER_1),
        ],
    );
    let expected_role_founder = UserRoles::new()
        .add_group_roles(1, vec![0, 1])
        .add_group_roles(3, vec![0]);
    let expected_roles_artists = UserRoles::new().add_group_roles(3, vec![0]);
    assert_user_roles(&contract, accounts(0), Some(expected_roles_artists.clone()));
    assert_user_roles(&contract, accounts(1), Some(expected_roles_artists.clone()));
    assert_user_roles(&contract, accounts(2), Some(expected_roles_artists.clone()));
    assert_user_roles(
        &contract,
        founder_account.clone(),
        Some(expected_role_founder),
    );

    // 2. Add reward.
    assert!(contract.wallet(accounts(0)).is_none());
    assert!(contract.wallet(accounts(1)).is_none());
    assert!(contract.wallet(accounts(2)).is_none());
    assert!(contract.wallet(founder_account.clone()).is_none());
    let reward_asset = Asset::Near;
    let reward = Reward::new(
        3,
        0,
        1,
        RewardType::new_wage(1),
        vec![(reward_asset.clone(), 1)],
        0,
        1000,
    );
    let reward_id = contract.reward_add(reward);
    assert_eq!(contract.reward_last_id, reward_id);
    assert!(contract.reward(reward_id).is_some());
    assert!(contract.reward(reward_id + 1).is_none());
    let group = contract.group(3).unwrap();
    assert_eq!(group.group_reward_ids(), vec![(1, 0)]);
    let wallet_acc_1 = contract.wallet(accounts(0)).unwrap();
    let wallet_acc_2 = contract.wallet(accounts(1)).unwrap();
    let wallet_acc_3 = contract.wallet(accounts(2)).unwrap();
    let wallet_acc_4 = contract.wallet(founder_account.clone()).unwrap();
    assert_wallet(
        &wallet_acc_1,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_2,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_3,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_4,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    let expected_role_founder = UserRoles::new()
        .add_group_roles(1, vec![0, 1])
        .add_group_roles(3, vec![0]);
    let expected_roles_artists = UserRoles::new().add_group_roles(3, vec![0]);
    assert_user_roles(&contract, accounts(0), Some(expected_roles_artists.clone()));
    assert_user_roles(&contract, accounts(1), Some(expected_roles_artists.clone()));
    assert_user_roles(&contract, accounts(2), Some(expected_roles_artists.clone()));
    assert_user_roles(
        &contract,
        founder_account.clone(),
        Some(expected_role_founder.clone()),
    );

    // 3. Add new account to the "artists" group.
    testing_env!(ctx.block_timestamp(tm(10)).build());
    let group = contract.group(3).unwrap();
    assert_group_members(
        &contract,
        3,
        vec![
            accounts(0),
            accounts(1),
            accounts(2),
            as_account_id(FOUNDER_1),
        ],
    );
    assert_eq!(contract.total_members_count as usize, 6 + members_count - 1);
    assert!(contract.wallet(accounts(3)).is_none());
    assert_user_roles(&contract, accounts(3), None);
    let expected_group_roles = Roles::new();
    assert_eq!(contract.group_roles(3).unwrap(), expected_group_roles);
    let added = contract.group_add_members(
        3,
        vec![GroupMember {
            account_id: accounts(3),
            tags: vec![],
        }],
        vec![MemberRoles {
            name: "eksmen".into(),
            members: vec![accounts(3)],
        }],
    );
    assert_eq!(
        contract.total_members_count as usize,
        6 + members_count - 1 + 1
    );
    let mut expected_group_roles = Roles::new();
    expected_group_roles.insert("eksmen".into());
    assert_eq!(contract.group_roles(3).unwrap(), expected_group_roles);
    let group = contract.group(3).unwrap();
    assert_group_members(
        &contract,
        3,
        vec![
            accounts(0),
            accounts(1),
            accounts(2),
            as_account_id(FOUNDER_1),
            accounts(3),
        ],
    );
    assert!(added);
    assert_user_roles(&contract, accounts(0), Some(expected_roles_artists.clone()));
    assert_user_roles(&contract, accounts(1), Some(expected_roles_artists.clone()));
    assert_user_roles(&contract, accounts(2), Some(expected_roles_artists.clone()));
    assert_user_roles(
        &contract,
        accounts(3),
        Some(expected_roles_artists.clone().add_role(3, 1)),
    );
    assert_user_roles(
        &contract,
        founder_account.clone(),
        Some(expected_role_founder.clone()),
    );
    let wallet_acc_1 = contract.wallet(accounts(0)).unwrap();
    let wallet_acc_2 = contract.wallet(accounts(1)).unwrap();
    let wallet_acc_3 = contract.wallet(accounts(2)).unwrap();
    let wallet_acc_4 = contract.wallet(founder_account.clone()).unwrap();
    let wallet_acc_5 = contract.wallet(accounts(3)).unwrap();
    assert_wallet(
        &wallet_acc_1,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_2,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_3,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_4,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_5,
        reward_id,
        vec![reward_asset.clone()],
        10,
        None,
    );
    let expected_group_members = vec![
        accounts(0),
        accounts(1),
        accounts(2),
        as_account_id(FOUNDER_1),
        accounts(3),
    ];
    assert_group_role_members(&contract, 3, 0, expected_group_members);
    let expected_group_members = vec![accounts(3)];
    assert_group_role_members(&contract, 3, 1, expected_group_members);

    // 4. Remove account(1) from the "artists" group.
    testing_env!(ctx.block_timestamp(tm(20)).build());
    assert_eq!(
        contract.total_members_count as usize,
        6 + members_count - 1 + 1
    );
    let removed = contract.group_remove_members(3, vec![accounts(1)]);
    assert!(removed);
    assert_eq!(
        contract.total_members_count as usize,
        6 + members_count - 1 + 1 - 1
    );
    let group = contract.group(3).unwrap();
    assert_group_members(
        &contract,
        3,
        vec![
            accounts(0),
            accounts(2),
            as_account_id(FOUNDER_1),
            accounts(3),
        ],
    );
    assert_user_roles(&contract, accounts(0), Some(expected_roles_artists.clone()));
    assert_user_roles(&contract, accounts(1), None);
    assert_user_roles(&contract, accounts(2), Some(expected_roles_artists.clone()));
    assert_user_roles(
        &contract,
        accounts(3),
        Some(expected_roles_artists.clone().add_role(3, 1)),
    );
    assert_user_roles(
        &contract,
        founder_account.clone(),
        Some(expected_role_founder),
    );
    let wallet_acc_1 = contract.wallet(accounts(0)).unwrap();
    let wallet_acc_2 = contract.wallet(accounts(1)).unwrap();
    let wallet_acc_3 = contract.wallet(accounts(2)).unwrap();
    let wallet_acc_4 = contract.wallet(founder_account.clone()).unwrap();
    let wallet_acc_5 = contract.wallet(accounts(3)).unwrap();
    assert_wallet(
        &wallet_acc_1,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_2,
        reward_id,
        vec![reward_asset.clone()],
        0,
        Some(20),
    );
    assert_wallet(
        &wallet_acc_3,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_4,
        reward_id,
        vec![reward_asset.clone()],
        0,
        None,
    );
    assert_wallet(
        &wallet_acc_5,
        reward_id,
        vec![reward_asset.clone()],
        10,
        None,
    );

    // 5. Remove the "artists" group.
    testing_env!(ctx.block_timestamp(tm(30)).build());
    assert_eq!(
        contract.total_members_count as usize,
        6 + members_count - 1 + 1 - 1
    );
    assert!(contract.group_roles.get(&3).is_some());
    contract.group_remove(3);
    assert!(contract.group_roles.get(&3).is_none());
    assert_eq!(contract.total_members_count as usize, 6);
    assert!(contract.group(3).is_none());
    assert_user_roles(&contract, accounts(0), None);
    assert_user_roles(&contract, accounts(1), None);
    assert_user_roles(&contract, accounts(2), None);
    assert_user_roles(&contract, accounts(3), None);
    assert_user_roles(
        &contract,
        founder_account.clone(),
        Some(UserRoles::new().add_group_roles(1, vec![0, 1])),
    );
    let wallet_acc_1 = contract.wallet(accounts(0)).unwrap();
    let wallet_acc_2 = contract.wallet(accounts(1)).unwrap();
    let wallet_acc_3 = contract.wallet(accounts(2)).unwrap();
    let wallet_acc_4 = contract.wallet(founder_account.clone()).unwrap();
    let wallet_acc_5 = contract.wallet(accounts(3)).unwrap();
    assert_wallet(
        &wallet_acc_1,
        reward_id,
        vec![reward_asset.clone()],
        0,
        Some(30),
    );
    assert_wallet(
        &wallet_acc_2,
        reward_id,
        vec![reward_asset.clone()],
        0,
        Some(20),
    );
    assert_wallet(
        &wallet_acc_3,
        reward_id,
        vec![reward_asset.clone()],
        0,
        Some(30),
    );
    assert_wallet(
        &wallet_acc_4,
        reward_id,
        vec![reward_asset.clone()],
        0,
        Some(30),
    );
    assert_wallet(
        &wallet_acc_5,
        reward_id,
        vec![reward_asset.clone()],
        10,
        Some(30),
    );

    // Bonus test: Withdraw rewards.
    let claimable_rewards = contract.claimable_rewards(accounts(0));
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        30
    );
    let withdraw_amount = contract.internal_withdraw_reward(&accounts(0), vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 30);
    let claimable_rewards = contract.claimable_rewards(accounts(1));
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        20
    );
    let withdraw_amount = contract.internal_withdraw_reward(&accounts(1), vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 20);
    let claimable_rewards = contract.claimable_rewards(accounts(3));
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        20
    );
    let withdraw_amount = contract.internal_withdraw_reward(&accounts(3), vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 20);
    let claimable_rewards = contract.claimable_rewards(founder_account.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        30
    );
    let withdraw_amount =
        contract.internal_withdraw_reward(&founder_account.clone(), vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 30);
}
