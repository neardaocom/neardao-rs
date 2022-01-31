use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId,
};

use crate::{
    action::ActionIdent,
    storage::{DataType, StorageBucket},
    FnCallId, GroupId, TagId,
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
pub struct WorkflowTemplate {
    pub name: String,
    pub version: u8,
    pub activity: Vec<WorkflowActivity>,
    pub transitions: Vec<WorkflowTransition>,
    pub start: Vec<u8>, // ids of activitys above
    pub end: Vec<u8>,
    pub storage_key: String, // storage for this WF
                             // TODO ad WF settings here?
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct WorkflowTransition {
    from: u8,
    to: u8,
    iteration_limit: u16,
    condition: Option<Expression>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Expression;

impl Expression {
    //TODO
    pub fn eval(&self, storage: &mut StorageBucket) -> DataType {
        DataType::Bool(true)
    }
}

type ArgValidatorId = u8;
type BindId = u8;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum WorkflowActivityArgType {
    Free,
    Checked(ArgValidatorId), //User provided
    Bind(BindId),            // Template hardcoded,
    Storage(String),
    Expression(Expression),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum WorkflowActivityExecutionResult {
    Success,
    ErrMaxTransitionLimitReached,
    ErrPostprocessing,
    ErrInputOutOfRange, // user provided too high or too low value
    ErrRuntime,         // datatype mismatch/register missing
                        // TODO ...
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct WorkflowActivity {
    pub name: String,
    pub exec_condition: Expression, //TODO
    pub action: ActionIdent,
    pub fncall_id: Option<FnCallId>, // only if self.action is FnCall variant
    pub gas: u64,
    pub deposit: u128,
    pub arg_types: Vec<WorkflowActivityArgType>,
    pub arg_validators: Vec<Expression>,
    pub binds: Vec<DataType>,
    pub postprocessing: Option<Postprocessing>,
}

impl WorkflowActivity {
    pub fn execute(&mut self) -> WorkflowActivityExecutionResult {
        todo!();
    }

    // TODO tests
    /// Interpolates args into result args
    pub fn interpolate_args(
        &self,
        mut args: Vec<DataType>,
        storage: &mut StorageBucket,
    ) -> Vec<DataType> {
        assert_eq!(self.arg_types.len(), args.len());

        let mut result_args = Vec::with_capacity(self.arg_types.len());

        for (i, arg_type) in self.arg_types.iter().enumerate() {
            match arg_type {
                WorkflowActivityArgType::Free => {
                    result_args.push(std::mem::replace(&mut args[i], DataType::Null))
                }
                WorkflowActivityArgType::Checked(id) => {
                    let expr = self.arg_validators.get(*id as usize).unwrap();
                    if !expr.eval(storage).try_into_bool().unwrap() {
                        panic!("{}", "Input is not valid");
                    }
                    result_args.push(std::mem::replace(&mut args[i], DataType::Null))
                }
                WorkflowActivityArgType::Bind(id) => {
                    result_args.push(self.binds[*id as usize].clone())
                }
                WorkflowActivityArgType::Storage(key) => {
                    result_args.push(storage.get_data(key).unwrap());
                }
                WorkflowActivityArgType::Expression(expr) => result_args.push(expr.eval(storage)),
            }
        }
        result_args
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Postprocessing {
    script: String, //TODO program expressions?
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum WorkflowInstanceState {
    Waiting,
    Running,
    FatalError,
    Finished,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct WorkflowInstance {
    pub state: WorkflowInstanceState,
    pub current_action: u8,
    pub transition_counter: Vec<u8>,
    pub template_id: u16,
}

impl WorkflowInstance {
    pub fn new(template_id: u16, transitions_len: usize) -> Self {
        WorkflowInstance {
            state: WorkflowInstanceState::Running,
            current_action: 0,
            transition_counter: vec![0; transitions_len],
            template_id,
        }
    }

    // TODO return type structure
    /// Tries to advance to next activity in workflow. Panics if anything is wrong.
    pub fn transition_to_next(
        &mut self,
        wft: &WorkflowTemplate,
        current_action_ident: ActionIdent,
        action_args: Vec<DataType>,
        storage_bucket: &mut StorageBucket,
    ) -> Option<Postprocessing> {
        if self.state == WorkflowInstanceState::Finished {
            return None;
        }

        // check if theres transition from current action to desired
        let transition = wft
            .transitions
            .iter()
            .filter(|t| t.from == self.current_action)
            .find(|t| wft.activity[t.to as usize].action == current_action_ident);

        let activity_id: u8 = match transition {
            Some(t) => match t
                .condition
                .as_ref()
                .map(|cond| cond.eval(storage_bucket).try_into_bool().unwrap())
                .unwrap_or(true)
            {
                true => t.to,
                false => panic!("{}", "Condition for transition not fullfiled"),
            },
            None => panic!("{}", "Undefined transition"),
        };

        // check if we can run this
        let can_be_exec = wft.activity[activity_id as usize]
            .exec_condition
            .eval(storage_bucket);

        if !can_be_exec.try_into_bool().unwrap() {
            //TODO
            return None;
        }

        // bind args and check values
        let args = wft.activity[self.current_action as usize]
            .interpolate_args(action_args, storage_bucket);

        //check if current activity is final
        if wft.end.contains(&self.current_action) {
            self.state = WorkflowInstanceState::Finished;
            //TODO
            return None;
        } else {
            self.current_action += 1;
        }

        None
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ActivityRight {
    Anyone,
    Group(GroupId),
    GroupMember(GroupId, String),
    Account(AccountId),
    TokenHolder,
    Member,
    GroupRole(GroupId, TagId),
    GroupLeader(GroupId),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct WorkflowSettings {
    pub allowed_proposers: Vec<ActivityRight>,
    pub deposit_propose: Option<u128>,
    pub deposit_vote: Option<u128>,   // Near
    pub deposit_propose_return: bool, // if return deposit above when workflow finishes
    pub allowed_voters: ActivityRight,
    pub vote_settings_id: u8,
    pub activity_rights: Vec<ActivityRight>,
    pub activity_inputs: Vec<Vec<WorkflowActivityArgType>>, //arguments for each activity
    pub automatic_start: bool,                              // maybe unnecessary
    pub scenario: VoteScenario,
    pub duration: u32,
    pub quorum: u8,
    pub approve_threshold: u8,
    pub spam_threshold: u8,
    pub vote_only_once: bool,
}
