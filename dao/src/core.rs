use crate::constants::MAX_FT_TOTAL_SUPPLY;
use crate::settings::{assert_valid_dao_settings, DaoSettings, VDaoSettings};
use crate::standard_impl::ft::FungibleToken;
use crate::standard_impl::ft_metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider};
use crate::tags::{TagInput, Tags};
use library::storage::StorageBucket;
use library::types::{DataType, FnCallMetadata};
use library::workflow::InstanceState;
use library::{
    workflow::{Instance, ProposeSettings, Template, TemplateSettings},
    FnCallId,
};

use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::Serialize;
use near_sdk::{
    env, log, near_bindgen, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault, Promise,
    PromiseOrValue,
};

use crate::group::{Group, GroupInput};

use crate::media::Media;
use crate::{calc_percent_u128_unchecked, proposal::*, StorageKey, TagCategory};
use crate::{GroupId, ProposalId};

near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ActivityLog {
    pub caller: AccountId,
    pub action_id: u8,
    pub timestamp: u64,
    pub args: Vec<Vec<DataType>>,
    pub args_collections: Option<Vec<Vec<DataType>>>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    FT,
    FTMetadata,
    Proposals,
    ProposalConfig,
    Council,
    Tags,
    Media,
    FunctionCalls,
    FunctionCallMetadata,
    Storage,
    DaoSettings,
    VoteSettings,
    VConfig,
    ReleaseConfig,
    ReleaseDb,
    DocMetadata,
    Mappers,
    NewVersionCode,
    FactoryAcc,
    TokenGroupRights,
    UserRights,
    StorageDeposit,
    RefPools,
    SkywardAuctions,
    Groups,
    Rights,
    FunctionCallWhitelist,
    WfTemplate,
    WfTemplateSettings,
    ProposedWfTemplateSettings, //for proposal workflow add template
    ActivityLog,
    WfInstance,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub ft_total_supply: u32,
    pub ft_total_locked: u32,
    pub ft_total_distributed: u32,
    pub total_members_count: u32,
    pub decimal_const: u128,
    pub ft: FungibleToken,
    pub ft_metadata: LazyOption<FungibleTokenMetadata>,
    pub group_last_id: GroupId,
    pub groups: UnorderedMap<GroupId, Group>,
    pub settings: LazyOption<VDaoSettings>,
    pub proposal_last_id: u32,
    pub proposals: UnorderedMap<u32, VProposal>,
    pub storage: UnorderedMap<StorageKey, StorageBucket>,
    pub tags: UnorderedMap<TagCategory, Tags>,
    pub media_last_id: u32,
    pub media: LookupMap<u32, Media>,
    pub function_call_metadata: UnorderedMap<FnCallId, Vec<FnCallMetadata>>,
    pub workflow_last_id: u16,
    pub workflow_template: UnorderedMap<u16, (Template, Vec<TemplateSettings>)>,
    pub workflow_instance: UnorderedMap<u32, (Instance, ProposeSettings)>,
    pub proposed_workflow_settings: LookupMap<ProposalId, Vec<TemplateSettings>>,
    pub workflow_activity_log: LookupMap<ProposalId, Vec<ActivityLog>>, // Logs will be moved to indexer when its ready
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        total_supply: u32,
        ft_metadata: FungibleTokenMetadata,
        settings: DaoSettings,
        groups: Vec<GroupInput>,
        media: Vec<Media>,
        tags: Vec<TagInput>,
        function_calls: Vec<FnCallId>,
        function_call_metadata: Vec<Vec<FnCallMetadata>>,
        workflow_templates: Vec<Template>,
        workflow_template_settings: Vec<Vec<TemplateSettings>>,
    ) -> Self {
        assert!(total_supply <= MAX_FT_TOTAL_SUPPLY);
        assert_valid_dao_settings(&settings);

        let mut contract = Contract {
            ft_total_supply: total_supply,
            ft_total_locked: 0,
            ft_total_distributed: 0,
            total_members_count: 0,
            decimal_const: 10u128.pow(ft_metadata.decimals as u32),
            ft: FungibleToken::new(StorageKeys::FT),
            ft_metadata: LazyOption::new(StorageKeys::FTMetadata, Some(&ft_metadata)),
            settings: LazyOption::new(StorageKeys::DaoSettings, None),
            group_last_id: 0,
            groups: UnorderedMap::new(StorageKeys::Groups),
            proposal_last_id: 0,
            proposals: UnorderedMap::new(StorageKeys::Proposals),
            storage: UnorderedMap::new(StorageKeys::Storage),
            tags: UnorderedMap::new(StorageKeys::Tags),
            media_last_id: 0,
            media: LookupMap::new(StorageKeys::Media),
            function_call_metadata: UnorderedMap::new(StorageKeys::FunctionCallMetadata),
            workflow_last_id: 0,
            workflow_template: UnorderedMap::new(StorageKeys::WfTemplate),
            workflow_instance: UnorderedMap::new(StorageKeys::WfInstance),
            proposed_workflow_settings: LookupMap::new(StorageKeys::ProposedWfTemplateSettings),
            workflow_activity_log: LookupMap::new(StorageKeys::ActivityLog),
        };

        //register self and mint all FT

        let contract_acc = env::current_account_id();
        contract.ft.internal_register_account(&contract_acc);
        contract.ft.internal_deposit(
            &contract_acc,
            contract.ft_total_supply as u128 * contract.decimal_const,
        );
        //substract self account
        contract.ft.token_holders_count -= 1;

        contract.init_dao_settings(settings);
        contract.init_tags(tags);
        contract.init_groups(groups);
        contract.init_media(media);
        contract.init_function_calls(function_calls, function_call_metadata);
        contract.init_workflows(workflow_templates, workflow_template_settings);

        contract
    }

    #[private]
    #[init(ignore_state)]
    pub fn upgrade() -> Self {
        assert!(env::storage_remove(
            &StorageKeys::NewVersionCode.into_storage_key()
        ));
        let mut _dao: Contract = env::state_read().expect("Failed to migrate");

        // ADD migration here

        _dao
    }

    #[payable]
    pub fn propose(
        &mut self,
        template_id: u16,
        template_settings_id: u8,
        propose_settings: ProposeSettings,
        template_settings: Option<Vec<TemplateSettings>>,
        desc: String,
        content: Option<ProposalContent>,
    ) -> u32 {
        let caller = env::predecessor_account_id();
        let (wft, wfs) = self.workflow_template.get(&template_id).unwrap();
        let settings = wfs
            .get(template_settings_id as usize)
            .expect("Undefined settings id");

        assert!(env::attached_deposit() >= settings.deposit_propose.unwrap_or(0.into()).0);

        if !self.check_rights(&settings.allowed_proposers, &caller) {
            panic!("You have no rights to propose this");
        }
        self.proposal_last_id += 1;

        //Assuming template_id for WorkflowAdd is always first wf added during dao init
        if template_id == 1 {
            assert!(
                template_settings.is_some(),
                "{}",
                "Expected template settings for 'WorkflowAdd' proposal"
            );
            self.proposed_workflow_settings
                .insert(&self.proposal_last_id, &template_settings.unwrap());
        }

        let proposal = Proposal::new(
            env::block_timestamp() / 10u64.pow(9) / 60 * 60 + 60, // Rounded up to minutes
            caller,
            template_id,
            template_settings_id,
            true,
            desc,
            content,
        );
        assert!(self.storage.get(&propose_settings.storage_key).is_none());

        self.proposals
            .insert(&self.proposal_last_id, &VProposal::Curr(proposal));
        self.workflow_instance.insert(
            &self.proposal_last_id,
            &(
                Instance::new(template_id, &wft.transitions),
                propose_settings,
            ),
        );

        self.proposal_last_id
    }

    #[payable]
    pub fn vote(&mut self, proposal_id: u32, vote_kind: u8) -> bool {
        if vote_kind > 2 {
            return false;
        }

        let caller = env::predecessor_account_id();
        let (mut proposal, _, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(env::attached_deposit() >= wfs.deposit_vote.unwrap_or(0.into()).0);

        if !self.check_rights(&[wfs.allowed_voters.clone()], &caller) {
            return false;
        }

        if proposal.state != ProposalState::InProgress
            || proposal.created + (wfs.duration as u64) < env::block_timestamp() / 10u64.pow(9)
        {
            //TODO update expired proposal state
            return false;
        }

        if wfs.vote_only_once && proposal.votes.contains_key(&caller) {
            return false;
        }

        proposal.votes.insert(caller, vote_kind);

        self.proposals
            .insert(&proposal_id, &VProposal::Curr(proposal));

        true
    }

    pub fn finish_proposal(&mut self, proposal_id: u32) -> ProposalState {
        let (mut proposal, _, wfs) = self.get_wf_and_proposal(proposal_id);
        let (mut instance, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        let new_state = match proposal.state {
            ProposalState::InProgress => {
                if proposal.created + wfs.duration as u64 > env::block_timestamp() / 10u64.pow(9) {
                    None
                } else {
                    // count votes
                    let (max_possible_amount, vote_results) =
                        self.calculate_votes(&proposal.votes, &wfs.scenario, &wfs.allowed_voters);
                    log!("Votes: {}, {:?}", max_possible_amount, vote_results);
                    // check spam
                    if calc_percent_u128_unchecked(
                        vote_results[0],
                        max_possible_amount,
                        self.decimal_const,
                    ) >= wfs.spam_threshold
                    {
                        Some(ProposalState::Spam)
                    } else if calc_percent_u128_unchecked(
                        vote_results.iter().sum(),
                        max_possible_amount,
                        self.decimal_const,
                    ) < wfs.quorum
                    {
                        // not enough quorum
                        Some(ProposalState::Invalid)
                    } else if calc_percent_u128_unchecked(
                        vote_results[1],
                        vote_results.iter().sum(),
                        self.decimal_const,
                    ) < wfs.approve_threshold
                    {
                        // not enough voters to accept
                        Some(ProposalState::Rejected)
                    } else {
                        // proposal is accepted, create new workflow activity with its storage
                        instance.state = InstanceState::Running;
                        // Storage key must be unique among all proposals
                        self.storage_bucket_add(settings.storage_key.as_str());
                        Some(ProposalState::Accepted)
                    }
                }
            }
            _ => None,
        };

        match new_state {
            Some(state) => {
                self.workflow_instance
                    .insert(&proposal_id, &(instance, settings));
                proposal.state = state.clone();
                self.proposals
                    .insert(&proposal_id, &VProposal::Curr(proposal));
                state
            }
            None => proposal.state,
        }
    }

    //TODO resolve other state combinations eg. FatalError on instance.
    /// Changes workflow instance state to finish.
    /// Rights to close are same as the "end" activity rights.
    pub fn wf_finish(&mut self, proposal_id: u32) -> bool {
        let caller = env::predecessor_account_id();
        let (proposal, wft, wfs) = self.get_wf_and_proposal(proposal_id);

        assert!(proposal.state == ProposalState::Accepted);

        let (mut wfi, settings) = self.workflow_instance.get(&proposal_id).unwrap();

        if wfi.state == InstanceState::FatalError
            || self.check_rights(
                &wfs.activity_rights[wfi.current_activity_id as usize - 1].as_slice(),
                &caller,
            )
        {
            let result = wfi.finish(&wft);
            self.workflow_instance
                .insert(&proposal_id, &(wfi, settings));

            result
        } else {
            false
        }
    }

    pub fn ft_unlock(&mut self, group_ids: Vec<GroupId>) -> Vec<u32> {
        let mut released = Vec::with_capacity(group_ids.len());
        for id in group_ids.into_iter() {
            if let Some(mut group) = self.groups.get(&id) {
                released.push(group.unlock_ft(env::block_timestamp()));
                self.groups.insert(&id, &group);
            }
        }
        released
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn clean_self(&mut self) {
        self.workflow_template.clear();
        self.workflow_instance.clear();
        self.function_call_metadata.clear();

        let settings: DaoSettings = self.settings.get().unwrap().into();
        assert_eq!(settings.dao_admin_account_id, env::predecessor_account_id());
        env::storage_remove(&StorageKeys::NewVersionCode.into_storage_key());
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn delete_self(&mut self) -> Promise {
        let settings: DaoSettings = self.settings.get().unwrap().into();
        assert_eq!(settings.dao_admin_account_id, env::predecessor_account_id());
        Promise::new(env::current_account_id()).delete_account(settings.dao_admin_account_id)
    }
}

/******************************************************************************
 *
 * Fungible Token (NEP-141)
 * https://nomicon.io/Standards/FungibleToken/Core.html
 *
 ******************************************************************************/

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>) {
        self.ft.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.ft.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.ft.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: ValidAccountId) -> U128 {
        self.ft.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: ValidAccountId,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> U128 {
        let sender_id: AccountId = sender_id.into();
        let (used_amount, burned_amount) =
            self.ft
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            //self.on_tokens_burned(sender_id, burned_amount);
        }
        used_amount.into()
    }
}

/******************************************************************************
 *
 * Fungible Token Metadata (NEP-148)
 * https://nomicon.io/Standards/FungibleToken/Metadata.html
 *
 ******************************************************************************/

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.ft_metadata.get().unwrap()
    }
}

