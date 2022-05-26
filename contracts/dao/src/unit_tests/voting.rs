use std::collections::HashMap;

use library::workflow::types::{ActivityRight, VoteScenario};
use near_sdk::AccountId;
use near_sdk::{testing_env, MockedBlockchain};

use super::get_context_builder;
use crate::unit_tests::get_role_id;
use crate::{
    unit_tests::{
        as_account_id, decimal_const, dummy_propose_settings, dummy_template_settings,
        get_default_contract, update_template_settings_vote_rights, ACC_1, ACC_2, ACC_3, FOUNDER_1,
        FOUNDER_2, FOUNDER_3, FOUNDER_4, FOUNDER_5, STAKING_ACC,
    },
    Proposal,
};

use near_sdk::ONE_NEAR;

// This is ugly. Refactoring is welcomed.
macro_rules! test_voting {
    ($fn_name:ident; $target:expr, TokenWeighted; $($name:expr => $vote:literal,$tokens:expr)*; TOTAL: $sum:expr, SPAM: $spam:expr, YES: $yes:expr, NO: $no:expr) => {
        #[test]
        fn $fn_name() {
                let mut ctx = get_context_builder();
                testing_env!(ctx.build());
                let mut contract = get_default_contract();
                update_template_settings_vote_rights(&mut contract, 1, 0, $target);
                testing_env!(ctx
                    .predecessor_account_id(as_account_id(FOUNDER_1))
                    .attached_deposit(ONE_NEAR)
                    .build());
                let proposal_id = contract.proposal_create(
                    None,
                    1,
                    0,
                    dummy_propose_settings(),
                    Some(vec![dummy_template_settings()]),
                    None,
                );
                $(
                    testing_env!(ctx.predecessor_account_id(as_account_id(STAKING_ACC)).build());
                    contract.register_delegation(as_account_id($name));
                    contract.delegate_owned(as_account_id($name), $tokens.into());
                    testing_env!(ctx.predecessor_account_id(as_account_id($name.clone())).attached_deposit(1).build());
                    contract.proposal_vote(proposal_id, $vote);
                )*
                let proposal: Proposal = contract.proposals.get(&proposal_id).unwrap().into();
                let vote_target = $target;
                let scenario = VoteScenario::TokenWeighted;
                let vote_result = contract.calculate_votes(&proposal.votes, &scenario, &vote_target);
                let expected_result = ($sum, [$spam, $yes, $no]);
                assert_eq!(vote_result, expected_result);
        }
    };
    ($fn_name:ident; $target:expr, Democratic; $($name:expr => $vote:literal,$tokens:expr)*; TOTAL: $sum:expr, SPAM: $spam:expr, YES: $yes:expr, NO: $no:expr) => {
        #[test]
        fn $fn_name() {
                let mut ctx = get_context_builder();
                testing_env!(ctx.build());
                let mut contract = get_default_contract();
                update_template_settings_vote_rights(&mut contract, 1, 0, $target);
                testing_env!(ctx
                    .predecessor_account_id(as_account_id(FOUNDER_1))
                    .attached_deposit(ONE_NEAR)
                    .build());
                let proposal_id = contract.proposal_create(
                    None,
                    1,
                    0,
                    dummy_propose_settings(),
                    Some(vec![dummy_template_settings()]),
                    None,
                );
                $(
                    testing_env!(ctx.predecessor_account_id(as_account_id(STAKING_ACC)).build());
                    contract.register_delegation(as_account_id($name));
                    contract.delegate_owned(as_account_id($name), $tokens.into());
                    testing_env!(ctx.predecessor_account_id(as_account_id($name.clone())).attached_deposit(1).build());
                    contract.proposal_vote(proposal_id, $vote);
                )*
                let proposal: Proposal = contract.proposals.get(&proposal_id).unwrap().into();
                let vote_target = $target;
                let scenario = VoteScenario::Democratic;
                let vote_result = contract.calculate_votes(&proposal.votes, &scenario, &vote_target);
                let expected_result = ($sum, [$spam, $yes, $no]);
                assert_eq!(vote_result, expected_result);
        }
    };
}

