use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    expression::{Condition, EExpr},
    storage::StorageBucket,
    types::{ActionIdent, DataType, DataTypeDef, FnCallMetadata, ValidatorType},
    BindId, FnCallId,
};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteScenario {
    Democratic,
    TokenWeighted,
}

/*
Scenario
Simple dao call:
- users sends inputs, dao puts them into vec<vec<inputs>>
- wfi finds transition and activity - checks rights
- wfi evaluates transition cond
- wfi evaluates activity cond
- bind args - binds values from templates
- calls dao fn with args

FnCall:
- user sends inputs in vec<vec<input>>,
- wfi finds transition and activity - checks rights
- wfi evaluates transition cond
- wfi evaluates activity cond
- bind args - binds values from templates
- parses binded args into json and calls fn call

How to solve array of objects???

primitive/object(...)/array[]
values: [[first object][second object][third object]...]

[
    [name1, name2, obj]
    [nested_obj_1_name, nested_obj_1_array]
    [VecObjValue1, VecObjValue2]
]
    [value1, value2, Obj(1)]
    [nested_obj_1_value1, VecObj(2)]
    [vec_obj_value1, vec_obj_value2]
]

*/

// Issue: ATM we are not able to bind/validate non-primitive data types, eg. bind GroupSettings type to WF
// TODO use the schema in ::actions
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Template {
    pub name: String,
    pub version: u8,
    pub activities: Vec<Option<TemplateActivity>>, // pos is ActivityId, None is always at 0. index as start activity
    pub transitions: Vec<Vec<u8>>,                 //u8 as ActivityId
    pub binds: Vec<DataType>,                      // TODO ??
    pub start: Vec<u8>,
    pub end: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateActivity {
    pub code: String,
    pub exec_condition: Option<Expression>,
    pub action: ActionIdent,
    pub fncall_id: Option<FnCallId>,
    pub tgas: Option<u32>,
    pub deposit: Option<u128>,
    pub arg_types: Vec<DataTypeDef>,
    //pub arg_validators: Vec<Expression>,
    //pub binds: Vec<DataType>,
    pub postprocessing: Option<Postprocessing>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateSettings {
    pub allowed_proposers: Vec<ActivityRight>,
    pub allowed_voters: ActivityRight,
    pub activity_rights: Vec<Vec<ActivityRight>>, //ActivityRight
    //pub constants: Vec<DataType>,       // ??
    //pub validators: Vec<Vec<DataType>>, //[activity_id][argument pos] = validator
    pub scenario: VoteScenario,
    pub duration: u32,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub spam_threshold: u8,
    pub vote_only_once: bool,
    pub deposit_propose: Option<u128>,
    pub deposit_vote: Option<u128>, // Near
    pub deposit_propose_return: u8, // how many percent of propose deposit to return
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TransitionConstraint {
    pub transition_limit: u8,
    pub cond: Option<Expression>,
}

// Template settings for proposing and limits
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposeSettings {
    pub activity_inputs: Vec<Vec<Vec<ArgType>>>, //arguments for each activity
    pub transition_constraints: Vec<Vec<TransitionConstraint>>,
    pub binds: Vec<DataType>,
    pub obj_validators: Vec<Vec<ValidatorType>>,
    pub validator_exprs: Vec<Expression>,
    pub storage_key: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)] //Remove clone + debug
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
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
    Bind(BindId), // Template hardcoded,
    Storage(String),
    Expression(Expression),
    Object(u8),
    VecObject(u8),
    Const(u8), // dao specific value known at runtime, eg 0. is dao account name
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
    ErrValidation,
    ErrPostprocessing,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone))]
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

pub type TransitionId = u8;
pub type ActivityId = u8;

impl Instance {
    pub fn new(template_id: u16, transitions: &Vec<Vec<u8>>) -> Self {
        let mut transition_counter = Vec::with_capacity(transitions.len());

        for t in transitions.iter() {
            transition_counter.push(vec![0; t.len()])
        }

        Instance {
            state: InstanceState::Waiting,
            current_activity_id: 0,
            transition_counter,
            template_id,
        }
    }

    /// Finds id of desired activity if theres existing transition from current to desired
    pub fn get_target_trans_with_act(
        &self,
        wft: &Template,
        action_ident: ActionIdent,
    ) -> Option<(TransitionId, ActivityId)> {
        wft.transitions
            .get(self.current_activity_id as usize)
            .map(|t| {
                t.iter()
                    .enumerate()
                    .find(|(_, act_id)| {
                        wft.activities[**act_id as usize].as_ref().unwrap().action == action_ident
                    })
                    .map(|(t_id, act_id)| (t_id as u8, *act_id))
            })
            .flatten()
    }

