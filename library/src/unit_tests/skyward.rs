use std::convert::TryFrom;

use near_sdk::json_types::{
    ValidAccountId, WrappedBalance, WrappedDuration, WrappedTimestamp, U128,
};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::MockedBlockchain;
use near_sdk::{serde_json, testing_env, AccountId};

use crate::data::skyward::{
    workflow_skyward_template_data_1, workflow_skyward_template_settings_data_1, SKYWARD_ACC,
    WNEAR_ACC,
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

/******  Skyward scenario structures  ******/

#[derive(Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct RegisterTokensInput {
    token_account_ids: Vec<AccountId>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct StorageDepositInput {
    account_id: AccountId,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct FtTransferCallInput {
    memo: Option<String>,
    amount: U128,
    receiver_id: AccountId,
    msg: String,
}

type BasicPoints = u16;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleInput {
    pub title: String,
    pub url: Option<String>,
    pub permissions_contract_id: Option<ValidAccountId>,
    pub out_tokens: Vec<SaleInputOutToken>,
    pub in_token_account_id: ValidAccountId,
    pub start_time: WrappedTimestamp,
    pub duration: WrappedDuration,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleInputOutToken {
    pub token_account_id: ValidAccountId,
    pub balance: WrappedBalance,
    pub referral_bpt: Option<BasicPoints>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleCreateInput {
    pub sale: SaleInput,
}

#[test]
fn workflow_skyward_finance() {
    // need context coz storage_bucket
    let mut builder = VMContextBuilder::new();
    testing_env!(builder.build());

    let (wft, fncalls, fn_metadata) = workflow_skyward_template_data_1();
    let (wfs, settings) = workflow_skyward_template_settings_data_1();

    let mut wfi = Instance::new(1, &wft.transitions);
    let mut bucket = StorageBucket::new(b"wf_skyward".to_vec());
    wfi.state = InstanceState::Running;

    // 1. Register tokens

    // Execute Workflow
    let expected_args = vec![vec![DataType::VecString(vec![
        "neardao.testnet".into(),
        WNEAR_ACC.into(),
    ])]];
    let mut user_args = vec![vec![]];
    let mut user_args_collection = vec![];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_fncall(&wft, (SKYWARD_ACC.into(), "register_tokens".into()))
        .unwrap();

    assert_eq!(activity_id, 1);
    assert_eq!(transition_id, 0);

    let dao_consts = get_dao_consts();

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_fncall_id = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::FnCall(data) => data.id.clone(),
        _ => panic!("Invalid Data"),
    };

    // activity result check
    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 1);
    assert_eq!(actual_fncall_id, fncalls[activity_id as usize - 1]);

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        user_args_collection.as_slice(),
        fn_metadata[activity_id as usize - 1].as_slice(),
    ));

    // Fetch user value before binding
    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref();

    assert_eq!(pp, None);

    bind_args(
        &dao_consts,
        wft.binds.as_slice(),
        settings.binds.as_slice(),
        wft.activities[activity_id as usize]
            .as_ref()
            .unwrap()
            .activity_inputs
            .as_slice(),
        &mut bucket,
        &mut user_args,
        &mut user_args_collection,
        0,
        0,
    );

    assert_eq!(user_args, expected_args);

    let args = args_to_json(
        user_args.as_slice(),
        user_args_collection.as_slice(),
        &fn_metadata[activity_id as usize - 1],
        0,
    );

    let expected_obj = RegisterTokensInput {
        token_account_ids: vec!["neardao.testnet".into(), WNEAR_ACC.into()],
    };

    assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

    // 2. Storage deposit on self

    let expected_args = vec![vec![DataType::String(SKYWARD_ACC.into())]];
    let mut user_args = vec![vec![]];
    let mut user_args_collection = vec![];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_fncall(&wft, ("self".into(), "storage_deposit".into()))
        .unwrap();

    assert_eq!(activity_id, 2);
    assert_eq!(transition_id, 0);

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_fncall_id = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::FnCall(data) => data.id.clone(),
        _ => panic!("Invalid Data"),
    };

    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 2);
    assert_eq!(actual_fncall_id, fncalls[activity_id as usize - 1]);

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        user_args_collection.as_slice(),
        fn_metadata[activity_id as usize - 1].as_slice(),
    ));

    // Fetch user value before binding
    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref();

    assert_eq!(pp, None);

    bind_args(
        &dao_consts,
        wft.binds.as_slice(),
        settings.binds.as_slice(),
        wft.activities[activity_id as usize]
            .as_ref()
            .unwrap()
            .activity_inputs
            .as_slice(),
        &mut bucket,
        &mut user_args,
        &mut user_args_collection,
        0,
        0,
    );

    assert_eq!(user_args, expected_args);

    let args = args_to_json(
        user_args.as_slice(),
        user_args_collection.as_slice(),
        &fn_metadata[activity_id as usize - 1],
        0,
    );

    let expected_obj = StorageDepositInput {
        account_id: SKYWARD_ACC.into(),
    };

    assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

    // 3. Storage deposit wrap.near

    let expected_args = vec![vec![DataType::String(SKYWARD_ACC.into())]];
    let mut user_args = vec![vec![]];
    let mut user_args_collection = vec![];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_fncall(&wft, (WNEAR_ACC.into(), "storage_deposit".into()))
        .unwrap();

    assert_eq!(activity_id, 3);
    assert_eq!(transition_id, 0);

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_fncall_id = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::FnCall(data) => data.id.clone(),
        _ => panic!("Invalid Data"),
    };

    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 3);
    assert_eq!(actual_fncall_id, fncalls[activity_id as usize - 1]);

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        user_args_collection.as_slice(),
        fn_metadata[activity_id as usize - 1].as_slice(),
    ));

    // Fetch user value before binding
    let pp = wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .postprocessing
        .as_ref();

    assert_eq!(pp, None);

    bind_args(
        &dao_consts,
        wft.binds.as_slice(),
        settings.binds.as_slice(),
        wft.activities[activity_id as usize]
            .as_ref()
            .unwrap()
            .activity_inputs
            .as_slice(),
        &mut bucket,
        &mut user_args,
        &mut user_args_collection,
        0,
        0,
    );

    assert_eq!(user_args, expected_args);

    let args = args_to_json(
        user_args.as_slice(),
        user_args_collection.as_slice(),
        &fn_metadata[activity_id as usize - 1],
        0,
    );

    let expected_obj = StorageDepositInput {
        account_id: SKYWARD_ACC.into(),
    };

    assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

    // 4. FT transfer call on self
    // 1 NEAR
    let expected_args = vec![vec![
        DataType::String("memo msg".into()),
        DataType::U128(1000.into()),
        DataType::String(SKYWARD_ACC.into()),
        DataType::String("\\\"AccountDeposit\\\"".into()),
    ]];
    let mut user_args = vec![vec![DataType::String("memo msg".into())]];
    let mut user_args_collection = vec![];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_fncall(&wft, ("self".into(), "ft_transfer_call".into()))
        .unwrap();

    assert_eq!(activity_id, 4);
    assert_eq!(transition_id, 0);

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_fncall_id = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::FnCall(data) => data.id.clone(),
        _ => panic!("Invalid Data"),
    };

    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 4);
    assert_eq!(actual_fncall_id, fncalls[activity_id as usize - 1]);

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        user_args_collection.as_slice(),
        fn_metadata[activity_id as usize - 1].as_slice(),
    ));

    bind_args(
        &dao_consts,
        wft.binds.as_slice(),
        settings.binds.as_slice(),
        wft.activities[activity_id as usize]
            .as_ref()
            .unwrap()
            .activity_inputs
            .as_slice(),
        &mut bucket,
        &mut user_args,
        &mut user_args_collection,
        0,
        0,
    );

    assert_eq!(user_args, expected_args);

    let args = args_to_json(
        user_args.as_slice(),
        user_args_collection.as_slice(),
        &fn_metadata[activity_id as usize - 1],
        0,
    );

    let expected_obj = FtTransferCallInput {
        receiver_id: SKYWARD_ACC.into(),
        amount: 1000.into(),
        memo: Some("memo msg".into()),
        msg: "\"AccountDeposit\"".into(),
    };

    assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

    // 5. Sale create on skyward.near
    let mut user_args = vec![vec![], vec![]];
    let mut user_args_collection = vec![vec![DataType::Null, DataType::Null, DataType::Null]];

    let expected_args = vec![
        vec![DataType::Null],
        vec![
            DataType::String("NearDAO auction".into()),
            DataType::String("www.neardao.com".into()),
            DataType::String("neardao.testnet".into()),
            DataType::Null,
            DataType::String(WNEAR_ACC.into()),
            DataType::U64(1653304093000000000.into()),
            DataType::U64(604800000000000.into()),
        ],
    ];

    let expected_args_collection = vec![vec![
        DataType::String("neardao.testnet".into()),
        DataType::U128(1000.into()),
        DataType::Null,
    ]];

    let (transition_id, activity_id) = wfi
        .get_target_trans_with_for_fncall(&wft, (SKYWARD_ACC.into(), "sale_create".into()))
        .unwrap();

    assert_eq!(activity_id, 5);
    assert_eq!(transition_id, 0);

    let result = wfi.transition_to_next(
        activity_id,
        transition_id,
        &wft,
        &dao_consts,
        &wfs[0],
        &settings,
        &user_args,
        &mut bucket,
        0,
    );

    let actual_fncall_id = match &*wft.activities[activity_id as usize]
        .as_ref()
        .unwrap()
        .action_data
        .as_ref()
        .unwrap()
    {
        ActionData::FnCall(data) => data.id.clone(),
        _ => panic!("Invalid Data"),
    };

    assert_eq!(result.0, ActionResult::Ok);
    assert_eq!(wfi.current_activity_id, 5);
    assert_eq!(actual_fncall_id, fncalls[activity_id as usize - 1]);

    assert!(validate_args(
        &dao_consts,
        &settings.binds,
        &wft.obj_validators[activity_id as usize - 1].as_slice(),
        &wft.validator_exprs.as_slice(),
        &bucket,
        user_args.as_slice(),
        user_args_collection.as_slice(),
        fn_metadata[activity_id as usize - 1].as_slice(),
    ));

    bind_args(
        &dao_consts,
        wft.binds.as_slice(),
        settings.binds.as_slice(),
        wft.activities[activity_id as usize]
            .as_ref()
            .unwrap()
            .activity_inputs
            .as_slice(),
        &mut bucket,
        &mut user_args,
        &mut user_args_collection,
        0,
        0,
    );

    assert_eq!(user_args, expected_args);
    assert_eq!(user_args_collection, expected_args_collection);

    let out_tokens = SaleInputOutToken {
        token_account_id: ValidAccountId::try_from("neardao.testnet").unwrap(),
        balance: 1000.into(),
        referral_bpt: None,
    };

    let token_account_id: String = WNEAR_ACC.into();
    let sale_create_input = SaleInput {
        title: "NearDAO auction".into(),
        url: Some("www.neardao.com".into()),
        permissions_contract_id: Some(ValidAccountId::try_from("neardao.testnet").unwrap()),
        out_tokens: vec![out_tokens],
        in_token_account_id: ValidAccountId::try_from(token_account_id).unwrap(),
        start_time: 1653304093000000000.into(),
        duration: 604800000000000.into(),
    };

    let args = SaleCreateInput {
        sale: sale_create_input,
    };

    let result_json_string = args_to_json(
        user_args.as_slice(),
        user_args_collection.as_slice(),
        fn_metadata[activity_id as usize - 1].as_slice(),
        0,
    );
    let expected_json_string = serde_json::to_string(&args).unwrap();
    assert_eq!(result_json_string, expected_json_string);
    assert_eq!(
        serde_json::from_str::<SaleCreateInput>(&result_json_string).unwrap(),
        args
    );
}
