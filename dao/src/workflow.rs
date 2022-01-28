use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::action::ActionIdent;

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
pub struct WorkflowActivity {
    pub action: ActionIdent,
    pub gas: u64,
    pub deposit: u128,         // only if its a fncall ??
    pub condition: Expression, //TODO
    pub arg_types: Vec<WorkflowActivityArgType>,
    pub args: Vec<String>,
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
    pub settings: String,
}