/******************************************************************************
 *
 * Storage Management (NEP-145)
 * https://nomicon.io/Standards/StorageManagement.html
 *
 ******************************************************************************/

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.ft.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        self.ft.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.ft.internal_storage_unregister(force) {
            //self.on_account_closed(account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.ft.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        self.ft.storage_balance_of(account_id)
    }
}

/// Triggers new version download from factory
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    use crate::constants::{GAS_DOWNLOAD_NEW_VERSION, VERSION};
    use env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

    //TODO who can trigger the download
    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
    );

    let factory_acc = dao_settings.dao_admin_account_id;
    let method_name = b"download_dao_bin".to_vec();

    unsafe {
        BLOCKCHAIN_INTERFACE.with(|b| {
            b.borrow().as_ref().unwrap().promise_create(
                factory_acc.len() as _,
                factory_acc.as_ptr() as _,
                method_name.len() as _,
                method_name.as_ptr() as _,
                1 as _,
                [VERSION].to_vec().as_ptr() as _,
                0,
                GAS_DOWNLOAD_NEW_VERSION,
            );
        });
    }
}

/// Method called by dao factory as response to download_new_version method
/// Saves provided dao binary in storage under "NewVersionCode"
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn store_new_version() {
    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    let contract: Contract = env::state_read().unwrap();
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
    );
    env::storage_write(
        &StorageKeys::NewVersionCode.into_storage_key(),
        &env::input().unwrap(),
    );
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn upgrade_self() {
    use crate::constants::GAS_UPGRADE;
    use near_sdk::env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    // We are not able to access council members any other way so we have deserialize SC
    let contract: Contract = env::state_read().unwrap();
    let dao_settings: DaoSettings = contract.settings.get().unwrap().into();

    assert_eq!(
        dao_settings.dao_admin_account_id,
        env::predecessor_account_id()
    );

    let current_acc = env::current_account_id().into_bytes();
    let method_name = "upgrade".as_bytes().to_vec();
    let key = StorageKeys::NewVersionCode.into_storage_key();

    unsafe {
        BLOCKCHAIN_INTERFACE.with(|b| {
            // Load stored wasm code into register 0.
            b.borrow()
                .as_ref()
                .unwrap()
                .storage_read(key.len() as _, key.as_ptr() as _, 0);
            // schedule a Promise tx to this same contract
            let promise_id = b
                .borrow()
                .as_ref()
                .unwrap()
                .promise_batch_create(current_acc.len() as _, current_acc.as_ptr() as _);
            // 1st item in the Tx: "deploy contract" (code is taken from register 0)
            b.borrow()
                .as_ref()
                .unwrap()
                .promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd item in the Tx: call this account's migrate() method
            b.borrow()
                .as_ref()
                .unwrap()
                .promise_batch_action_function_call(
                    promise_id,
                    method_name.len() as _,
                    method_name.as_ptr() as _,
                    0 as _,
                    0 as _,
                    0 as _,
                    GAS_UPGRADE,
                );
        });
    }
}

pub struct StorageKeyWrapper(pub Vec<u8>);

impl IntoStorageKey for StorageKeyWrapper {
    fn into_storage_key(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for StorageKeyWrapper {
    fn from(bytes: Vec<u8>) -> StorageKeyWrapper {
        StorageKeyWrapper(bytes)
    }
}
