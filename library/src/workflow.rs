use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    expression::{Condition, EExpr},
    storage::StorageBucket,
    types::{ActionIdent, DataType, DataTypeDef},
};

type ArgValidatorId = u8;
type BindId = u8;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteScenario {
    Democratic,
    TokenWeighted,
}

// Issue: ATM we are not able to bind/validate non-primitive data types, eg. bind GroupSettings type to WF
// TODO use the schema in ::actions
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Template {
    pub name: String,
    pub version: u8,
    pub activity: Vec<Option<Activity>>,
    pub transitions: Vec<Vec<Transition>>,
    pub start: Vec<u8>,
    pub end: Vec<u8>,
    pub storage_key: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Transition {
    pub to: u8,
    pub iteration_limit: u8,
    pub condition: Option<Expression>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Expression {
    pub args: Vec<ExprArg>,
    pub expr: EExpr,
}

impl Expression {
    pub fn bind_and_eval(
        &self,
        storage: &StorageBucket,
        binds: &[DataType],
        args: &[DataType],
    ) -> DataType {
        let mut binded_args: Vec<DataType> = Vec::with_capacity(args.len());

        for arg in self.args.iter() {
            match arg {
                ExprArg::User(id) => {
                    binded_args.push(args[*id as usize].clone());
                }
                ExprArg::Bind(id) => {
                    binded_args.push(binds[*id as usize].clone());
                }
                ExprArg::Storage(key) => {
                    binded_args.push(storage.get_data(key).unwrap().clone());
                }
            }
        }

        self.expr.eval(&mut binded_args)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ExprArg {
    User(u8),
    Bind(u8),
    Storage(String),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ArgType {
    Free,
    Checked(ArgValidatorId), //User provided
    Bind(BindId),            // Template hardcoded,
    Storage(String),
    Expression(Expression),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ActivityResult {
    Ok,
    Finished,
    MaxTransitionLimitReached,
    TransitionCondFailed,
    ActivityCondFailed,
    ErrPostprocessing,
    ErrInputOutOfRange, // user provided too high or too low value
    ErrRuntime,         // datatype mismatch/register missing
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Activity {
    pub name: String,
    pub exec_condition: Option<Expression>,
    pub action: ActionIdent,
    pub fncall_id: Option<String>, // only if self.action is FnCall variant
    pub gas: u64,
    pub deposit: u128,
    pub arg_types: Vec<ArgType>,
    pub arg_validators: Vec<Expression>,
    pub binds: Vec<DataType>,
    pub postprocessing: Option<Postprocessing>,
}

impl Activity {
    pub fn execute(&mut self) -> ActivityResult {
        todo!();
    }

    // TODO tests
    /// Interpolates args into result args
    pub fn interpolate_args(&self, mut args: &mut Vec<DataType>, storage: &mut StorageBucket) {
        assert_eq!(self.arg_types.len(), args.len());

        let mut result_args = Vec::with_capacity(self.arg_types.len());

        for (i, arg_type) in self.arg_types.iter().enumerate() {
            match arg_type {
                ArgType::Free => result_args.push(std::mem::replace(&mut args[i], DataType::Null)),
                ArgType::Checked(id) => {
                    let expr = self.arg_validators.get(*id as usize).unwrap();
                    if !expr
                        .bind_and_eval(storage, self.binds.as_slice(), result_args.as_slice())
                        .try_into_bool()
                        .unwrap()
                    {
                        panic!("{}", "Input is not valid");
                    }
                }
                ArgType::Bind(id) => result_args.push(self.binds[*id as usize].clone()),
                ArgType::Storage(key) => {
                    result_args.push(storage.get_data(key).unwrap());
                }
                ArgType::Expression(expr) => result_args.push(expr.bind_and_eval(
                    storage,
                    self.binds.as_slice(),
                    result_args.as_slice(),
                )),
            }
        }

        std::mem::swap(&mut result_args, &mut args);
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum CondOrExpr {
    Cond(Condition),
    Expr(EExpr),
}

/// Simple post-fncall instructions which say what to do based on FnCall result
/// ATM Used its only used to save fncall action result to the storage
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Postprocessing {
    pub storage_key: String,
    pub fn_call_result_type: DataTypeDef,
    pub instructions: Vec<CondOrExpr>,
}

impl Postprocessing {
    pub fn process(&self, result_input: &[DataType]) -> DataType {
        if self.instructions.len() == 0 {
            return result_input[0].clone();
        }

        let mut idx = 0;
        return loop {
            match &self.instructions[idx] {
                CondOrExpr::Cond(c) => idx = c.eval(result_input) as usize,
                CondOrExpr::Expr(e) => {
                    break e.eval(result_input);
                }
            }
        };
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum InstanceState {
    Waiting,
    Running,
    FatalError,
    Finished,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Instance {
    pub state: InstanceState,
    pub current_activity_id: u8,
    pub transition_counter: Vec<Vec<u8>>,
    pub template_id: u16,
}

impl Instance {
    pub fn new(template_id: u16, transitions: &Vec<Vec<Transition>>) -> Self {
        let mut transition_counter = Vec::with_capacity(transitions.len());

        for t in transitions.iter() {
            transition_counter.push(vec![0; t.len()])
        }

        Instance {
            state: InstanceState::Running,
            current_activity_id: 0,
            transition_counter,
            template_id,
        }
    }

    /// Tries to advance to next activity in workflow. Panics if anything is wrong - for now.
    pub fn transition_to_next<'a>(
        &mut self,
        wft: &Template,
        current_action_ident: ActionIdent,
        action_args: &mut Vec<DataType>,
        storage_bucket: &mut StorageBucket,
    ) -> (ActivityResult, Option<Postprocessing>) {
        if self.state == InstanceState::Finished {
            return (ActivityResult::Finished, None);
        }

        // check if theres transition from current action to desired
        let transition = wft.transitions[self.current_activity_id as usize]
            .iter()
            .enumerate()
            .find(|(_, t)| {
                wft.activity[t.to as usize].as_ref().unwrap().action == current_action_ident
            });

        // check if we can do the transition
        let current_activity;
        match transition {
            Some((_, t)) => {
                self.current_activity_id = t.to;
                current_activity = wft.activity[t.to as usize].as_ref().unwrap();
                match t
                    .condition
                    .as_ref()
                    .map(|cond| {
                        // Cond if desired activity can be run
                        cond.bind_and_eval(
                            storage_bucket,
                            current_activity.binds.as_slice(),
                            action_args.as_slice(),
                        )
                        .try_into_bool()
                        .unwrap()
                    })
                    .unwrap_or(true)
                {
                    true => (),
                    false => return (ActivityResult::TransitionCondFailed, None),
                };
            }
            None => panic!("{}", "Undefined transition"),
        }

        // ATM we know the activity is valid workflow activity

        // check transition counter
        if self.transition_counter[self.current_activity_id as usize - 1][transition.unwrap().0] + 1
            > transition.unwrap().1.iteration_limit
        {
            return (ActivityResult::MaxTransitionLimitReached, None);
        }

        self.transition_counter[self.current_activity_id as usize - 1][transition.unwrap().0] += 1;

        // check if we can run this
        let can_be_exec = match current_activity.exec_condition {
            Some(ref e) => e.bind_and_eval(
                storage_bucket,
                current_activity.binds.as_slice(),
                action_args.as_slice(),
            ),
            None => DataType::Bool(true),
        };

        if !can_be_exec.try_into_bool().unwrap() {
            return (ActivityResult::ActivityCondFailed, None);
        }

        // bind args and check values
        current_activity.interpolate_args(action_args, storage_bucket);

        // TODO to end transition - should by done by app

        (ActivityResult::Ok, current_activity.postprocessing.clone())
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ActivityRight {
    Anyone,
    Group(u16),
    GroupMember(u16, String),
    Account(AccountId),
    TokenHolder,
    Member,
    GroupRole(u16, u16),
    GroupLeader(u16),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Settings {
    pub allowed_proposers: Vec<ActivityRight>,
    pub deposit_propose: Option<u128>,
    pub deposit_vote: Option<u128>,   // Near
    pub deposit_propose_return: bool, // if return deposit above when workflow finishes
    pub allowed_voters: ActivityRight,
    pub vote_settings_id: u8,
    pub activity_rights: Vec<ActivityRight>,
    pub activity_inputs: Vec<Vec<ArgType>>, //arguments for each activity
    pub automatic_start: bool,              // maybe unnecessary
    pub scenario: VoteScenario,
    pub duration: u32,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub spam_threshold: u8,
    pub vote_only_once: bool,
}

#[cfg(test)]

mod test {
    use crate::{
        expression::{Condition, EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
        storage::StorageBucket,
        types::{ActionIdent, DataType, DataTypeDef},
        workflow::{ActivityResult, ExprArg},
    };

    use super::{
        Activity, ArgType, CondOrExpr, Expression, Instance, InstanceState, Postprocessing,
        Template, Transition,
    };

    // PoC test case
    #[test]
    pub fn workflow_simple() {
        // Eg: start -> send_near -> create_group -> end
        // Args: receiver -> binded

        let pp1 = Some(Postprocessing {
            storage_key: "activity_1_postprocessing".into(),
            fn_call_result_type: DataTypeDef::String(false),
            instructions: vec![],
        });

        // 100 > user input
        let expr_send_near = EExpr::Boolean(TExpr {
            operators: vec![Op {
                op_type: EOp::Rel(RelOp::Gt),
                operands_ids: [0, 1],
            }],
            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
        });

        // group_name == concat(input test_bind) (bind = "_group")
        let expr_add_group = EExpr::Boolean(TExpr {
            operators: vec![Op {
                operands_ids: [0, 1],
                op_type: EOp::Rel(RelOp::Eqs),
            }],
            terms: vec![ExprTerm::FnCall(FnName::Concat, (0, 1)), ExprTerm::Arg(2)],
        });

        let wft = Template {
            name: "test".into(),
            version: 1,
            activity: vec![
                None,
                Some(Activity {
                    name: "send_near".into(),
                    exec_condition: Some(Expression {
                        args: vec![ExprArg::Bind(0), ExprArg::User(1)],
                        expr: expr_send_near,
                    }),
                    action: ActionIdent::NearSend,
                    fncall_id: None,
                    gas: 0,
                    deposit: 0,
                    arg_types: vec![ArgType::Free, ArgType::Free],
                    arg_validators: vec![],
                    binds: vec![DataType::U8(100)],
                    postprocessing: pp1.clone(),
                }),
                Some(Activity {
                    name: "create_group".into(),
                    exec_condition: Some(Expression {
                        args: vec![ExprArg::User(1), ExprArg::Bind(1), ExprArg::Bind(2)],
                        expr: expr_add_group,
                    }),
                    action: ActionIdent::GroupAdd,
                    fncall_id: None,
                    gas: 0,
                    deposit: 0,
                    arg_types: vec![ArgType::Bind(0), ArgType::Bind(2), ArgType::Free],
                    arg_validators: vec![],
                    binds: vec![
                        DataType::String("rustaceans".into()),
                        DataType::String("_group".into()),
                        DataType::String("leaderisme_group".into()),
                    ],
                    postprocessing: pp1.clone(),
                }),
            ],
            transitions: vec![
                vec![Transition {
                    to: 1,
                    iteration_limit: 1,
                    condition: None,
                }],
                vec![Transition {
                    to: 2,
                    iteration_limit: 1,
                    condition: None,
                }],
                vec![Transition {
                    to: 3,
                    iteration_limit: 1,
                    condition: None,
                }],
            ],
            start: vec![0],
            end: vec![2],
            storage_key: "simple_wf".into(),
        };

        let mut wfi = Instance {
            state: InstanceState::Running,
            current_activity_id: 0,
            transition_counter: vec![vec![0], vec![0], vec![0]],
            template_id: 1,
        };
        let mut storage_bucket = StorageBucket::new(b"simple_wf".to_vec());

        // Execute Workflow
        let expected_args = vec![DataType::String("jonnyis.near".into()), DataType::U8(50)];

        let mut args_result = expected_args.clone();

        let result = wfi.transition_to_next(
            &wft,
            ActionIdent::NearSend,
            &mut args_result,
            &mut storage_bucket,
        );

        let expected_result = (ActivityResult::Ok, pp1.clone());

        assert_eq!(result, expected_result);
        assert_eq!(args_result, expected_args);
        assert_eq!(wfi.current_activity_id, 1);

        let mut args = vec![
            DataType::String("rustlovers".into()),
            DataType::String("leaderisme".into()),
            DataType::String("user_provided_settings".into()),
        ];

        let result =
            wfi.transition_to_next(&wft, ActionIdent::GroupAdd, &mut args, &mut storage_bucket);

        let expected_result = (ActivityResult::Ok, pp1.clone());
        let expected_args = vec![
            DataType::String("rustaceans".into()),
            DataType::String("leaderisme_group".into()),
            DataType::String("user_provided_settings".into()),
        ];

        assert_eq!(result, expected_result);
        assert_eq!(args, expected_args);
        assert_eq!(wfi.current_activity_id, 2);
    }

    #[test]
    fn workflow_simple_loop() {
        // 100 > user input
        let expr_send_near = EExpr::Boolean(TExpr {
            operators: vec![Op {
                op_type: EOp::Rel(RelOp::GtE),
                operands_ids: [0, 1],
            }],
            terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
        });

        let pp1 = Some(Postprocessing {
            storage_key: "activity_1_postprocessing".into(),
            fn_call_result_type: DataTypeDef::String(false),
            instructions: vec![],
        });

        let wft = Template {
            name: "test".into(),
            version: 1,
            activity: vec![
                None,
                Some(Activity {
                    name: "send_near".into(),
                    exec_condition: Some(Expression {
                        args: vec![ExprArg::Bind(0), ExprArg::User(1)],
                        expr: expr_send_near,
                    }),
                    action: ActionIdent::NearSend,
                    fncall_id: None,
                    gas: 0,
                    deposit: 0,
                    arg_types: vec![ArgType::Free, ArgType::Free],
                    arg_validators: vec![],
                    binds: vec![DataType::U8(50)],
                    postprocessing: pp1.clone(),
                }),
            ],
            transitions: vec![
                vec![Transition {
                    to: 1,
                    iteration_limit: 1,
                    condition: None,
                }],
                // From 1 to 1 loop
                vec![Transition {
                    to: 1,
                    iteration_limit: 5,
                    condition: None,
                }],
            ],
            start: vec![0],
            end: vec![2],
            storage_key: "simple_wf".into(),
        };

        let mut wfi = Instance {
            state: InstanceState::Running,
            current_activity_id: 0,
            transition_counter: vec![vec![0]],
            template_id: 1,
        };
        let mut storage_bucket = StorageBucket::new(b"simple_wf".to_vec());

        // Execute Workflow
        for i in 1..=5 {
            let expected_args = vec![DataType::String("jonnyis.near".into()), DataType::U8(50)];

            let mut args_result = expected_args.clone();

            let result = wfi.transition_to_next(
                &wft,
                ActionIdent::NearSend,
                &mut args_result,
                &mut storage_bucket,
            );

            let expected_result = (ActivityResult::Ok, pp1.clone());

            assert_eq!(result, expected_result);
            assert_eq!(args_result, expected_args);
            assert_eq!(wfi.current_activity_id, 1);
            assert_eq!(wfi.transition_counter[0][0], i);
        }

        let expected_args = vec![DataType::String("jonnyis.near".into()), DataType::U8(50)];

        let mut args_result = expected_args.clone();

        let result = wfi.transition_to_next(
            &wft,
            ActionIdent::NearSend,
            &mut args_result,
            &mut storage_bucket,
        );

        let expected_result = (ActivityResult::MaxTransitionLimitReached, None);

        assert_eq!(result, expected_result);
        assert_eq!(args_result, expected_args);
        assert_eq!(wfi.current_activity_id, 1);
        assert_eq!(wfi.transition_counter[0][0], 5);
    }

    #[test]
    fn postprocessing_with_cond() {
        // FnCall result > 5 then 20 else 40
        let input: Vec<DataType> = vec![DataType::U8(1)];
        let postprocessing = Postprocessing {
            storage_key: "key".into(),
            fn_call_result_type: DataTypeDef::String(false),
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
