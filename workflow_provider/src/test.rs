/*

Workflow templates for providers


*/

#[cfg(test)]
mod test {
    use library::{
        expression::{EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        types::ActionIdent,
        workflow::{Activity, ArgType, ExprArg, Expression, Template, Transition},
    };
    use near_sdk::serde_json;

    #[test]
    fn add_workflow() {
        let wf = Template {
            name: "add_wf".into(),
            version: 1,
            activity: vec![
                None,
                Some(Activity {
                    name: "add_wf".into(),
                    exec_condition: None,
                    action: ActionIdent::WorkflowAdd,
                    fncall_id: None,
                    gas: 0,
                    deposit: 0,
                    arg_types: vec![ArgType::Free],
                    arg_validators: vec![],
                    binds: vec![],
                    postprocessing: None,
                }),
            ],
            transitions: vec![vec![Transition {
                to: 1,
                iteration_limit: 10,
                condition: None,
            }]],
            start: vec![0],
            end: vec![1],
            storage_key: "wf_add_wft".into(),
        };

        println!(
            "------------------------------ WORKFLOW ADD ------------------------------\n{}",
            serde_json::to_string(&wf).unwrap()
        );
    }

    #[test]
    fn payout_workflow() {
        let wf = Template {
            name: "payout_wf".into(),
            version: 1,
            activity: vec![
                None,
                Some(Activity {
                    name: "payout_wf".into(),
                    exec_condition: None,
                    action: ActionIdent::NearSend,
                    fncall_id: None,
                    gas: 0,
                    deposit: 0,
                    arg_types: vec![ArgType::Free, ArgType::Checked(0)],
                    arg_validators: vec![Expression {
                        args: vec![ExprArg::Bind(0), ExprArg::User(1)],
                        expr: EExpr::Boolean(TExpr {
                            operators: vec![Op {
                                operands_ids: [0, 1],
                                op_type: EOp::Rel(RelOp::Eqs),
                            }],
                            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                        }),
                    }],
                    binds: vec![],
                    postprocessing: None,
                }),
            ],
            transitions: vec![vec![Transition {
                to: 1,
                iteration_limit: 5,
                condition: None,
            }]],
            start: vec![0],
            end: vec![1],
            storage_key: "wf_payout_wft".into(),
        };

        println!(
            "------------------------------ WORKFLOW PAYOUT NEAR ------------------------------\n{}",
            serde_json::to_string(&wf).unwrap()
        );
    }
}
