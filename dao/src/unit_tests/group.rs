use std::collections::HashMap;

use library::workflow::types::{ActivityRight, VoteScenario};
use near_sdk::{testing_env, MockedBlockchain};

use crate::{
    group::{GroupInput, GroupMember, GroupSettings, GroupTokenLockInput},
    unit_tests::{
        decimal_const, get_default_contract, FOUNDER_1, FOUNDER_2, FOUNDER_3, FOUNDER_4, FOUNDER_5,
    },
};

use super::get_context_builder;

#[test]
fn add_group() {
    let ctx = get_context_builder();
    testing_env!(ctx.build());

    let mut contract = get_default_contract();

    assert_eq!(contract.total_members_count, 3);

    let expected_group_names: Vec<String> = vec!["council".into()];
    assert_eq!(contract.group_names(), expected_group_names);

    let expect_group_members = vec![
        GroupMember {
            account_id: FOUNDER_1.into(),
            tags: vec![0],
        },
        GroupMember {
            account_id: FOUNDER_2.into(),
            tags: vec![1],
        },
        GroupMember {
            account_id: FOUNDER_3.into(),
            tags: vec![2],
        },
    ];

    let mut actual_group_members = contract.group_members(1).unwrap();
    actual_group_members.sort();
    assert_eq!(actual_group_members, expect_group_members);

    let new_group_members = vec![
        GroupMember {
            account_id: FOUNDER_4.into(),
            tags: vec![0],
        },
        GroupMember {
            account_id: FOUNDER_5.into(),
            tags: vec![1],
        },
    ];

    contract.add_group(GroupInput {
        settings: GroupSettings {
            name: "council_rest".into(),
            leader: Some(FOUNDER_4.into()),
        },
        members: new_group_members.clone(),
        token_lock: Some(GroupTokenLockInput {
            amount: 0,
            start_from: 0,
            duration: 0,
            init_distribution: 0,
            unlock_interval: 0,
            periods: vec![],
        }),
    });

    assert_eq!(contract.total_members_count, 5);

    let expected_group_names: Vec<String> = vec!["council".into(), "council_rest".into()];

    let mut actual_group_names = contract.group_names();
    actual_group_names.sort();
    assert_eq!(actual_group_names, expected_group_names);

    let mut actual_group_members = contract.group_members(2).unwrap();
    actual_group_members.sort();
    assert_eq!(actual_group_members, new_group_members);
}