// ----- SCENARIO: DEMOCRATIC -----
test_voting!(
    voting_democratic_anyone;
    ActivityRight::Anyone, Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 6,
    SPAM: 0,
    YES: 3,
    NO: 3
);
test_voting!(
    voting_democratic_tokenholder;
    ActivityRight::TokenHolder, Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 5,
    SPAM: 0,
    YES: 3,
    NO: 2
);
test_voting!(
    voting_democratic_tokenholder_no_delegation;
    ActivityRight::TokenHolder, Democratic;
    FOUNDER_1 => 1,0 FOUNDER_2 => 1,0 FOUNDER_3 => 2,0  "guest_1.testnet" => 1,0 "guest_2.testnet" => 2,0 "guest_3.testnet" => 2,0;
    TOTAL: 0,
    SPAM: 0,
    YES: 0,
    NO: 0
);
test_voting!(
    voting_democratic_group_1;
    ActivityRight::Group(1), Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 3,
    SPAM: 0,
    YES: 2,
    NO: 1
);
test_voting!(
    voting_democratic_group_1_member;
    ActivityRight::GroupMember(1, as_account_id(FOUNDER_2)), Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 1,
    SPAM: 0,
    YES: 1,
    NO: 0
);
test_voting!(
    voting_democratic_account;
    ActivityRight::Account(as_account_id("guest_1.testnet")), Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 1,
    SPAM: 0,
    YES: 1,
    NO: 0
);
test_voting!(
    voting_democratic_group_1_leader;
    ActivityRight::GroupLeader(1), Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 1,
    SPAM: 0,
    YES: 1,
    NO: 0
);
test_voting!(
    voting_democratic_group_1_role_leader;
    ActivityRight::GroupRole(1, 1), Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 1,
    SPAM: 0,
    YES: 1,
    NO: 0
);
test_voting!(
    voting_democratic_member;
    ActivityRight::Member, Democratic;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0 ACC_1 => 1,1;
    TOTAL: 6,
    SPAM: 0,
    YES: 3,
    NO: 1
);

// ----- SCENARIO: TOKEN WEIGHTED -----
test_voting!(
    voting_tokenweighted_anyone_big_int;
    ActivityRight::Anyone, TokenWeighted;
    FOUNDER_1 => 1,1_000_000_000 * ONE_NEAR FOUNDER_2 => 1,1_000_000_000 * ONE_NEAR FOUNDER_3 => 2,1_000_000_000 * ONE_NEAR  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 3_000_000_000 * ONE_NEAR + 3,
    SPAM: 0,
    YES: 2_000_000_000 * ONE_NEAR + 1,
    NO:  1_000_000_000 * ONE_NEAR + 2
);
test_voting!(
    voting_tokenweighted_anyone;
    ActivityRight::Anyone, TokenWeighted;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 337,
    SPAM: 0,
    YES: 334,
    NO: 3
);
test_voting!(
    voting_tokenweighted_group_1;
    ActivityRight::Group(1), TokenWeighted;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 334,
    SPAM: 0,
    YES: 333,
    NO: 1
);
test_voting!(
    voting_tokenweighted_group_1_leader;
    ActivityRight::GroupLeader(1), TokenWeighted;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 111,
    SPAM: 0,
    YES: 111,
    NO: 0
);
test_voting!(
    voting_tokenweighted_group_1_member;
    ActivityRight::GroupMember(1, as_account_id(FOUNDER_2)), TokenWeighted;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 222,
    SPAM: 0,
    YES: 222,
    NO: 0
);
test_voting!(
    voting_tokenweighted_group_1_role_leader;
    ActivityRight::GroupRole(1, 1), TokenWeighted;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 111,
    SPAM: 0,
    YES: 111,
    NO: 0
);
test_voting!(
    voting_tokenweighted_account;
    ActivityRight::Account(as_account_id("guest_1.testnet")), TokenWeighted;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 1,
    SPAM: 0,
    YES: 1,
    NO: 0
);
test_voting!(
    voting_tokenweighted_tokenholder;
    ActivityRight::TokenHolder, TokenWeighted;
    FOUNDER_1 => 1,111 FOUNDER_2 => 1,222 FOUNDER_3 => 2,1  "guest_1.testnet" => 1,1 "guest_2.testnet" => 2,2 "guest_3.testnet" => 2,0;
    TOTAL: 337,
    SPAM: 0,
    YES: 334,
    NO: 3
);
test_voting!(
    voting_tokenweighted_tokenholder_no_delegation;
    ActivityRight::TokenHolder, TokenWeighted;
    FOUNDER_1 => 1,0 FOUNDER_2 => 1,0 FOUNDER_3 => 2,0  "guest_1.testnet" => 1,0 "guest_2.testnet" => 2,0 "guest_3.testnet" => 2,0;
    TOTAL: 0,
    SPAM: 0,
    YES: 0,
    NO: 0
);
