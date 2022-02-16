use std::convert::TryFrom;

use near_sdk::json_types::{
    ValidAccountId, WrappedBalance, WrappedDuration, WrappedTimestamp, U128,
};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::MockedBlockchain;
use near_sdk::{serde_json, testing_env, AccountId};

use crate::data::bounty::{
    workflow_bounty_template_data_1, workflow_bounty_template_settings_data_1,
};

use crate::types::ActionData;
use crate::unit_tests::{get_dao_consts, ONE_NEAR};
use crate::utils::{args_to_json, bind_args, validate_args};
use crate::workflow::ActionResult;
use crate::{
    storage::StorageBucket,
    types::DataType,
    workflow::{Instance, InstanceState},
};

#[test]
fn workflow_bounty_happy_scenario() {
    let mut builder = VMContextBuilder::new();
    testing_env!(builder.build());

    let (wft, fncalls, fn_metadata) = workflow_bounty_template_data_1();
    let (wfs, settings) = workflow_bounty_template_settings_data_1();

    let mut wfi = Instance::new(1, &wft.transitions);
    let mut bucket = StorageBucket::new(b"wf_bounty".to_vec());
    wfi.state = InstanceState::Running;
}
