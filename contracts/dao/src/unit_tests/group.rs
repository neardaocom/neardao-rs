use std::collections::HashMap;

use library::workflow::types::{ActivityRight, VoteScenario};
use near_sdk::{testing_env, MockedBlockchain};

use crate::{
    group::{GroupInput, GroupMember, GroupSettings},
    unit_tests::{
        decimal_const, get_default_contract, FOUNDER_1, FOUNDER_2, FOUNDER_3, FOUNDER_4, FOUNDER_5,
    },
};

use super::get_context_builder;

#[test]
#[ignore]
fn add_group() {
    let ctx = get_context_builder();
    testing_env!(ctx.build());

    let mut contract = get_default_contract();

    assert_eq!(contract.total_members_count, 3);

    //let expected_group_names: Vec<String> = vec!["council".into()];
    //assert_eq!(contract.group_names(), expected_group_names);

    let expect_group_members = vec![
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

    let new_group_members = vec![
        GroupMember {
            account_id: FOUNDER_4.to_string().try_into().unwrap(),
            tags: vec![0],
        },
        GroupMember {
            account_id: FOUNDER_5.to_string().try_into().unwrap(),
            tags: vec![1],
        },
    ];

    contract.group_add(GroupInput {
        settings: GroupSettings {
            name: "council_rest".into(),
            leader: Some(FOUNDER_4.to_string().try_into().unwrap()),
            parent_group: 0,
        },
        members: new_group_members.clone(),
        member_roles: HashMap::default(),
    });

    assert_eq!(contract.total_members_count, 5);
}
