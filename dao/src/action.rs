use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, WrappedDuration, WrappedTimestamp, U128, U64};
use near_sdk::log;
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::serde_json::{self, Value};
use near_sdk::{env, near_bindgen, AccountId, Promise};

use crate::constants::TGAS;
use crate::errors::{ERR_NO_ACCESS, ERR_UNKNOWN_FNCALL};
use crate::group::Group;
use crate::internal::utils;
use crate::release::ReleaseDb;
use crate::settings::assert_valid_dao_settings;
use crate::settings::DaoSettings;
use crate::tags::Tags;
use crate::{
    core::*,
    group::{GroupInput, GroupMember, GroupReleaseInput, GroupSettings},
    media::{FileType, Media, VFileMetadata},
    GroupId, GroupName, ProposalId, CID,
};
use crate::{TagCategory, TagId};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum TokenGroup {
    Council,
    Public,
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum TxInput {
    Pay {
        amount_near: U128,
        account_id: AccountId,
    },
    AddMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    RemoveMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    GeneralProposal {
        title: String,
    },
    AddDocFile {
        cid: CID,
        metadata: VFileMetadata,
        new_tags: Vec<String>,
        new_category: Option<String>,
    },
    InvalidateFile {
        cid: CID,
    },
    DistributeFT {
        total_amount: u32,
        from_group: TokenGroup,
        accounts: Vec<AccountId>,
    },
    RightForActionCall {
        to: RightTarget,
        rights: Vec<ActionGroupRight>,
        time_from: Option<WrappedTimestamp>,
        time_to: Option<WrappedTimestamp>,
    },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionTx {
    pub actions: Vec<Action>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)] //TODO Remove debug in production
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum TxValidationErr {
    NotEnoughGas,
    NotEnoughNears,
    NotEnoughFT,
    InvalidTimeInputs,
    CIDExists,
    GroupForbidden,
    UserAlreadyInGroup,
    UserNotInGroup,
    Custom(String),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    SendNear {
        account_id: AccountId,
        amount_near: u128,
    },
    AddMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    RemoveMember {
        account_id: AccountId,
        group: TokenGroup,
    },
    GeneralProposal {
        title: String,
    },
    //TODO split into two actions ?
    AddFile {
        cid: CID,
        ftype: FileType,
        metadata: VFileMetadata,
        new_category: Option<String>,
        new_tags: Vec<String>,
    },
    InvalidateFile {
        cid: CID,
    },
    DistributeFT {
        amount: u32,
        from_group: TokenGroup,
        accounts: Vec<AccountId>,
    },
    AddRightsForActionGroup {
        to: RightTarget,
        rights: Vec<ActionGroupRight>,
        time_from: u64,
        time_to: u64,
    },
    FunctionCall {
        account: AccountId,
        actions: Vec<ActionCall>,
    },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum RightTarget {
    Group { value: TokenGroup },
    Users { values: Vec<AccountId> },
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionGroupRight {
    RefFinance,
    SkywardFinance,
}

/// ActionGroup structure represents input type for external action calls from privileged user
/// One ActionGroup will be splitted into 1..N actions
#[derive(Deserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionGroupInput {
    RefRegisterTokens,
    RefAddPool {
        fee: Option<u32>,
    },
    RefAddLiquidity {
        pool_id: u32,
        amount_near: U128,
        amount_ft: U128,
    },
    RefWithdrawLiquidity {
        pool_id: u32,
        shares: U128,
        min_ft: Option<U128>,
        min_near: Option<U128>,
    },
    RefWithdrawDeposit {
        token_id: AccountId,
        amount: U128,
    },
    SkyCreateSale {
        title: String,
        url: String,
        amount_ft: U128,
        out_token_id: AccountId,
        time_from: WrappedTimestamp,
        duration: WrappedDuration,
    },
}
pub enum ActionResult<T> {
    Success,
    Error(T),
}

// TODO method whitelist
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    method: String,
    args: String,
    tgas: u32,
    deposit: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ExecutionRight {
    id: u8,
    valid_from: u64,
    valid_duration: u64,
    near_limit: u128,
    near_current: u128,
    ft_limit: Option<u32>,
    ft_current: Option<u32>,
    actions: Vec<Action>,
}

// Jmeno FT by mohlo byt evidovane v ramci workflows tak, aby to zbytecne nezabiralo misto v kazdem rights

// ---------------- NEW ----------------

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ActionIdent {
    GroupAdd,
    GroupRemove,
    GroupUpdate,
    GroupMemberAdd,
    GroupMemberRemove,
    FnCall(String),
    SettingsUpdate,
    MediaAdd,
    MediaInvalidate,
    FnCallAdd,
    FnCallRemove,
    TagAdd,
    TagEdit,
    TagRemove,
    FtUnlock,
    FtDistribute,
    FtSend,
    NftSend,
    NearSend,
    WorkflowChange, // TODO rozdelit na Add a remove??
    WorkflowInstall,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone,Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum DataType {
    String,
    Bool,
    U8,
    U32,
    U64,
    U128,
    VecString,
    VecU8,
    VecU16,
    VecU32,
    VecU64,
    VecU128,
    Object(u8),
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum MyValue {
    String(String),
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(U64),
    U128(U128),
    VecString(Vec<String>),
    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
    VecU32(Vec<u32>),
    VecU64(Vec<U64>),
    VecU128(Vec<U128>),
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallArguments {
    arg_names: Vec<String>,
    arg_values: Vec<MyValue>, // TODO user Vec u8 or Vec string ?? check with serializer
}

// Represents object schema
// Coz compiler yelling at me: "error[E0275]: overflow evaluating the requirement" on Borsh we do it this way
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallMetadata {
    pub arg_names: Vec<String>,
    pub arg_types: Vec<DataType>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct FnCallDefinition {
    pub name: String,
    pub receiver: AccountId,
}

// ---------- TEST OBJECTS
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TestObject {
    name1: String,
    name2: Vec<String>,
    name3: Vec<u128>,
    obj: InnerTestObject,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct InnerTestObject {
    nested_1_arr_8: Vec<u8>,
    nested_1_obj: Inner2TestObject,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Inner2TestObject {
    nested_2_arr_u64: Vec<U64>,
    bool_val: bool,
}

impl FnCallDefinition {
    /// Checks if fn metadata and provided fncall arguments match with their data type
    pub fn validate_input_types(
        &self,
        inputs: &FnCallArguments,
        metadata: &FnCallMetadata,
    ) -> bool {
        for i in 0..metadata.arg_names.len() {
            match (&metadata.arg_types[i], &inputs.arg_values[i]) {
                (DataType::Bool, MyValue::Bool(_)) => (),
                (DataType::String, MyValue::String(_)) => (),
                (DataType::U8, MyValue::U8(_)) => (),
                (DataType::U32, MyValue::U32(_)) => (),
                (DataType::U64, MyValue::U64(_)) => (),
                (DataType::U128, MyValue::U128(_)) => (),
                (DataType::VecString, MyValue::VecString(_)) => (),
                (DataType::VecU8, MyValue::VecU8(_)) => (),
                (DataType::VecU16, MyValue::VecU16(_)) => (),
                (DataType::VecU32, MyValue::VecU32(_)) => (),
                (DataType::VecU64, MyValue::VecU64(_)) => (),
                (DataType::VecU128, MyValue::VecU128(_)) => (),
                _ => {
                    return false;
                }
            }
        }
        true
    }

    pub fn validate_types_and_parse(
        &self,
        arg_names: &Vec<Vec<String>>,
        arg_values: &Vec<Vec<Value>>,
        metadata: &Vec<FnCallMetadata>,
    ) -> bool {
        //let metadata = metadata.get(self.metadata_id).expect("Undefined metadata");
        if arg_names.len() != arg_values.len() || arg_values.len() != metadata.len() {
            return false;
        }

        self.validate_types_by_metadata(arg_names, arg_values, metadata, 0)
    }

    pub fn validate_types_by_metadata(
        &self,
        arg_names: &Vec<Vec<String>>,
        arg_values: &Vec<Vec<Value>>,
        metadata: &Vec<FnCallMetadata>,
        metadata_id: usize,
    ) -> bool {
        if arg_names[metadata_id].len() != arg_values[metadata_id].len()
            || arg_values[metadata_id].len() != metadata[metadata_id].arg_types.len()
        {
            return false;
        }

        for i in 0..metadata[metadata_id].arg_names.len() {
            match (
                &metadata[metadata_id].arg_types[i],
                &arg_values[metadata_id][i],
            ) {
                (DataType::Bool, Value::Bool(_)) => (),
                (DataType::String, Value::String(_)) => (),
                (DataType::U8, Value::Number(_)) => (),
                (DataType::U32, Value::Number(_)) => (),
                (DataType::U64, Value::String(v)) => {
                    let _: U64 = serde_json::from_str(v).unwrap();
                }
                (DataType::U128, Value::String(v)) => {
                    let _: U64 = serde_json::from_str(v).unwrap();
                }
                (DataType::VecString, Value::Array(v)) => {
                    for v in v.iter() {
                        if !v.is_string() {
                            panic!("Invalid value")
                        }
                    }
                }
                (DataType::VecU8, Value::Array(v)) => {
                    for v in v.iter() {
                        if !v.is_number() {
                            panic!("Invalid value")
                        }
                    }
                }
                (DataType::VecU16, Value::Array(v)) => {
                    for v in v.iter() {
                        if !v.is_number() {
                            panic!("Invalid value")
                        }
                    }
                }
                (DataType::VecU32, Value::Array(v)) => {
                    for v in v.iter() {
                        if !v.is_number() {
                            panic!("Invalid value")
                        }
                    }
                }
                (DataType::VecU64, Value::Array(v)) => {
                    for v in v.iter() {
                        match v {
                            Value::String(s) => {
                                let _: u64 = serde_json::from_str(s).expect("Failed to parse U64");
                            }
                            _ => panic!("Invalid value"),
                        }
                    }
                }
                (DataType::VecU128, Value::Array(v)) => {
                    for v in v.iter() {
                        match v {
                            Value::String(s) => {
                                let _: u128 =
                                    serde_json::from_str(s).expect("Failed to parse U128");
                            }
                            _ => panic!("Invalid value"),
                        }
                    }
                }
                (DataType::Object(id), _) => {
                    return self.validate_types_by_metadata(
                        arg_names,
                        arg_values,
                        metadata,
                        *id as usize,
                    )
                }
                _ => {
                    panic!("Invalid value")
                }
            }
        }
        true
    }

    /*
    Objest always must be on index > 0 ??

    Object example:
    { name: test, value: { something1: something }, another_value2: { another:another}}

    Input structure:
    ArgIdents = [[name], [something1], [another_value2]]
    ArgValues = [[test], [something], [another]]
    Metadata: = [
        { arg_names: [test, something1, another_value2], arg_types: [String, Object, Object]},
        { arg_names: [something1], arg_types: [String]}
        { arg_names: [another], arg_types: [String]}
    ]
    */

    /// Bind function argument names with values
    /// Returns serialized JSON object
    pub fn bind_args(
        &self,
        arg_names: &Vec<Vec<String>>,
        arg_values: &Vec<Vec<Value>>,
        metadata: &Vec<FnCallMetadata>,
        metadata_id: usize,
    ) -> String {
        //TODO empty string, null values ??

        // Create raw json object string
        let mut args = String::with_capacity(256);
        args.push('{');
        for i in 0..metadata[metadata_id].arg_names.len() {
            assert_eq!(
                metadata[metadata_id].arg_names[i], arg_names[metadata_id][i],
                "arg names must be equals"
            );
            args.push('"');
            args.push_str(arg_names[metadata_id][i].as_str()); //json attribute
            args.push('"');
            args.push(':');
            log!("id {}", i);
            log!("metadata type {:?}", metadata[metadata_id].arg_types[i]);
            match &metadata[metadata_id].arg_types[i] {
                DataType::Object(id) => {
                    args.push_str(
                        self.bind_args(arg_names, arg_values, metadata, *id as usize)
                            .as_str(),
                    );
                }
                _ => {
                    log!("provided arg name {:?}", arg_names[metadata_id][i]);
                    log!("provided arg value {:?}", arg_values[metadata_id][i]);
                    match (
                        &metadata[metadata_id].arg_types[i],
                        &arg_values[metadata_id][i],
                    ) {
                        (DataType::String, Value::String(v)) => {
                            args.push('"');
                            args.push_str(v.as_str());
                            args.push('"');
                        }
                        (DataType::Bool, Value::Bool(v)) => {
                            args.push_str(serde_json::to_string(v).unwrap().as_str());
                        }
                        (DataType::Bool, Value::Number(v)) => {
                            args.push_str(serde_json::to_string(v).unwrap().as_str());
                        }
                        (DataType::VecString, Value::Array(v))
                        | (DataType::VecU8, Value::Array(v))
                        | (DataType::VecU16, Value::Array(v))
                        | (DataType::VecU32, Value::Array(v))
                        | (DataType::VecU64, Value::Array(v))
                        | (DataType::VecU128, Value::Array(v)) => {
                            args.push('[');
                            for v in v.iter() {
                                args.push_str(serde_json::to_string(v).unwrap().as_str());
                                args.push(',');
                            }
                            args.pop();
                            args.push(']');
                            // _ => panic!("Invalid type during parsing");
                        }
                        _ => panic!("Invalid type during parsing"),
                    }
                }
            }
            //if is not arg_values, then must be object

            //match (&metadata[metadata_id].arg_types[i],&arg_values[metadata_id][i]) {
            //    (DataType::String,Value::String(v)) => {
            //        args.push('"');
            //        args.push_str(v.as_str());
            //        args.push('"');
            //    }
            //    (DataType::Bool, Value::Bool(v)) => {
            //        args.push_str(serde_json::to_string(v).unwrap().as_str());
            //    }
            //    (DataType::Bool, Value::Number(v)) => {
            //        args.push_str(serde_json::to_string(v).unwrap().as_str());
            //    }
            //    (DataType::VecString, Value::Array(v)) | (DataType::VecU8, Value::Array(v)) | (DataType::VecU16, Value::Array(v)) | (DataType::VecU32, Value::Array(v))
            //    | (DataType::VecU64, Value::Array(v)) | (DataType::VecU128, Value::Array(v))  => {
            //        for v in v.iter() {
            //            args.push_str(serde_json::to_string(v).unwrap().as_str());
            //        }
            //    }
            //    (DataType::Object(id), Value::Object(_)) => {
            //        args.push_str(self.bind_args(arg_names, arg_values, metadata, *id as usize).as_str());
            //    }
            //    _ => panic!("Invalid type during parsing"),
            //}
            args.push(',');
        }
        args.pop();
        args.push('}');

        log!("args: {}", args);

        args
        //serde_json::from_str(&args).expect("Failed to serialize JSON object")
    }
}

#[near_bindgen]
impl NewDaoContract {
    pub fn propose(&mut self) {
        let caller = env::predecessor_account_id();
        //assert!(self.check_propose_rights(&caller,), "{}", ERR_NO_ACCESS);
        todo!()
    }

    pub fn group_create(
        &mut self,
        proposal_id: ProposalId,
        settings: GroupSettings,
        members: Vec<GroupMember>,
        token_lock: GroupReleaseInput,
    ) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);

        self.add_group(GroupInput {
            settings,
            members,
            release: token_lock,
        });
    }
    pub fn group_remove(&mut self, proposal_id: ProposalId, id: GroupId) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);

        match self.groups.remove(&id) {
            Some(mut group) => {
                let group_release_key = utils::get_group_key(id);
                let release: ReleaseDb = group.remove_storage_data(group_release_key).data.into();
                //TODO check: UPDATE DAO release settings
                self.ft_total_locked -= release.total - release.init_distribution;
            }
            _ => (),
        }
    }
    pub fn group_update(&mut self, proposal_id: ProposalId, id: GroupId, settings: GroupSettings) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);

        match self.groups.get(&id) {
            Some(mut group) => {
                group.settings = settings;
                self.groups.insert(&id, &group);
            }
            _ => (),
        }
    }
    pub fn group_add_members(
        &mut self,
        proposal_id: ProposalId,
        id: GroupId,
        members: Vec<GroupMember>,
    ) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);

        match self.groups.get(&id) {
            Some(mut group) => {
                group.add_members(members);
                self.groups.insert(&id, &group);
            }
            _ => (),
        }
    }
    pub fn group_remove_member(&mut self, proposal_id: ProposalId, id: GroupId, member: AccountId) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        match self.groups.get(&id) {
            Some(mut group) => {
                group.remove_member(member);
                self.groups.insert(&id, &group);
            }
            _ => (),
        }
    }
    pub fn settings_update(&mut self, proposal_id: ProposalId, settings: DaoSettings) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        assert_valid_dao_settings(&settings);
        self.settings.replace(&settings.into());
    }
    pub fn media_add(&mut self, proposal_id: ProposalId, media: Media) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);

        self.media_count += 1;
        self.media.insert(&self.media_count, &media);
    }
    pub fn media_invalidate(&mut self, proposal_id: ProposalId, id: u32) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        match self.media.get(&id) {
            Some(mut media) => {
                media.valid = false;
                self.media.insert(&id, &media);
            }
            _ => (),
        }
    }
    //TODO ??
    pub fn media_remove(&mut self, proposal_id: ProposalId, id: u32) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        self.media.remove(&id);
    }

    pub fn tag_insert(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        tags: Vec<String>,
    ) -> Option<(TagId, TagId)> {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        let mut t = self.tags.get(&category).unwrap_or(Tags::new());
        let ids = t.insert(tags);
        self.tags.insert(&category, &t);
        ids
    }

    pub fn tag_edit(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        id: TagId,
        name: String,
    ) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        match self.tags.get(&category) {
            Some(mut t) => {
                t.rename(id, name);
                self.tags.insert(&category, &t);
            }
            None => (),
        }
    }

    pub fn tag_clear(&mut self, proposal_id: ProposalId, category: TagCategory, id: TagId) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        //TODO implement check for all usage
        match self.tags.get(&category) {
            Some(mut t) => {
                t.remove(id);
                self.tags.insert(&category, &t);
            }
            None => (),
        }
    }

    pub fn ft_unlock(&mut self, proposal_id: ProposalId, group_ids: Vec<GroupId>) -> Vec<u32> {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        let mut released = Vec::with_capacity(group_ids.len());
        for id in group_ids.into_iter() {
            if let Some(mut group) = self.groups.get(&id) {
                released.push(group.unlock_ft(env::block_timestamp()));
                self.groups.insert(&id, &group);
            }
        }
        released
    }
    pub fn ft_distribute(
        &mut self,
        proposal_id: ProposalId,
        group_id: u16,
        amount: u32,
        account_ids: Vec<AccountId>,
    ) -> bool {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        if let Some(mut group) = self.groups.get(&group_id) {
            match group.distribute_ft(amount) && account_ids.len() > 0 {
                true => {
                    self.groups.insert(&group_id, &group);
                    self.distribute_ft(amount, &account_ids);
                    true
                }
                false => false,
            }
        } else {
            false
        }
    }
    pub fn treasury_send_near(
        &mut self,
        proposal_id: ProposalId,
        receiver_id: AccountId,
        amount: U128,
    ) -> Promise {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        Promise::new(receiver_id).transfer(amount.0)
    }
    pub fn treasury_send_ft(
        &mut self,
        proposal_id: ProposalId,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        is_contract: bool,
        amount_ft: U128,
        memo: Option<String>,
        msg: String,
    ) -> Promise {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);

        let promise = Promise::new(ft_account_id);
        if is_contract {
            //TODO test formating memo
            promise.function_call(
                b"ft_transfer_call".to_vec(),
                format!(
                    "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"memo\":\"{}\",\"msg\":\"{}\"}}",
                    receiver_id,
                    amount_ft.0,
                    memo.unwrap_or("".into()),
                    msg
                )
                .as_bytes()
                .to_vec(),
                0,
                TGAS,
            )
        } else {
            promise.function_call(
                b"ft_transfer".to_vec(),
                format!(
                    "{{\"receiver_id\":{},\"amount\":\"{}\",\"msg\":\"{}\"}}",
                    receiver_id, amount_ft.0, msg
                )
                .as_bytes()
                .to_vec(),
                0,
                TGAS,
            )
        }
    }
    //TODO check correct NFT usage
    pub fn treasury_send_nft(
        &mut self,
        proposal_id: ProposalId,
        nft_account_id: AccountId,
        nft_id: String,
        approval_id: String,
        receiver_id: String,
        is_contract: bool,
        memo: Option<String>,
        msg: String,
    ) -> Promise {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        let promise = Promise::new(nft_account_id);
        if is_contract {
            //TODO test formating memo
            promise.function_call(b"nft_transfer_call".to_vec(), format!("{{\"receiver_id\":\"{}\",\"token_id\":\"{}\",\"approval_id\":{},\"memo\":\"{}\",\"msg\":\"{}\"}}", receiver_id, nft_id, approval_id, memo.unwrap_or("".into()), msg).as_bytes().to_vec(), 0, TGAS)
        } else {
            promise.function_call(
                b"nft_transfer".to_vec(),
                format!(
                    "{{\"receiver_id\":{},\"token_id\":\"{}\",\"approval_id\":{},\"memo\":\"{}\"}}",
                    receiver_id,
                    nft_id,
                    approval_id,
                    memo.unwrap_or("".into())
                )
                .as_bytes()
                .to_vec(),
                0,
                TGAS,
            )
        }
    }

    // TODO own Value for inputs and use serde Value for transforming to JSON ??
    // TODO write tests parsing arguments
    /// Invokes registered function call
    /*     pub fn function_call(
        &mut self,
        proposal_id: ProposalId,
        fncall_id: String,
        fncall_arguments: FnCallArguments,
        deposit: U128,
        tgas: u16,
    ) -> Promise {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        let fncall = self.function_calls.get(&fncall_id).expect(ERR_UNKNOWN_FNCALL);
        fncall.validate_input_types(fncall_arguments);

        // TODO get constrains and binds from workflow template and postprocessing
        // Should be some match for Option

        // TODO validate fn args
        fncall.bind_and_execute()

        //add postprocessing (save promise result - must be from workflow)

    } */

    pub fn fn_call_validity_test(
        &self,
        fncall_id: String,
        names: Vec<Vec<String>>,
        args: Vec<Vec<Value>>,
    ) -> Promise {
        let fncall = self.function_calls.get(&fncall_id).unwrap();
        let metadata = self.function_call_metadata.get(&fncall_id).unwrap();

        //assert!(fncall.validate_types_and_parse(&names, &args, &metadata)); Maybe just throw runtime err during validation ??

        let args = fncall.bind_args(&names, &args, &metadata, 0);
        Promise::new(fncall.receiver).function_call(
            fncall.name.into_bytes(),
            args.into_bytes(),
            0,
            12 * TGAS,
        )
    }

    pub fn test(
        &self,
        name1: String,
        name2: Vec<String>,
        name3: Vec<U128>,
        obj: InnerTestObject,
    ) -> bool {
        log!("args: ");
        dbg!("{}, {}, {},{}", name1, name2, name3, obj);
        true
    }

    pub fn function_call_add(&mut self, proposal_id: ProposalId, func: FnCallDefinition) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        let id = format!("{}_{}", func.receiver, func.name);
        self.function_calls.insert(&id, &func);
    }
    //TODO key as ID or func name
    pub fn function_call_remove(&mut self, proposal_id: ProposalId, id: String) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        self.function_calls.remove(&id);
    }

    pub fn workflow_install(&mut self, proposal_id: ProposalId) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        todo!()
    }
    pub fn workflow_add(&mut self, proposal_id: ProposalId) {
        assert!(self.check_action_rights(proposal_id), "{}", ERR_NO_ACCESS);
        todo!()
    }

    // TODO workflow settings??
}
