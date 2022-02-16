use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
    serde_json, AccountId,
};

use crate::{
    expression::{Condition, EExpr},
    storage::StorageBucket,
    types::{ActionData, ActionIdent, DataType, DataTypeDef, ValidatorType},
    ActivityId, BindId, Consts, EventCode, FnCallId, TransitionId,
};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteScenario {
    Democratic,
    TokenWeighted,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Template {
    pub name: String,
    pub version: u8,
    pub activities: Vec<Option<TemplateActivity>>, // pos is ActivityId, None is always at 0. index as start activity
    pub obj_validators: Vec<Vec<ValidatorType>>,
    pub validator_exprs: Vec<Expression>,
    pub transitions: Vec<Vec<u8>>, //u8 as ActivityId
    pub binds: Vec<DataType>,      // TODO
    pub start: Vec<u8>,
    pub end: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateActivity {
    pub code: String,
    pub exec_condition: Option<Expression>, //TODO remove??
    pub action: ActionIdent,
    pub action_data: Option<ActionData>,
    pub arg_types: Vec<DataTypeDef>,
    pub activity_inputs: Vec<Vec<ArgType>>, //arguments for each activity
    pub postprocessing: Option<Postprocessing>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TemplateSettings {
    pub allowed_proposers: Vec<ActivityRight>,
    pub allowed_voters: ActivityRight,
    pub activity_rights: Vec<Vec<ActivityRight>>, //ActivityRight
    pub transition_constraints: Vec<Vec<TransitionConstraint>>,
    pub scenario: VoteScenario,
    pub duration: u32,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub spam_threshold: u8,
    pub vote_only_once: bool,
    pub deposit_propose: Option<U128>,
    pub deposit_vote: Option<U128>, //
    pub deposit_propose_return: u8, // how many percent of propose deposit to return
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TransitionConstraint {
    pub transition_limit: u16,
    pub cond: Option<Expression>,
}

// Template settings for proposing and limits
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ProposeSettings {
    pub binds: Vec<DataType>,
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
        consts: &Consts,
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
                ExprArg::Const(const_id) => {
                    binded_args.push(consts(*const_id));
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
    Const(u8),
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionResult {
    Ok,
    Finished,
    NoRights,
    NotEnoughDeposit,
    TransitionNotPossible,
    ProposalNotAccepted,
    Postprocessing,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum PostprocessingType {
    RemoveActionStorage(String),
    SaveBind(u8),
    SaveValue(DataType),
    SaveUserValue((u8, u8)), //object id - value pos
    FnCallResult(DataTypeDef),
    Instructions,
}

/// Simple post-fncall instructions which say what to do based on FnCall result.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Postprocessing {
    pub storage_key: String,
    pub op_type: PostprocessingType,
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

    pub fn try_to_get_inner_value(
        &self,
        user_inputs: &[Vec<DataType>],
        binds: &[DataType],
    ) -> Option<DataType> {
        match self.op_type {
            PostprocessingType::SaveUserValue((obj_id, arg_pos)) => Some(
                // Should panic if value is not there to signal wrong workflow structure
                user_inputs
                    .get(obj_id as usize)
                    .unwrap()
                    .get(arg_pos as usize)
                    .unwrap()
                    .clone(),
            ),
            PostprocessingType::SaveBind(bind_id) => Some(binds[bind_id as usize].clone()),
            _ => None,
        }
    }

    // TODO handle parse error
    /// Executes postprocessing
    ///
    pub fn postprocess(
        self,
        fn_result_val: Vec<u8>,
        inner_value: Option<DataType>,
        storage: &mut StorageBucket,
    ) -> Option<DataType> {
        match self.op_type {
            PostprocessingType::RemoveActionStorage(key) => {
                storage.remove_data(&key);
                None
            }
            PostprocessingType::SaveBind(_) => Some(inner_value.unwrap()),
            PostprocessingType::SaveUserValue(_) => Some(inner_value.unwrap()),
            PostprocessingType::FnCallResult(t) => {
                let result = match t {
                    DataTypeDef::String(_) => {
                        DataType::String(serde_json::from_slice::<String>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::Bool(_) => {
                        DataType::Bool(serde_json::from_slice::<bool>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U8(_) => {
                        DataType::U8(serde_json::from_slice::<u8>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U16(_) => {
                        DataType::U16(serde_json::from_slice::<u16>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U32(_) => {
                        DataType::U32(serde_json::from_slice::<u32>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U64(_) => {
                        DataType::U64(serde_json::from_slice::<U64>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::U128(_) => {
                        DataType::U128(serde_json::from_slice::<U128>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::VecString => DataType::VecString(
                        serde_json::from_slice::<Vec<String>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU8 => {
                        DataType::VecU8(serde_json::from_slice::<Vec<u8>>(&fn_result_val).unwrap())
                    }
                    DataTypeDef::VecU16 => DataType::VecU16(
                        serde_json::from_slice::<Vec<u16>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU32 => DataType::VecU32(
                        serde_json::from_slice::<Vec<u32>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU64 => DataType::VecU64(
                        serde_json::from_slice::<Vec<U64>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::VecU128 => DataType::VecU128(
                        serde_json::from_slice::<Vec<U128>>(&fn_result_val).unwrap(),
                    ),
                    DataTypeDef::Object(_) => {
                        unimplemented!("object is not supported yet");
                    }
                    DataTypeDef::NullableObject(_) => {
                        unimplemented!("object is not supported yet");
                    }
                    DataTypeDef::VecObject(_) => {
                        unimplemented!("object is not supported yet");
                    }
                };
                Some(result)
            }
            PostprocessingType::SaveValue(val) => Some(val),
            PostprocessingType::Instructions => {
                unimplemented!()
            }
        }
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
    pub transition_counter: Vec<Vec<u16>>,
    pub template_id: u16,
}

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

    // TODO optimalize so we dont have to subtract index by one each time
    /// Finds transition for dao action
    pub fn get_target_trans_with_for_dao_action(
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

    /// Finds transition for fncall
    pub fn get_target_trans_with_for_fncall(
        &self,
        wft: &Template,
        fn_call_ident: FnCallId,
    ) -> Option<(TransitionId, ActivityId)> {
        wft.transitions
            .get(self.current_activity_id as usize)
            .map(|t| {
                t.iter()
                    .enumerate()
                    .find(|(_, act_id)| {
                        if wft.activities[**act_id as usize].as_ref().unwrap().action
                            != ActionIdent::FnCall
                        {
                            return false;
                        }

                        match wft.activities[**act_id as usize]
                            .as_ref()
                            .unwrap()
                            .action_data
                            .as_ref()
                        {
                            Some(data) => match data {
                                ActionData::FnCall(fncall) => fncall.id == fn_call_ident,
                                ActionData::Event(_) => false,
                            },
                            None => false,
                        }
                    })
                    .map(|(t_id, act_id)| (t_id as u8, *act_id))
            })
            .flatten()
    }

    /// Finds transition for event
    pub fn get_target_trans_with_for_event(
        &self,
        wft: &Template,
        event_code: &EventCode,
    ) -> Option<(TransitionId, ActivityId)> {
        wft.transitions
            .get(self.current_activity_id as usize)
            .map(|t| {
                t.iter()
                    .enumerate()
                    .find(|(_, act_id)| {
                        if wft.activities[**act_id as usize].as_ref().unwrap().action
                            != ActionIdent::Event
                        {
                            return false;
                        }

                        match wft.activities[**act_id as usize]
                            .as_ref()
                            .unwrap()
                            .action_data
                            .as_ref()
                        {
                            Some(data) => match data {
                                ActionData::FnCall(_) => false,
                                ActionData::Event(e) => e.code == *event_code,
                            },
                            None => false,
                        }
                    })
                    .map(|(t_id, act_id)| (t_id as u8, *act_id))
            })
            .flatten()
    }

    // TODO remove pos_level when cond is changed
    /// Tries to advance to next activity in workflow.
    /// Returns transition result + Postprocessing if theres some
    /// Conditions might panics underneath
    pub fn transition_to_next(
        &mut self,
        activity_id: u8,
        transition_id: u8,
        wft: &Template,
        consts: &Consts,
        wfs: &TemplateSettings,
        settings: &ProposeSettings,
        action_args: &[Vec<DataType>],
        storage_bucket: &StorageBucket,
        pos_level: usize,
    ) -> (ActionResult, Option<Postprocessing>) {
        //TODO switching to finish state
        if self.state == InstanceState::Finished {
            return (ActionResult::Finished, None);
        }

        let transition_settings =
            &wfs.transition_constraints[self.current_activity_id as usize][transition_id as usize];

        // TODO trans and activity cond should be required only validation against storage
        //check transition cond
        let transition_cond_result = match &transition_settings.cond {
            Some(c) => c
                .bind_and_eval(
                    consts,
                    storage_bucket,
                    settings.binds.as_slice(),
                    &action_args[pos_level],
                )
                .try_into_bool()
                .unwrap_or(true),
            None => true,
        };

        if !transition_cond_result {
            return (ActionResult::TransitionCondFailed, None);
        }

        // check transition counter
        if self.transition_counter[self.current_activity_id as usize][transition_id as usize] + 1
            > transition_settings.transition_limit
        {
            return (ActionResult::MaxTransitionLimitReached, None);
        }

        self.transition_counter[self.current_activity_id as usize][transition_id as usize] += 1;
        self.current_activity_id = activity_id;

        // check if we can run this
        let wanted_activity = wft.activities[activity_id as usize].as_ref().unwrap();
        let can_be_exec = match wanted_activity.exec_condition {
            Some(ref e) => e.bind_and_eval(
                consts,
                storage_bucket,
                settings.binds.as_slice(),
                &action_args[pos_level],
            ),
            None => DataType::Bool(true),
        };

        if !can_be_exec.try_into_bool().unwrap() {
            return (ActionResult::ActivityCondFailed, None);
        }

        (ActionResult::Ok, wanted_activity.postprocessing.clone())
    }

    pub fn finish(&mut self, wft: &Template) -> bool {
        if wft.end.contains(&self.current_activity_id) {
            self.state = InstanceState::Finished;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]

mod test {
    use crate::{
        expression::{Condition, EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        storage::StorageBucket,
        types::{ActionIdent, DataType, DataTypeDef, FnCallMetadata, ValidatorType},
        utils::{bind_args, validate_args},
        workflow::{
            ActionResult, ActivityRight, ExprArg, InstanceState, PostprocessingType,
            ProposeSettings, TemplateActivity, TemplateSettings, TransitionConstraint,
            VoteScenario,
        },
    };

    use super::{ArgType, CondOrExpr, Expression, Instance, Postprocessing, Template};

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
            name: "send_near_and_create_group".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "near_send".into(),
                    exec_condition: None,
                    action: ActionIdent::TreasurySendNear,
                    action_data: None,
                    arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                    postprocessing: pp.clone(),
                    activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                }),
                Some(TemplateActivity {
                    code: "group_add".into(),
                    exec_condition: None,
                    action: ActionIdent::GroupAdd,
                    action_data: None,
                    arg_types: vec![
                        DataTypeDef::String(false),
                        DataTypeDef::String(false),
                        DataTypeDef::String(false),
                    ],
                    postprocessing: pp.clone(),
                    activity_inputs: vec![vec![ArgType::Bind(0), ArgType::Free, ArgType::Free]],
                }),
            ],
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
            transitions: vec![vec![1], vec![2]],
            binds: vec![],
            start: vec![0],
            end: vec![2],
        };

        //Template Settings example
        let wfs = TemplateSettings {
            transition_constraints: vec![
                vec![TransitionConstraint {
                    transition_limit: 1,
                    cond: None,
                }],
                vec![TransitionConstraint {
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
            .get_target_trans_with_for_dao_action(&wft, ActionIdent::TreasurySendNear)
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
            .get_target_trans_with_for_dao_action(&wft, ActionIdent::GroupAdd)
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
            name: "send_near_in_loop".into(),
            version: 1,
            activities: vec![
                None,
                Some(TemplateActivity {
                    code: "near_send".into(),
                    exec_condition: None,
                    action: ActionIdent::TreasurySendNear,
                    action_data: None,
                    arg_types: vec![DataTypeDef::String(false), DataTypeDef::U128(false)],
                    postprocessing: pp.clone(),
                    activity_inputs: vec![vec![ArgType::Free, ArgType::Free]],
                }),
            ],
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
            transitions: vec![vec![1], vec![1]],
            binds: vec![],
            start: vec![0],
            end: vec![1],
        };

        //Template Settings example
        let wfs = TemplateSettings {
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
                .get_target_trans_with_for_dao_action(&wft, ActionIdent::TreasurySendNear)
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
            .get_target_trans_with_for_dao_action(&wft, ActionIdent::TreasurySendNear)
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
