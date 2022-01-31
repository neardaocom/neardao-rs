use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize}, AccountId,
};

use crate::{action::ActionIdent, FnCallId, GroupId, TagId};

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
    pub storage_id: String, // storage for this WF
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum WorkflowActivityArgType {
    Free,         //User provided
    Bind(String), // Template hardcoded,
    Register(String),
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
    pub postprocessing: Option<Postprocessing>,
}

impl WorkflowActivity {
    pub fn execute(&mut self) -> WorkflowActivityExecutionResult {
        todo!();
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Postprocessing {
    script: String, //TODO program expressions?
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
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
    pub fn transition_to_next(&mut self) {}
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
    pub deposit_vote: Option<u128>,              // Near
    pub deposit_propose_return: bool,            // if return deposit above when workflow finishes
    pub allowed_voters: ActivityRight,
    //pub vote_deposit: Option<u128>, //yoctoNear // get from DAO settings?
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
