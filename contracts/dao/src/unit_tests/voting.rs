use std::collections::HashMap;

use library::workflow::types::{ActivityRight, VoteScenario};
use near_sdk::AccountId;
use near_sdk::{testing_env, MockedBlockchain};

use crate::unit_tests::{
    decimal_const, get_default_contract, FOUNDER_1, FOUNDER_2, FOUNDER_3, FOUNDER_4, FOUNDER_5,
};

use super::get_context_builder;

macro_rules! test_voting {
    ($fn_name:ident; $target:ident, TokenWeighted; $($name:expr => $vote:literal)*; TOTAL: $sum:expr, SPAM: $spam:expr, YES: $yes:expr, NO: $no:expr) => {
        #[test]
        fn $fn_name() {
                let ctx = get_context_builder();
                testing_env!(ctx.build());

                let contract = get_default_contract();

                let votes = HashMap::from([
                $(
                    (AccountId::try_from($name.to_string()).unwrap(), $vote),
                )*

                ]);
                let vote_target = ActivityRight::$target;
                let scenario = VoteScenario::TokenWeighted;

                let vote_result = contract.calculate_votes(&votes, &scenario, &vote_target);
                let expected_result = ($sum * decimal_const(), [$spam * decimal_const(), $yes * decimal_const(), $no * decimal_const()]);

                assert_eq!(vote_result, expected_result);
        }
    };

    ($fn_name:ident; $target:ident, Democratic; $($name:expr => $vote:literal)*; TOTAL: $sum:expr, SPAM: $spam:expr, YES: $yes:expr, NO: $no:expr) => {
        #[test]
        fn $fn_name() {
                let ctx = get_context_builder();
                testing_env!(ctx.build());

                let contract = get_default_contract();

                let votes = HashMap::from([
                $(
                    (AccountId::try_from($name.to_string()).unwrap(), $vote),
                )*

                ]);
                let vote_target = ActivityRight::$target;
                let scenario = VoteScenario::Democratic;

                let vote_result = contract.calculate_votes(&votes, &scenario, &vote_target);
                let expected_result = ($sum, [$spam, $yes, $no]);

                assert_eq!(vote_result, expected_result);
        }
    };
}
/* test_voting!(
    democratic_tokenholder;
    TokenHolder, Democratic;
    FOUNDER_1 => 1 FOUNDER_2 => 1 FOUNDER_3 => 2;
    TOTAL: 3,
    SPAM: 0,
    YES: 2,
    NO: 1
);
test_voting!(
    tokenweighted_tokenholder;
    TokenHolder, TokenWeighted;
    FOUNDER_1 => 1 FOUNDER_2 => 1 FOUNDER_3 => 2;
    TOTAL: 9_999_999,
    SPAM: 0,
    YES: 6_666_666,
    NO: 3_333_333
); */