    /// Tries to advance to next activity in workflow.
    /// Panics if anything is wrong - for now.
    pub fn transition_to_next(
        &mut self,
        activity_id: u8,
        transition_id: u8,
        wft: &Template,
        settings: &ProposeSettings,
        action_args: &[Vec<DataType>],
        storage_bucket: &StorageBucket,
        pos_level: usize,
    ) -> (ActivityResult, Option<Postprocessing>) {
        //TODO switching to finish state
        if self.state == InstanceState::Finished {
            return (ActivityResult::Finished, None);
        }

        assert_eq!(self.state, InstanceState::Running);

        let transition_settings = &settings.transition_constraints
            [self.current_activity_id as usize][transition_id as usize];

        // TODO trans and activity cond should required only validation against storage
        //check transition cond
        let transition_cond_result = match &transition_settings.cond {
            Some(c) => c
                .bind_and_eval(
                    storage_bucket,
                    settings.binds.as_slice(),
                    &action_args[pos_level],
                )
                .try_into_bool()
                .unwrap_or(true),
            None => true,
        };

        if !transition_cond_result {
            return (ActivityResult::TransitionCondFailed, None);
        }

        // check transition counter
        if self.transition_counter[self.current_activity_id as usize][transition_id as usize] + 1
            > transition_settings.transition_limit
        {
            return (ActivityResult::MaxTransitionLimitReached, None);
        }

        self.transition_counter[self.current_activity_id as usize][transition_id as usize] += 1;
        self.current_activity_id = activity_id;

        // check if we can run this
        let wanted_activity = wft.activities[activity_id as usize].as_ref().unwrap();
        let can_be_exec = match wanted_activity.exec_condition {
            Some(ref e) => e.bind_and_eval(
                storage_bucket,
                settings.binds.as_slice(),
                &action_args[pos_level],
            ),
            None => DataType::Bool(true),
        };

        if !can_be_exec.try_into_bool().unwrap() {
            return (ActivityResult::ActivityCondFailed, None);
        }

        // TODO to end transition - should by done by app

        (ActivityResult::Ok, wanted_activity.postprocessing.clone())
    }
}

#[cfg(test)]

mod test {
    use crate::{
        expression::{Condition, EExpr, EOp, ExprTerm, FnName, Op, RelOp, TExpr},
        storage::StorageBucket,
        types::{ActionIdent, DataType, DataTypeDef, FnCallMetadata, ValidatorType},
        utils::{bind_args, validate_args},
        workflow::{
            ActivityResult, ActivityRight, ExprArg, InstanceState, ProposeSettings,
            TemplateActivity, TemplateSettings, TransitionConstraint, VoteScenario,
        },
    };

    use super::{ArgType, CondOrExpr, Expression, Instance, Postprocessing, Template};

