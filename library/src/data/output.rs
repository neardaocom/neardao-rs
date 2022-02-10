



/********************  WF DATA FOR PROVIDER  ********************/




mod test {
    use crate::{
        expression::{EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
        types::{ActionIdent, DataType, DataTypeDef, FnCallMetadata, ValidatorType},
        workflow::{
            ActivityRight, ArgType, ExprArg, Expression, Postprocessing, ProposeSettings, Template,
            TemplateActivity, TemplateSettings, TransitionConstraint, VoteScenario,
        },
        FnCallId,
    };

    use crate::data::skyward::{
        workflow_skyward_template_data_1, workflow_skyward_template_settings_data_1,
    };

    use near_sdk::serde_json;

    #[test]
    fn output_workflow_add_wf() {
        let wf = Template {
            name: "wf_add".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "wf_add".into(),
                    exec_condition: None,
                    action: ActionIdent::WorkflowAdd,
                    fncall_id: None,
                    tgas: 0,
                    deposit: 0,
                    arg_types: vec![DataTypeDef::U16(false), DataTypeDef::Object(0)],
                    postprocessing: None,
                }),
            ],
            transitions: vec![vec![1]],
            start: vec![0],
            end: vec![1],
            binds: vec![],
        };

        println!(
            "------------------------------ WORKFLOW ADD ------------------------------\n{}",
            serde_json::to_string(&wf).unwrap()
        );
    }

    #[test]
    fn output_workflow_payout() {
        let wf = Template {
            name: "wf_near_send".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "near_send".into(),
                    exec_condition: None,
                    action: ActionIdent::NearSend,
                    fncall_id: None,
                    tgas: 0,
                    deposit: 0,
                    arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                    postprocessing: None,
                }),
            ],
            transitions: vec![vec![1], vec![1]],
            binds: vec![],
            start: vec![0],
            end: vec![1],
        };

        println!(
            "------------------------------ WORKFLOW PAYOUT NEAR ------------------------------\n{}",
            serde_json::to_string(&wf).unwrap()
        );
    }

    #[test]
    pub fn output_workflow_skyward_template_1() {
        let (wf, fncalls, metadata) = workflow_skyward_template_data_1();

        println!(
            "------------------------------ WORKFLOW SKYWARD ------------------------------\n{}",
            serde_json::to_string(&wf).unwrap()
        );

        println!(
            "------------------------------ WORKFLOW SKYWARD FNCALLS ------------------------------\n{}",
            serde_json::to_string(&fncalls).unwrap()
        );

        println!(
            "------------------------------ WORKFLOW SKYWARD FN_METADATA ------------------------------\n{}",
            serde_json::to_string(&metadata).unwrap()
        );
    }

    #[test]
    fn output_workflow_skyward_settings_1() {
        let (wfs, settings) = workflow_skyward_template_settings_data_1();

        println!(
            "------------------------------ WORKFLOW SKYWARD TEMPLATE SETTINGS ------------------------------\n{}",
            serde_json::to_string(&wfs).unwrap()
        );

        println!(
            "------------------------------ WORKFLOW SKYWARD PROPOSE SETTINGS ------------------------------\n{}",
            serde_json::to_string(&settings).unwrap()
        );
    }

    #[test]
    fn output_settings() {
        let settings = ProposeSettings {
            activity_inputs: vec![vec![vec![ArgType::Free]]],
            transition_constraints: vec![
                vec![TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                }],
                vec![TransitionConstraint {
                    transition_limit: 0,
                    cond: None,
                }],
            ],
            binds: vec![],
            storage_key: "wf_add_wf_1".into(),
            obj_validators: vec![vec![]],
            validator_exprs: vec![],
        };

        println!(
            "------------------------------ PROPOSE SETTINGS ADD WORKFLOW ------------------------------\n{}",
            serde_json::to_string(&settings).unwrap()
        );

        let wfs = TemplateSettings {
            activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::TokenHolder,
            scenario: VoteScenario::TokenWeighted,
            duration: 60,
            quorum: 51,
            approve_threshold: 20,
            spam_threshold: 80,
            vote_only_once: true,
            deposit_propose: Some(1),
            deposit_vote: Some(1000),
            deposit_propose_return: 0,
        };

        println!(
            "------------------------------ TEMPLATE SETTINGS ADD WORFLOW ------------------------------\n{}",
            serde_json::to_string(&wfs).unwrap()
        );

        let wfs = TemplateSettings {
            activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
            allowed_proposers: vec![ActivityRight::Group(1)],
            allowed_voters: ActivityRight::TokenHolder,
            scenario: VoteScenario::TokenWeighted,
            duration: 60,
            quorum: 51,
            approve_threshold: 20,
            spam_threshold: 80,
            vote_only_once: true,
            deposit_propose: Some(1),
            deposit_vote: Some(1000),
            deposit_propose_return: 0,
        };

        println!(
            "------------------------------ TEMPLATE SETTINGS SEND NEAR WORKFLOW ------------------------------\n{}",
            serde_json::to_string(&wfs).unwrap()
        );

        let propose_settings = ProposeSettings {
            activity_inputs: vec![vec![vec![ArgType::Free, ArgType::Free]]],
            transition_constraints: vec![
                vec![TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                }],
                vec![TransitionConstraint {
                    transition_limit: 4,
                    cond: None,
                }],
            ],
            binds: vec![DataType::U128(10u128.pow(24).into())],
            obj_validators: vec![vec![ValidatorType::Primitive(0)]],
            validator_exprs: vec![Expression {
                args: vec![ExprArg::User(1), ExprArg::Bind(0)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::GtE),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                }),
            }],
            storage_key: "wf_send_near_1".into(),
        };

        println!(
            "------------------------------ PROPOSE SETTINGS PAYOUT ------------------------------\n{}",
            serde_json::to_string(&propose_settings).unwrap()
        );
    }
}
