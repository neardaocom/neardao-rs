pub mod action;
pub mod activity;

pub mod error;
pub mod expression;
pub mod instance;
pub mod postprocessing;
pub mod runtime;
pub mod settings;
pub mod template;
pub mod types;
pub mod validator;

/*
#[cfg(test)]

mod test {
    use crate::{
        expression::{Condition, EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        storage::StorageBucket,
        utils::{bind_args, validate_args},
        workflow::{
            ActionResult, ActivityRight, ExprArg, InstanceState, PostprocessingType,
            ProposeSettings, TemplateActivity, TemplateSettings,
        },
        workflow::{
            ActionType, DataType, DataTypeDef, FnCallMetadata, ValidatorType, VoteScenario,
        },
    };

    use super::{ArgSrc, CondOrExpr, Expression, Instance, Postprocessing, Template};

    // PoC test case
    #[test]
    pub fn workflow_simple_1() {
        let pp = Some(Postprocessing {
            storage_key: "activity_1_postprocessing".into(),
            op_type: PostprocessingType::FnCallResult(DataTypeDef::String(false)),
            instructions: vec![],
        });

        let metadata: Vec<FnCallMetadata> = vec![];

        // Template
        let wft = Template {
            code: "send_near_and_create_group".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "near_send".into(),
                    exec_condition: None,
                    action: ActionType::TreasurySendNear,
                    action_data: None,
                    arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                    postprocessing: pp.clone(),
                    activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                    must_succeed: true,
                }),
                Some(TemplateActivity {
                    code: "group_add".into(),
                    exec_condition: None,
                    action: ActionType::GroupAdd,
                    action_data: None,
                    arg_types: vec![
                        DataTypeDef::String(false),
                        DataTypeDef::String(false),
                        DataTypeDef::String(false),
                    ],
                    postprocessing: pp.clone(),
                    activity_inputs: vec![vec![ArgSrc::Bind(0), ArgSrc::Free, ArgSrc::Free]],
                    must_succeed: true,
                }),
            ],
            obj_validators: vec![vec![], vec![ValidatorType::Simple(0)], vec![]],
            validator_exprs: vec![Expression {
                args: vec![ExprArg::User(1), ExprArg::Bind(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                }),
            }],
            transitions: vec![vec![1], vec![2]],
            binds: vec![],
            start: vec![0],
            end: vec![2],
        };

        //Template Settings example
        let wfs = TemplateSettings {
            transition_constraints: vec![
                vec![Transition {
                    transition_limit: 1,
                    cond: None,
                }],
                vec![Transition {
                    transition_limit: 1,
                    cond: None,
                }],
            ],
            activity_rights: vec![
                vec![ActivityRight::Group(1)],
                vec![ActivityRight::GroupLeader(1)],
            ],
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::TokenHolder,
            scenario: VoteScenario::TokenWeighted,
            duration: 3600,
            quorum: 51,
            approve_threshold: 20,
            spam_threshold: 80,
            vote_only_once: true,
            deposit_propose: Some(1.into()),
            deposit_vote: Some(1000.into()),
            deposit_propose_return: 0,
        };

        //User proposed settings type
        let settings = ProposeSettings {
            binds: vec![
                DataType::String("rustaceans_group".into()),
                DataType::U8(100),
            ],
            storage_key: "wf_simple".into(),
        };

        let mut wfi = Instance::new(1, &wft.transitions);
        let mut bucket = StorageBucket::new(b"simple_wf".to_vec());

        // Execute Workflow
        let expected_args = vec![vec![
            DataType::String("jonnyis.near".into()),
            DataType::U8(50),
        ]];
        let mut user_args = expected_args.clone();
        let mut user_args_collection = vec![];

        wfi.state = InstanceState::Running;

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_for_dao_action(&wft, ActionType::TreasurySendNear)
            .unwrap();

        assert_eq!(activity_id, 1);
        assert_eq!(transition_id, 0);

        let dao_consts = Box::new(|id: u8| match id {
            0 => DataType::String("neardao.near".into()),
            _ => unimplemented!(),
        });

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );

        assert!(validate_args(
            &dao_consts,
            &settings.binds,
            &wft.obj_validators[activity_id as usize].as_slice(),
            &wft.validator_exprs.as_slice(),
            &bucket,
            user_args.as_slice(),
            user_args_collection.as_slice(),
            metadata.as_slice(),
        ));

        let expected_result = (ActionResult::Ok, pp.clone());

        assert_eq!(result, expected_result);
        assert_eq!(wfi.current_activity_id, 1);

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

        // 2. action
        let mut user_args = vec![vec![
            DataType::String("rustlovers_group".into()),
            DataType::String("leaderisme".into()),
            DataType::String("user_provided_settings".into()),
        ]];

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_for_dao_action(&wft, ActionType::GroupAdd)
            .unwrap();

        assert_eq!(activity_id, 2);
        assert_eq!(transition_id, 0);

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            &user_args,
            &bucket,
            0,
        );

        assert!(validate_args(
            &dao_consts,
            &settings.binds,
            &wft.obj_validators[activity_id as usize].as_slice(),
            &wft.validator_exprs.as_slice(),
            &bucket,
            user_args.as_slice(),
            user_args_collection.as_slice(),
            metadata.as_slice(),
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

        let expected_result = (ActionResult::Ok, pp.clone());
        let expected_args = vec![vec![
            DataType::String("rustaceans_group".into()),
            DataType::String("leaderisme".into()),
            DataType::String("user_provided_settings".into()),
        ]];

        assert_eq!(result, expected_result);
        assert_eq!(user_args, expected_args);
        assert_eq!(wfi.current_activity_id, 2);
    }

    #[test]
    fn workflow_simple_loop() {
        let pp = Some(Postprocessing {
            storage_key: "activity_1_postprocessing".into(),
            op_type: PostprocessingType::FnCallResult(DataTypeDef::String(false)),
            instructions: vec![],
        });

        // Template
        let wft = Template {
            code: "send_near_in_loop".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "near_send".into(),
                    exec_condition: None,
                    action: ActionType::TreasurySendNear,
                    action_data: None,
                    arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                    postprocessing: pp.clone(),
                    activity_inputs: vec![vec![ArgSrc::Free, ArgSrc::Free]],
                    must_succeed: true,
                }),
            ],
            obj_validators: vec![vec![], vec![ValidatorType::Simple(0)], vec![]],
            validator_exprs: vec![Expression {
                args: vec![ExprArg::User(1), ExprArg::Bind(0)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                }),
            }],
            transitions: vec![vec![1], vec![1]],
            binds: vec![],
            start: vec![0],
            end: vec![1],
        };

        //Template Settings example
        let wfs = TemplateSettings {
            transition_constraints: vec![
                vec![Transition {
                    transition_limit: 1,
                    cond: None,
                }],
                vec![Transition {
                    transition_limit: 4,
                    cond: None,
                }],
            ],
            activity_rights: vec![
                vec![],
                vec![ActivityRight::Group(1)],
                vec![ActivityRight::GroupLeader(1)],
            ],
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::TokenHolder,
            scenario: VoteScenario::TokenWeighted,
            duration: 3600,
            quorum: 51,
            approve_threshold: 20,
            spam_threshold: 80,
            vote_only_once: true,
            deposit_propose: Some(1.into()),
            deposit_vote: Some(1000.into()),
            deposit_propose_return: 0,
        };

        //User proposed settings type
        let settings = ProposeSettings {
            binds: vec![DataType::U8(100)],
            storage_key: "wf_simple_loop".into(),
        };

        let mut wfi = Instance::new(1, &wft.transitions);
        let mut bucket = StorageBucket::new(b"simple_wf".to_vec());
        let expected_args = vec![vec![
            DataType::String("jonnyis.near".into()),
            DataType::U8(50),
        ]];
        let mut user_args = expected_args.clone();
        let mut user_args_collection: Vec<Vec<DataType>> = vec![];

        let metadata: Vec<FnCallMetadata> = vec![];
        let dao_consts = Box::new(|id: u8| match id {
            0 => DataType::String("neardao.near".into()),
            _ => unimplemented!(),
        });

        wfi.state = InstanceState::Running;

        // Execute Workflow
        for i in 0..5 {
            let (transition_id, activity_id) = wfi
                .get_target_trans_with_for_dao_action(&wft, ActionType::TreasurySendNear)
                .unwrap();

            assert_eq!(activity_id, 1);
            assert_eq!(transition_id, 0);

            let result = wfi.transition_to_next(
                activity_id,
                transition_id,
                &wft,
                &dao_consts,
                &wfs,
                &settings,
                &user_args,
                &mut bucket,
                0,
            );
            let expected_result = (ActionResult::Ok, pp.clone());

            assert!(validate_args(
                &dao_consts,
                &settings.binds,
                &wft.obj_validators[activity_id as usize].as_slice(),
                &wft.validator_exprs.as_slice(),
                &bucket,
                user_args.as_slice(),
                user_args_collection.as_slice(),
                metadata.as_slice(),
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

            assert_eq!(result, expected_result);
            assert_eq!(user_args, expected_args);
            assert_eq!(wfi.current_activity_id, 1);
            assert_eq!(wfi.transition_counter[0][0], 1);
            assert_eq!(wfi.transition_counter[1][0], i);
        }

        let (transition_id, activity_id) = wfi
            .get_target_trans_with_for_dao_action(&wft, ActionType::TreasurySendNear)
            .unwrap();

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &dao_consts,
            &wfs,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );

        let expected_args = vec![vec![
            DataType::String("jonnyis.near".into()),
            DataType::U8(50),
        ]];

        let mut user_args = expected_args.clone();
        let expected_result = (ActionResult::MaxTransitionLimitReached, None);

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

        assert_eq!(result, expected_result);
        assert_eq!(user_args, expected_args);
        assert_eq!(wfi.transition_counter[0][0], 1);
        assert_eq!(wfi.transition_counter[1][0], 4);
    }

    #[test]
    fn postprocessing_with_cond() {
        // FnCall result > 5 then 20 else 40
        let input: Vec<DataType> = vec![DataType::U8(1)];
        let postprocessing = Postprocessing {
            storage_key: "key".into(),
            op_type: PostprocessingType::FnCallResult(DataTypeDef::String(false)),
            instructions: vec![
                CondOrExpr::Cond(Condition {
                    expr: EExpr::Boolean(TExpr {
                        operators: vec![Op {
                            op_type: EOp::Rel(RelOp::Gt),
                            operands_ids: [0, 1],
                        }],
                        terms: vec![ExprTerm::Arg(0), ExprTerm::Value(DataType::U8(5))],
                    }),
                    true_path: 1,
                    false_path: 2,
                }),
                CondOrExpr::Expr(EExpr::Value(DataType::U8(20))),
                CondOrExpr::Expr(EExpr::Value(DataType::U8(40))),
            ],
        };

        let result = postprocessing.process(input.as_slice());
        let expected_result = 40;

        if let DataType::U8(v) = result {
            assert_eq!(v, expected_result);
        } else {
            panic!("expected DataType::U8");
        }
    }
}
 */
