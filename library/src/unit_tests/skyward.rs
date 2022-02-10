mod test {
    use std::convert::TryFrom;

    use near_sdk::json_types::{
        ValidAccountId, WrappedBalance, WrappedDuration, WrappedTimestamp, U128,
    };
    use near_sdk::serde::{Deserialize, Serialize};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::MockedBlockchain;
    use near_sdk::{serde_json, testing_env, AccountId};

    use crate::data::skyward::{
        workflow_skyward_template_data_1, workflow_skyward_template_settings_data_1,
    };

    use crate::unit_tests::{get_dao_consts, ONE_NEAR};
    use crate::utils::{args_to_json, bind_args, validate_args};
    use crate::workflow::ActivityResult;
    use crate::{
        storage::StorageBucket,
        types::{ActionIdent, DataType},
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
        amount: U128,
        memo: Option<String>,
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
        let mut bucket = StorageBucket::new(b"simple_wf".to_vec());
        wfi.state = InstanceState::Running;

        // 1. Register tokens

        // Execute Workflow
        let expected_args = vec![vec![DataType::VecString(vec![
            "neardao.near".into(),
            "wrap.near".into(),
        ])]];
        let mut user_args = vec![vec![DataType::String("wrap.near".into())]];
        let mut user_args_collection = vec![];

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_act(
                &wft,
                ActionIdent::FnCall,
                Some(("skyward.near".into(), "register_tokens".into())),
            )
            .unwrap();

        assert_eq!(activity_id, 1);
        assert_eq!(transition_id, 0);

        let dao_consts = get_dao_consts();

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );

        // activity result check
        assert_eq!(result.0, ActivityResult::Ok);
        assert_eq!(wfi.current_activity_id, 1);
        assert_eq!(
            *wft.activities[activity_id as usize]
                .as_ref()
                .unwrap()
                .fncall_id
                .as_ref()
                .unwrap(),
            fncalls[activity_id as usize - 1]
        );

        assert!(validate_args(
            &dao_consts,
            &settings.binds,
            &settings.obj_validators[activity_id as usize - 1].as_slice(),
            &settings.validator_exprs.as_slice(),
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
            .as_ref()
            .unwrap();

        // Save wrap.near to storage so activity storage_deposit and sale_create get the right values
        let user_value = pp.try_to_get_user_value(user_args.as_slice()).unwrap();
        assert_eq!(user_value, DataType::String("wrap.near".into()));

        bucket.add_data(&pp.storage_key, &user_value);

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize - 1].as_slice(),
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
            token_account_ids: vec!["neardao.near".into(), "wrap.near".into()],
        };

        assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

        // 2. Storage deposit on self

        let expected_args = vec![vec![DataType::String("skyward.near".into())]];
        let mut user_args = vec![vec![]];
        let mut user_args_collection = vec![];

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_act(
                &wft,
                ActionIdent::FnCall,
                Some(("self".into(), "storage_deposit".into())),
            )
            .unwrap();

        assert_eq!(activity_id, 2);
        assert_eq!(transition_id, 0);

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );
        assert_eq!(result.0, ActivityResult::Ok);
        assert_eq!(wfi.current_activity_id, 2);
        assert_eq!(
            *wft.activities[activity_id as usize]
                .as_ref()
                .unwrap()
                .fncall_id
                .as_ref()
                .unwrap(),
            fncalls[activity_id as usize - 1]
        );

        assert!(validate_args(
            &dao_consts,
            &settings.binds,
            &settings.obj_validators[activity_id as usize - 1].as_slice(),
            &settings.validator_exprs.as_slice(),
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
            .as_ref()
            .unwrap();

        let user_value = pp.try_to_get_user_value(user_args.as_slice());
        assert_eq!(user_value, None);

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize - 1].as_slice(),
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
            account_id: "skyward.near".into(),
        };

        assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

        // Assuming storage deposit fn call result was ok
        let user_value = pp.clone().postprocess(vec![], user_value);
        assert_eq!(user_value, DataType::Bool(true));

        bucket.add_data(&pp.storage_key, &user_value);

        // 3. Storage deposit wrap.near

        let expected_args = vec![vec![DataType::String("skyward.near".into())]];
        let mut user_args = vec![vec![]];
        let mut user_args_collection = vec![];

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_act(
                &wft,
                ActionIdent::FnCall,
                Some(("wrap.near".into(), "storage_deposit".into())),
            )
            .unwrap();

        assert_eq!(activity_id, 3);
        assert_eq!(transition_id, 0);

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );
        assert_eq!(result.0, ActivityResult::Ok);
        assert_eq!(wfi.current_activity_id, 3);
        assert_eq!(
            *wft.activities[activity_id as usize]
                .as_ref()
                .unwrap()
                .fncall_id
                .as_ref()
                .unwrap(),
            fncalls[activity_id as usize - 1]
        );

        assert!(validate_args(
            &dao_consts,
            &settings.binds,
            &settings.obj_validators[activity_id as usize - 1].as_slice(),
            &settings.validator_exprs.as_slice(),
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
            .as_ref()
            .unwrap();

        let user_value = pp.try_to_get_user_value(user_args.as_slice());
        assert_eq!(user_value, None);

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize - 1].as_slice(),
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
            account_id: "skyward.near".into(),
        };

        assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

        // Assuming storage deposit fn call result was ok
        let user_value = pp.clone().postprocess(vec![], user_value);
        assert_eq!(user_value, DataType::Bool(true));

        bucket.add_data(&pp.storage_key, &user_value);

        // 4. FT transfer call on self
        // 1 NEAR
        let expected_args = vec![vec![
            DataType::U128(ONE_NEAR.into()),
            DataType::String("memo msg".into()),
            DataType::String("skyward.near".into()),
            DataType::String("\\\"AccountDeposit\\\"".into()),
        ]];
        let mut user_args = vec![vec![
            DataType::U128(ONE_NEAR.into()),
            DataType::String("memo msg".into()),
        ]];
        let mut user_args_collection = vec![];

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_act(
                &wft,
                ActionIdent::FnCall,
                Some(("self".into(), "ft_transfer_call".into())),
            )
            .unwrap();

        assert_eq!(activity_id, 4);
        assert_eq!(transition_id, 0);

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );
        assert_eq!(result.0, ActivityResult::Ok);
        assert_eq!(wfi.current_activity_id, 4);
        assert_eq!(
            *wft.activities[activity_id as usize]
                .as_ref()
                .unwrap()
                .fncall_id
                .as_ref()
                .unwrap(),
            fncalls[activity_id as usize - 1]
        );

        assert!(validate_args(
            &dao_consts,
            &settings.binds,
            &settings.obj_validators[activity_id as usize - 1].as_slice(),
            &settings.validator_exprs.as_slice(),
            &bucket,
            user_args.as_slice(),
            user_args_collection.as_slice(),
            fn_metadata[activity_id as usize - 1].as_slice(),
        ));

        // Postprocess before binding
        let pp = wft.activities[activity_id as usize]
            .as_ref()
            .unwrap()
            .postprocessing
            .as_ref()
            .unwrap();

        let user_value = pp.try_to_get_user_value(user_args.as_slice());
        assert_eq!(user_value, Some(DataType::U128(ONE_NEAR.into())));

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize - 1].as_slice(),
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
            receiver_id: "skyward.near".into(),
            amount: ONE_NEAR.into(),
            memo: Some("memo msg".into()),
            msg: "\"AccountDeposit\"".into(),
        };

        assert_eq!(args, serde_json::to_string(&expected_obj).unwrap());

        // Assuming storage deposit fn call result was ok
        let user_value = pp.clone().postprocess(vec![], user_value);
        assert_eq!(user_value, DataType::U128(ONE_NEAR.into()));

        bucket.add_data(&pp.storage_key, &user_value);

        // 5. Sale create on skyward.near
        let mut user_args = vec![
            vec![DataType::Null],
            vec![
                DataType::String("Neardao token auction".into()),
                DataType::String("www.neardao.com".into()),
                DataType::String("neardao.testnet".into()),
                DataType::Null,
                DataType::String("wrap.near".into()),
                DataType::U64(0.into()),
                DataType::U64(3600.into()),
            ],
        ];
        let mut user_args_collection = vec![vec![
            DataType::String("neardao.testnet".into()),
            DataType::U128((2 * ONE_NEAR).into()),
            DataType::Null,
        ]];

        let expected_args = vec![
            vec![DataType::Null],
            vec![
                DataType::String("Neardao token auction".into()),
                DataType::String("www.neardao.com".into()),
                DataType::String("neardao.near".into()),
                DataType::Null,
                DataType::String("wrap.near".into()),
                DataType::U64(0.into()),
                DataType::U64(3600.into()),
            ],
        ];

        let expected_args_collection = vec![vec![
            DataType::String("neardao.near".into()),
            DataType::U128(ONE_NEAR.into()),
            DataType::Null,
        ]];

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_act(
                &wft,
                ActionIdent::FnCall,
                Some(("skyward.near".into(), "sale_create".into())),
            )
            .unwrap();

        assert_eq!(activity_id, 5);
        assert_eq!(transition_id, 0);

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );
        assert_eq!(result.0, ActivityResult::Ok);
        assert_eq!(wfi.current_activity_id, 5);
        assert_eq!(
            *wft.activities[activity_id as usize]
                .as_ref()
                .unwrap()
                .fncall_id
                .as_ref()
                .unwrap(),
            fncalls[activity_id as usize - 1]
        );

        assert!(validate_args(
            &dao_consts,
            &settings.binds,
            &settings.obj_validators[activity_id as usize - 1].as_slice(),
            &settings.validator_exprs.as_slice(),
            &bucket,
            user_args.as_slice(),
            user_args_collection.as_slice(),
            fn_metadata[activity_id as usize - 1].as_slice(),
        ));

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize - 1].as_slice(),
            &mut bucket,
            &mut user_args,
            &mut user_args_collection,
            0,
            0,
        );

        assert_eq!(user_args, expected_args);
        assert_eq!(user_args_collection, expected_args_collection);

        let out_tokens = SaleInputOutToken {
            token_account_id: ValidAccountId::try_from("neardao.near").unwrap(),
            balance: ONE_NEAR.into(),
            referral_bpt: None,
        };
        let sale_create_input = SaleInput {
            title: "Neardao token auction".into(),
            url: Some("www.neardao.com".into()),
            permissions_contract_id: Some(ValidAccountId::try_from("neardao.near").unwrap()),
            out_tokens: vec![out_tokens],
            in_token_account_id: ValidAccountId::try_from("wrap.near").unwrap(),
            start_time: 0.into(),
            duration: 3600.into(),
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
}
