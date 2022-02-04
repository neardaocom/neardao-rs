/*

Workflow templates for providers


*/

#[cfg(test)]
mod test {
    use library::{
        expression::{EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        types::{ActionIdent, DataType, DataTypeDef},
        workflow::{
            Activity, ActivityRight, ArgType, ExprArg, Expression, ProposeSettings, Template,
            TemplateActivity, TemplateSettings, TransitionConstraint, VoteScenario,
        },
    };
    use near_sdk::serde_json;

    #[test]
    fn add_workflow() {
        let wf = Template {
            name: "wf_add".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    exec_condition: None,
                    action: ActionIdent::WorkflowAdd,
                    fncall_id: None,
                    gas: Some(0),
                    deposit: Some(0),
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
    fn payout_workflow() {
        let wf = Template {
            name: "wf_near_send".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    exec_condition: None,
                    action: ActionIdent::NearSend,
                    fncall_id: None,
                    gas: None,
                    deposit: None,
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
    fn settings() {
        let settings = ProposeSettings {
            activity_rights: vec![vec![ActivityRight::GroupLeader(1)]],
            activity_inputs: vec![vec![ArgType::Free]],
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
            validators: vec![],
            storage_key: "test".into(),
        };

        println!(
            "------------------------------ PROPOSE SETTINGS ADD WORKFLOW ------------------------------\n{}",
            serde_json::to_string(&settings).unwrap()
        );

        let wfs = TemplateSettings {
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

        let propose_settings = ProposeSettings {
            activity_rights: vec![
                vec![ActivityRight::Group(1)],
                vec![ActivityRight::GroupLeader(1)],
            ],
            activity_inputs: vec![vec![ArgType::Free, ArgType::Checked(0)]],
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
            binds: vec![DataType::U8(100)],
            validators: vec![Expression {
                args: vec![ExprArg::User(1), ExprArg::Bind(0)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                }),
            }],
            storage_key: "test".into(),
        };

        println!(
            "------------------------------ PROPOSE SETTINGS PAYOUT ------------------------------\n{}",
            serde_json::to_string(&propose_settings).unwrap()
        );

    }
}