    // PoC test case
    #[test]
    pub fn workflow_simple_1() {
        let pp = Some(Postprocessing {
            storage_key: "activity_1_postprocessing".into(),
            fn_call_result_type: DataTypeDef::String(false),
            instructions: vec![],
        });

        let metadata: Vec<FnCallMetadata> = vec![];

        // Template
        let wft = Template {
            name: "send_near_and_create_group".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "near_send".into(),
                    exec_condition: None,
                    action: ActionIdent::NearSend,
                    fncall_id: None,
                    tgas: None,
                    deposit: None,
                    arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                    postprocessing: pp.clone(),
                }),
                Some(TemplateActivity {
                    code: "group_add".into(),
                    exec_condition: None,
                    action: ActionIdent::GroupAdd,
                    fncall_id: None,
                    tgas: None,
                    deposit: None,
                    arg_types: vec![
                        DataTypeDef::String(false),
                        DataTypeDef::String(false),
                        DataTypeDef::String(false),
                    ],
                    postprocessing: pp.clone(),
                }),
            ],
            transitions: vec![vec![1], vec![2]],
            binds: vec![],
            start: vec![0],
            end: vec![2],
        };

        //Template Settings example
        let wfs = TemplateSettings {
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
            deposit_propose: Some(1),
            deposit_vote: Some(1000),
            deposit_propose_return: 0,
        };

        //User proposed settings type
        let settings = ProposeSettings {
            activity_inputs: vec![
                vec![],
                vec![vec![ArgType::Free, ArgType::Free]],
                vec![vec![ArgType::Bind(0), ArgType::Free, ArgType::Free]],
            ],
            transition_constraints: vec![
                vec![TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                }],
                vec![TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                }],
                vec![TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                }],
            ],
            binds: vec![
                DataType::String("rustaceans_group".into()),
                DataType::U8(100),
            ],
            storage_key: "wf_simple".into(),
            obj_validators: vec![vec![], vec![ValidatorType::Primitive(0)], vec![]],
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
            .get_target_trans_with_act(&wft, ActionIdent::NearSend)
            .unwrap();

        assert_eq!(activity_id, 1);
        assert_eq!(transition_id, 0);

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &settings,
            &user_args,
            &mut bucket,
            0,
        );

        assert!(validate_args(
            &settings.binds,
            &settings.obj_validators[activity_id as usize].as_slice(),
            &settings.validator_exprs.as_slice(),
            &bucket,
            user_args.as_slice(),
            user_args_collection.as_slice(),
            metadata.as_slice(),
        ));

        let expected_result = (ActivityResult::Ok, pp.clone());

        assert_eq!(result, expected_result);
        assert_eq!(wfi.current_activity_id, 1);

        let dao_consts = Box::new(|id: u8| match id {
            0 => DataType::String("neardao.near".into()),
            _ => unimplemented!(),
        });

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize].as_slice(),
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
            .get_target_trans_with_act(&wft, ActionIdent::GroupAdd)
            .unwrap();

        assert_eq!(activity_id, 2);
        assert_eq!(transition_id, 0);

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
            &settings,
            &user_args,
            &bucket,
            0,
        );

        assert!(validate_args(
            &settings.binds,
            &settings.obj_validators[activity_id as usize].as_slice(),
            &settings.validator_exprs.as_slice(),
            &bucket,
            user_args.as_slice(),
            user_args_collection.as_slice(),
            metadata.as_slice(),
        ));

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize].as_slice(),
            &mut bucket,
            &mut user_args,
            &mut user_args_collection,
            0,
            0,
        );

        let expected_result = (ActivityResult::Ok, pp.clone());
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
            fn_call_result_type: DataTypeDef::String(false),
            instructions: vec![],
        });

        // Template
        let wft = Template {
            name: "send_near_in_loop".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "near_send".into(),
                    exec_condition: None,
                    action: ActionIdent::NearSend,
                    fncall_id: None,
                    tgas: None,
                    deposit: None,
                    arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                    postprocessing: pp.clone(),
                }),
            ],
            transitions: vec![vec![1], vec![1]],
            binds: vec![],
            start: vec![0],
            end: vec![1],
        };

        //Template Settings example
        let wfs = TemplateSettings {
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
            deposit_propose: Some(1),
            deposit_vote: Some(1000),
            deposit_propose_return: 0,
        };

        //User proposed settings type
        let settings = ProposeSettings {
            activity_inputs: vec![vec![], vec![vec![ArgType::Free, ArgType::Free]]],
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
            obj_validators: vec![vec![], vec![ValidatorType::Primitive(0)], vec![]],
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
                .get_target_trans_with_act(&wft, ActionIdent::NearSend)
                .unwrap();

            assert_eq!(activity_id, 1);
            assert_eq!(transition_id, 0);

            let result = wfi.transition_to_next(
                activity_id,
                transition_id,
                &wft,
                &settings,
                &user_args,
                &mut bucket,
                0,
            );
            let expected_result = (ActivityResult::Ok, pp.clone());

            assert!(validate_args(
                &settings.binds,
                &settings.obj_validators[activity_id as usize].as_slice(),
                &settings.validator_exprs.as_slice(),
                &bucket,
                user_args.as_slice(),
                user_args_collection.as_slice(),
                metadata.as_slice(),
            ));

            bind_args(
                &dao_consts,
                settings.binds.as_slice(),
                settings.activity_inputs[activity_id as usize].as_slice(),
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
            .get_target_trans_with_act(&wft, ActionIdent::NearSend)
            .unwrap();

        let result = wfi.transition_to_next(
            activity_id,
            transition_id,
            &wft,
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
        let expected_result = (ActivityResult::MaxTransitionLimitReached, None);

        bind_args(
            &dao_consts,
            settings.binds.as_slice(),
            settings.activity_inputs[activity_id as usize].as_slice(),
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

    /*
       #[test]
       fn workflow_skyward_finance() {
           let input: Vec<DataType> = vec![];

           let pp = Some(Postprocessing {
               storage_key: "activity_1_postprocessing".into(),
               fn_call_result_type: DataTypeDef::String(false),
               instructions: vec![],
           });

           // Template
           let wft = Template {
               name: "wf_skyward".into(),
               version: 1,
               activities: vec![
                   None,
                   Some(TemplateActivity {
                       code: "register_tokens".into(),
                       exec_condition: None,
                       action: ActionIdent::FnCall,
                       fncall_id: Some(("skyward.near".into(), "register_tokens".into())),
                       tgas: Some(15),
                       deposit: Some(20_000_000_000_000_000_000_000),
                       metadata: vec![DataTypeDef::Object(0)],
                       postprocessing: pp.clone(),
                   }),
                   Some(TemplateActivity {
                       code: "storage_deposit".into(),
                       exec_condition: None,
                       action: ActionIdent::FnCall,
                       fncall_id: Some(("self".into(), "storage_deposit".into())),
                       tgas: Some(10),
                       deposit: Some(20_000_000_000_000_000_000_000),
                       metadata: vec![DataTypeDef::Object(0)],
                       postprocessing: pp.clone(),
                   }),
                   Some(TemplateActivity {
                       code: "storage_deposit".into(),
                       exec_condition: None,
                       action: ActionIdent::FnCall,
                       fncall_id: Some(("skyward.near".into(), "storage_deposit".into())),
                       tgas: Some(10),
                       deposit: Some(1_250_000_000_000_000_000_000),
                       metadata: vec![DataTypeDef::Object(0)],
                       postprocessing: pp.clone(),
                   }),
                   Some(TemplateActivity {
                       code: "ft_transfer_call".into(),
                       exec_condition: None,
                       action: ActionIdent::FnCall,
                       fncall_id: Some(("self".into(), "ft_transfer_call".into())),
                       tgas: Some(30),
                       deposit: Some(0),
                       metadata: vec![DataTypeDef::Object(0)],
                       postprocessing: pp.clone(),
                   }),
                   Some(TemplateActivity {
                       code: "sale_create".into(),
                       exec_condition: None,
                       action: ActionIdent::FnCall,
                       fncall_id: Some(("skyward.near".into(), "sale_create".into())),
                       tgas: Some(50),
                       deposit: Some(2_000_000_000_000_000_000_000_000_000),
                       metadata: vec![DataTypeDef::Object(0)],
                       postprocessing: pp.clone(),
                   }),
               ],
               transitions: vec![vec![1], vec![2], vec![3], vec![4], vec![5]],
               binds: vec![],
               start: vec![0],
               end: vec![5],
           };

           //Template Settings example
           let wfs = TemplateSettings {
               activity_rights: vec![
                   vec![],
                   vec![ActivityRight::GroupLeader(1)],
                   vec![ActivityRight::GroupLeader(1)],
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
               deposit_propose: Some(1),
               deposit_vote: Some(1000),
               deposit_propose_return: 0,
           };

           // User proposed settings type
           let settings = ProposeSettings {
               activity_inputs: vec![
                   // register tokens
                   vec![ArgType::Expression(Expression {
                       args: vec![ExprArg::User(0), ExprArg::Bind(0)],
                       expr: EExpr::Fn(FnName::ArrayMerge),
                   })],
                   // storage_deposit on self
                   vec![ArgType::Bind(1)],
                   // storage_deposit on other token
                   vec![ArgType::Bind(1)],
                   // ft_transfer_call on self
                   vec![
                       ArgType::Bind(1),
                       ArgType::Checked(0),
                       ArgType::Free,
                       ArgType::Bind(2),
                   ],
                   // sale_create
                   vec![ArgType::Object(0)],
               ],
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
               object_inputs: vec![
                   vec![
                       ArgType::Free,                                        //title
                       ArgType::Free,                                        //url
                       ArgType::Bind(0),                                     //permissions_contract_id
                       ArgType::VecObject(1),                                //out_tokens
                       ArgType::Storage("activity_1_postprocessing".into()), //in_token_account_id - eg. wnear
                       ArgType::Free,                                        //start_time
                       ArgType::Free,                                        //duration
                   ],
                   vec![
                       ArgType::Bind(3),                                     //token_account_id
                       ArgType::Storage("activity_4_postprocessing".into()), //balance
                       ArgType::Free,                                        //referral_bpt - optional
                   ],
               ],
               binds: vec![
                   DataType::VecString(vec!["self".into()]),
                   DataType::String("skyward.near".into()),
                   DataType::String("\"AccountDeposit\"".into()),
                   DataType::String("self".into()),
                   DataType::U128(1_000_000_000),
               ],
               validators: vec![Expression {
                   args: vec![ExprArg::User(0), ExprArg::Bind(4)],
                   expr: EExpr::Boolean(TExpr {
                       operators: vec![Op {
                           op_type: EOp::Rel(RelOp::Gt),
                           operands_ids: [0, 1],
                       }],
                       terms: vec![ExprTerm::Arg(1), ExprTerm::Arg(0)],
                   }),
               }],
               storage_key: "wf_simple_loop".into(),
           };
       }
    */

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
