use std::u128;

use crate::constants::{
    DEPOSIT_ADD_PROPOSAL, DEPOSIT_VOTE, GAS_ADD_PROPOSAL, GAS_FINISH_PROPOSAL, GROUP_PREFIX,
    MAX_FT_TOTAL_SUPPLY, METADATA_MAX_DECIMALS, PROPOSAL_KIND_COUNT,
};
use crate::settings::{
    assert_valid_dao_settings, assert_valid_vote_settings, DaoSettings, VDaoSettings,
    VVoteSettings, VoteSettings,
};
use crate::standard_impl::ft::FungibleToken;
use crate::standard_impl::ft_metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider};
use crate::storage::{StorageBucket, StorageData};
use crate::tags::{TagInput, Tags};
use crate::workflow::{WorkflowInstance, WorkflowSettings, WorkflowTemplate};
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde_json::Value;
use near_sdk::Promise;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault, PromiseOrValue,
};

use crate::group::{Group, GroupInput};

use crate::media::Media;
use crate::release::{ReleaseDb, ReleaseModel, ReleaseModelInput, VReleaseDb, VReleaseModel};
use crate::{action::*, GroupId, GroupName};
use crate::{calc_percent_u128_unchecked, FnCallId};
use crate::{proposal::*, StorageKey, TagCategory};

near_sdk::setup_alloc!();

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
    WfInstance,
}

// ------  NEW smartcontract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NewDaoContract {
    pub deposit_min_vote: u128,
    pub deposit_min_add_proposal: u128,
    pub ft_total_supply: u32,
    pub ft_total_locked: u32,
    pub ft_total_distributed: u32,
    pub decimal_const: u128,
    pub ft: FungibleToken,
    pub ft_metadata: LazyOption<FungibleTokenMetadata>,
    pub group_last_id: GroupId,
    pub groups: UnorderedMap<GroupId, Group>, //TODO use name as key??
    pub settings: LazyOption<VDaoSettings>,
    pub vote_settings: LazyOption<Vec<VVoteSettings>>,
    pub proposal_last_id: u32,
    pub proposals: UnorderedMap<u32, VProposal>,
    pub storage: UnorderedMap<StorageKey, StorageBucket>, // TODO
    pub tags: UnorderedMap<TagCategory, Tags>, //Once added cannot be removed or special DT??
    pub media_last_id: u32,
    pub media: LookupMap<u32, Media>, //TODO categorize??
    pub function_call_metadata: LookupMap<FnCallId, Vec<FnCallMetadata>>,
    pub function_calls: UnorderedMap<FnCallId, FnCallDefinition>,
    pub workflow_last_id: u16,
    pub workflow_template: UnorderedMap<u16, (WorkflowTemplate, Vec<WorkflowSettings>)>,
    pub workflow_instance: UnorderedMap<u32, WorkflowInstance>,
}

#[near_bindgen]
impl NewDaoContract {
    #[init]
    pub fn new(
        deposit_min_vote: U128,
        deposit_min_add_proposal: U128,
        total_supply: u32,
        ft_metadata: FungibleTokenMetadata,
        settings: DaoSettings,
        vote_settings: Vec<VoteSettings>,
        groups: Vec<GroupInput>,
        media: Vec<Media>,
        tags: Vec<TagInput>,
        function_calls: Vec<FnCallDefinition>,
        function_call_metadata: Vec<Vec<FnCallMetadata>>,
        workflow_template_settings: Vec<Vec<WorkflowSettings>>,
        workflow_templates: Vec<WorkflowTemplate>,
    ) -> Self {
        assert!(total_supply <= MAX_FT_TOTAL_SUPPLY);
        assert_valid_dao_settings(&settings);
        assert_valid_vote_settings(&vote_settings);

        let mut contract = NewDaoContract {
            deposit_min_vote: deposit_min_vote.0,
            deposit_min_add_proposal: deposit_min_add_proposal.0,
            ft_total_supply: total_supply,
            ft_total_locked: 0,
            ft_total_distributed: 0,
            decimal_const: 10u128.pow(ft_metadata.decimals as u32),
            ft: FungibleToken::new(StorageKeys::FT),
            ft_metadata: LazyOption::new(StorageKeys::FTMetadata, Some(&ft_metadata)),
            settings: LazyOption::new(StorageKeys::DaoSettings, None),
            vote_settings: LazyOption::new(StorageKeys::VoteSettings, None),
            group_last_id: 0,
            groups: UnorderedMap::new(StorageKeys::Groups),
            proposal_last_id: 0,
            proposals: UnorderedMap::new(StorageKeys::Proposals),
            storage: UnorderedMap::new(StorageKeys::Storage),
            tags: UnorderedMap::new(StorageKeys::Tags),
            media_last_id: 0,
            media: LookupMap::new(StorageKeys::Media),
            function_call_metadata: LookupMap::new(StorageKeys::FunctionCallMetadata),
            function_calls: UnorderedMap::new(StorageKeys::FunctionCalls),
            workflow_last_id: 0,
            workflow_template: UnorderedMap::new(StorageKeys::WfTemplate),
            workflow_instance: UnorderedMap::new(StorageKeys::WfInstance),
        };

        //register self and mint all FT
        let contract_acc = env::current_account_id();
        contract.ft.internal_register_account(&contract_acc);
        contract.ft.internal_deposit(
            &contract_acc,
            contract.ft_total_supply as u128 * contract.decimal_const,
        );

        contract.init_dao_settings(settings);
        contract.init_vote_settings(vote_settings);
        contract.init_tags(tags);
        contract.init_groups(groups);
        contract.init_media(media);
        contract.init_function_calls(function_calls, function_call_metadata);
        contract.init_workflows(workflow_templates, workflow_template_settings);

        contract
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn clean_self(&mut self) {
        env::storage_remove(&StorageKeys::NewVersionCode.into_storage_key());
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn delete_self(self) -> Promise {
        let settings: DaoSettings = self.settings.get().unwrap().into();
        Promise::new(env::current_account_id()).delete_account(settings.dao_admin_account_id)
    }
}

/*
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DaoContract {
    pub factory_acc: String,
    pub decimal_const: u128,
    pub ft_total_supply: u32,
    pub ft_total_distributed: u32,
    pub proposal_count: u32,
    pub config: LazyOption<VConfig>,
    pub council: UnorderedSet<AccountId>,
    pub group_rights: LookupMap<TokenGroup, Vec<(ActionGroupRight, TimeInterval)>>,
    pub user_rights: LookupMap<AccountId, Vec<(ActionGroupRight, TimeInterval)>>,
    pub ft_metadata: LazyOption<FungibleTokenMetadata>,
    pub ft: FungibleToken,
    pub proposals: UnorderedMap<u32, VProposal>,
    pub release_config: LookupMap<TokenGroup, VReleaseModel>, //TODO merge with release_db
    pub release_db: LookupMap<TokenGroup, VReleaseDb>,
    pub vote_policy_config: LookupMap<ProposalKindIdent, VVoteConfig>,
    pub doc_metadata: UnorderedMap<String, VFileMetadata>,
    pub mappers: UnorderedMap<MapperKind, Mapper>,
    pub storage_deposit: UnorderedSet<AccountId>,
    pub ref_pools: LazyOption<Vec<u32>>,
    pub skyward_auctions: LazyOption<Vec<u64>>,

    pub groups: UnorderedMap<GroupName, Group>, //TODO iterate over LookupMap with use of last_group_key ?
    pub rights: LookupMap<AccountId, Vec<ExecutionRight>>,
    pub function_call_whitelist: LazyOption<Vec<String>>,
}
*/

/*
#[near_bindgen]
impl DaoContract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        // TODO: ON EACH migration migrate config, policy, release model etc !!!

        //We remove this new version from storage so we do not have to pay NEARs for unneccessary storage
        assert!(env::storage_remove(
            &StorageKeys::NewVersionCode.into_storage_key()
        ));
        let mut _dao: DaoContract = env::state_read().expect("Failed to migrate");

        // Migration example process
        /*

        let mut config =  dao.config.get().unwrap();
        config.migrate();
        dao.config.set(&config);

        let mut vote_policy = dao.vote_policy_config.get(&ProposalKindIdent::Pay).unwrap().migrate();
        dao.vote_policy_config.insert(&ProposalKindIndet::Pay);
        ... Migrate all

        let mut release_model = dao.release_config.get().unwrap();
        release_config.migrate();
        dao.release_config.set(&release_model);

        let mut docs = dao.doc_metadata.to_vec();
        for (uuid, doc) in docs.into_ter() {
            dao.insert(&uuid, &doc.migrate());
        }

        */

        _dao
    }

    #[init]
    pub fn new(
        total_supply: u32,
        founders_init_distribution: u32,
        ft_metadata: FungibleTokenMetadata,
        config: ConfigInput,
        release_config: Vec<(TokenGroup, ReleaseModelInput)>,
        vote_policy_configs: Vec<VoteConfigInput>,
        mut founders: Vec<AccountId>,
    ) -> Self {
        assert!(total_supply <= MAX_FT_TOTAL_SUPPLY);
        assert!(ft_metadata.decimals <= METADATA_MAX_DECIMALS);
        assert_eq!(vote_policy_configs.len(), PROPOSAL_KIND_COUNT as usize);

        ft_metadata.assert_valid();
        assert_valid_founders(&mut founders);
        assert_valid_init_config(&config);
        assert!(
            total_supply as u64 * config.council_share.unwrap_or_default() as u64 / 100
                >= founders_init_distribution as u64,
            "{}",
            "Founders init distribution cannot be larger than their total amount share"
        );

        let amount_per_founder: u32 = founders_init_distribution / founders.len() as u32;

        let decimal_const = 10u128.pow(ft_metadata.decimals as u32);

        let mut contract = DaoContract {
            factory_acc: env::predecessor_account_id(),
            config: LazyOption::new(StorageKeys::VConfig, Some(&VConfig::from(config))),
            council: UnorderedSet::new(StorageKeys::Council),
            group_rights: LookupMap::new(StorageKeys::TokenGroupRights),
            user_rights: LookupMap::new(StorageKeys::UserRights),
            ft_metadata: LazyOption::new(StorageKeys::FTMetadata, Some(&ft_metadata)),
            ft: FungibleToken::new(StorageKeys::FT),
            ft_total_supply: total_supply,
            ft_total_distributed: founders_init_distribution,
            decimal_const: decimal_const,
            proposals: UnorderedMap::new(StorageKeys::Proposals),
            proposal_count: 0,
            release_config: LookupMap::new(StorageKeys::ReleaseConfig),
            release_db: LookupMap::new(StorageKeys::ReleaseDb),
            vote_policy_config: LookupMap::new(StorageKeys::ProposalConfig),
            doc_metadata: UnorderedMap::new(StorageKeys::DocMetadata),
            mappers: UnorderedMap::new(StorageKeys::Mappers),
            storage_deposit: UnorderedSet::new(StorageKeys::StorageDeposit),
            ref_pools: LazyOption::new(StorageKeys::RefPools, Some(&Vec::new())),
            skyward_auctions: LazyOption::new(StorageKeys::SkywardAuctions, Some(&Vec::new())),

            groups: UnorderedMap::new(StorageKeys::Groups),
            rights: LookupMap::new(StorageKeys::Rights),
            function_call_whitelist: LazyOption::new(
                StorageKeys::FunctionCallWhitelist,
                Some(&Vec::new()),
            ),
        };

        //contract.setup_groups(),
        contract.setup_voting_policy(vote_policy_configs);
        contract.setup_release_models(release_config, founders_init_distribution);
        contract.init_mappers();

        //register contract account and transfer all total supply of GT to it
        contract
            .ft
            .internal_register_account(&env::current_account_id());
        contract.ft.internal_deposit(
            &env::current_account_id(),
            contract.ft_total_supply as u128 * contract.decimal_const,
        );

        // register council and distribute them their amount of the tokens
        for founder in founders.iter() {
            contract.ft.internal_register_account(&founder);

            contract.ft.internal_transfer(
                &env::current_account_id(),
                &founder,
                amount_per_founder as u128 * contract.decimal_const,
                None,
            );
            contract.council.insert(founder);
        }

        contract.ft.registered_accounts_count -= 1;

        // We store factory acc directly into trie so we dont have to deserialize SC when we upgrade/migrate
        env::storage_write(
            &StorageKeys::FactoryAcc.into_storage_key(),
            &env::predecessor_account_id().into_bytes(),
        );

        contract
    }

    #[payable]
    pub fn add_proposal(&mut self, proposal_input: ProposalInput, tx_input: TxInput) -> u32 {
        let predeccesor_account_id = env::predecessor_account_id();

        assert!(env::attached_deposit() >= DEPOSIT_ADD_PROPOSAL);
        assert!(env::prepaid_gas() >= GAS_ADD_PROPOSAL);
        proposal_input.assert_valid();

        let vote_policy = VoteConfig::from(
            self.vote_policy_config
                .get(&ProposalKindIdent::get_ident_from(&tx_input))
                .expect("Invalid proposal input"),
        );

        let tx = self
            .create_tx(tx_input, &predeccesor_account_id, env::block_timestamp())
            .unwrap();

        self.proposal_count += 1;

        let proposal = Proposal::new(
            self.proposal_count,
            predeccesor_account_id,
            proposal_input,
            tx,
            vote_policy,
            env::block_timestamp(),
        );

        self.proposals
            .insert(&self.proposal_count, &VProposal::Curr(proposal));
        self.proposal_count
    }

    /// vote_kind values: 0 = spam, 1 = yes, 2 = no
    #[payable]
    pub fn vote(&mut self, proposal_id: u32, vote_kind: u8) -> VoteResult {
        let predeccesor_account_id = env::predecessor_account_id();

        assert!(env::prepaid_gas() >= GAS_VOTE);
        assert!(env::attached_deposit() >= DEPOSIT_VOTE);
        assert!(predeccesor_account_id != self.factory_acc);

        let mut proposal =
            Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));

        if proposal.status != ProposalState::InProgress
            || proposal.duration_to <= env::block_timestamp()
        {
            return VoteResult::VoteEnded;
        }

        if vote_kind > 2 {
            return VoteResult::InvalidVote;
        }

        if proposal.vote_only_once && proposal.votes.contains_key(&predeccesor_account_id) {
            return VoteResult::AlreadyVoted;
        }

        proposal.votes.insert(predeccesor_account_id, vote_kind);

        self.proposals
            .insert(&proposal_id, &VProposal::Curr(proposal));
        VoteResult::Ok
    }

    pub fn finish_proposal(&mut self, proposal_id: u32) -> ProposalState {
        assert!(env::prepaid_gas() >= GAS_FINISH_PROPOSAL);
        let mut proposal =
            Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));

        let new_status = match &proposal.status {
            &ProposalState::InProgress => {
                if env::block_timestamp() < proposal.duration_to {
                    None
                } else {
                    // count votes
                    let mut votes = [0 as u128; 3];
                    for (voter, vote_value) in proposal.votes.iter() {
                        votes[*vote_value as usize] += self.ft.accounts.get(voter).unwrap_or(0);
                    }

                    let total_voted_amount: u128 = votes.iter().sum();

                    // we need to read config just because of spam TH value - could be moved to voting ??
                    let config = Config::from(self.config.get().unwrap());

                    // check spam
                    if calc_percent_u128_unchecked(votes[0], total_voted_amount, self.decimal_const)
                        >= config.vote_spam_threshold
                    {
                        Some(ProposalState::Spam)
                    } else if calc_percent_u128_unchecked(
                        total_voted_amount,
                        self.ft_total_distributed as u128 * self.decimal_const,
                        self.decimal_const,
                    ) < proposal.quorum
                    {
                        // not enough quorum
                        Some(ProposalState::Invalid)
                    } else if calc_percent_u128_unchecked(
                        votes[1],
                        total_voted_amount,
                        self.decimal_const,
                    ) < proposal.approve_threshold
                    {
                        // not enough voters to accept
                        Some(ProposalState::Rejected)
                    } else {
                        // proposal is accepted, try to execute transaction
                        if let Err(errors) = self.execute_tx(
                            &proposal.transactions,
                            Context {
                                proposal_id: proposal.uuid,
                                attached_deposit: env::attached_deposit(),
                                current_balance: env::account_balance(),
                                current_block_timestamp: env::block_timestamp(),
                            },
                        ) {
                            log!("errors: {:?}", errors);
                            Some(ProposalState::Invalid)
                        } else {
                            Some(ProposalState::Accepted)
                        }
                    }
                }
            }
            _ => None,
        };

        match new_status {
            Some(status) => {
                proposal.status = status.clone();
                self.proposals
                    .insert(&proposal.uuid.clone(), &VProposal::Curr(proposal));
                status
            }
            None => proposal.status,
        }
    }

    /// Returns amount of newly unlocked tokens
    pub fn unlock_tokens(&mut self, group: TokenGroup) -> u32 {
        let model: ReleaseModel = self.release_config.get(&group).unwrap().into();
        let mut db: ReleaseDb = self.release_db.get(&group).unwrap().into();

        if db.total == db.unlocked {
            return 0;
        }

        let total_released_now = model.release(
            db.total,
            db.init_distribution,
            db.unlocked,
            (env::block_timestamp() / 10u64.pow(9)) as u32,
        );

        if total_released_now > 0 {
            let delta = total_released_now - (db.unlocked - db.init_distribution);
            db.unlocked += delta;
            self.release_db.insert(&group, &VReleaseDb::Curr(db));
            delta
        } else {
            total_released_now
        }
    }

    /// Allows privileged users to call and execute actions directly as DAO
    pub fn execute_privileged_action(&mut self, action: ActionGroupInput) -> Promise {
        let caller = env::predecessor_account_id();

        // check caller has right to do this
        let mut user_rights = self.user_rights.get(&caller);

        if user_rights.is_none() {
            let group = self.get_users_group(&caller);
            assert!(group.is_some());

            user_rights = self.group_rights.get(&group.unwrap());
            assert!(user_rights.is_some())
        }

        let group_right = match action {
            ActionGroupInput::SkyCreateSale { .. } => ActionGroupRight::SkywardFinance,
            _ => ActionGroupRight::RefFinance,
        };

        let mut right_to_call = false;
        for (right, interval) in user_rights.unwrap().iter() {
            if *right == group_right
                && interval.from <= env::block_timestamp()
                && interval.to >= env::block_timestamp()
            {
                right_to_call = true;
                break;
            }
        }

        assert!(right_to_call, "You have no rights to call this action");

        self.execute_privileged_action_group_call(action)
    }

    pub fn execute_action(&mut self, action: Action) -> PromiseOrValue<Result<(), String>> {
        unimplemented!();
        //PromiseOrValue::Value(ActionResult::Success)
    }

    pub fn add_group(&mut self, group_input: GroupInput) {
        assert!(self.groups.get(&group_input.name).is_none());
        let hash = env::sha256(group_input.name.as_bytes());
        let gkey: StorageKeyWrapper = to_storage_key_raw(GROUP_PREFIX, &hash).into();
        let rkey: StorageKeyWrapper = to_storage_key_raw(GROUP_RELEASE_MODEL_SUFFIX, &hash).into();

        self.groups.insert(
            &group_input.name,
            &Group {
                members: UnorderedSet::new(gkey),
                release: LazyOption::new(rkey, None),
            },
        );
    }

    pub fn remove_group(&mut self, group_name: GroupName) {
        if let Some(mut group) = self.groups.get(&group_name) {
            group.members.clear();
            group.release.remove();
            let hash = env::sha256(group_name.as_bytes());
            let _ = env::storage_remove(&to_storage_key_raw(GROUP_PREFIX, &hash));
            let _ = env::storage_remove(&to_storage_key_raw(GROUP_RELEASE_MODEL_SUFFIX, &hash));
            self.groups.remove(&group_name);
        }
    }

    pub fn add_group_member(&mut self, group_name: GroupName, account_id: AccountId) -> bool {
        match self.groups.get(&group_name) {
            Some(mut group) => {
                group.members.insert(&account_id);
                self.groups.insert(&group_name, &group);
                true
            }
            _ => false,
        }
    }

    pub fn remove_group_member(&mut self, group_name: GroupName, account_id: AccountId) {
        if let Some(mut group) = self.groups.get(&group_name) {
            group.members.remove(&account_id);
            self.groups.insert(&group_name, &group);
        }
    }

    //TODO implement on receiving contract based on wanted functionality
    // sender_id - who sent the tokens
    // env::predeccesor_account_id - token acc that confirms sender_id transfered this amount of FT to this account
    // receiver - this acc should register it
    pub fn ft_on_transfer(&self, _sender_id: String, _amount: U128, _msg: String) -> String {
        unimplemented!();
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn clean_self(&mut self) {
        assert!(self.council.contains(&env::predecessor_account_id()));
        env::storage_remove(&StorageKeys::NewVersionCode.into_storage_key());
    }

    /// For dev/testing purposes only
    #[cfg(feature = "testnet")]
    pub fn delete_self(self) -> Promise {
        assert!(self.council.contains(&env::predecessor_account_id()));
        Promise::new(env::current_account_id()).delete_account(self.factory_acc)
    }
}
*/

/******************************************************************************
 *
 * Fungible Token (NEP-141)
 * https://nomicon.io/Standards/FungibleToken/Core.html
 *
 ******************************************************************************/

#[near_bindgen]
impl FungibleTokenCore for NewDaoContract {
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
impl FungibleTokenResolver for NewDaoContract {
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
impl FungibleTokenMetadataProvider for NewDaoContract {
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
impl StorageManagement for NewDaoContract {
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

//TODO: MOVE to action.rs

/// Triggers new version download from factory
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    use crate::constants::{GAS_MIN_DOWNLOAD_LIMIT, VERSION};
    use env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    // We are not able to access council members any other way so we have deserialize SC
    let contract: NewDaoContract = env::state_read().unwrap();

    // Currently only council member can call this
    //assert!(contract.council.contains(&env::predecessor_account_id())); //TODO FIX

    let factory_acc = env::storage_read(&StorageKeys::FactoryAcc.into_storage_key()).unwrap();
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
                GAS_MIN_DOWNLOAD_LIMIT,
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
    assert_eq!(
        env::predecessor_account_id(),
        String::from_utf8(env::storage_read(&StorageKeys::FactoryAcc.into_storage_key()).unwrap())
            .unwrap()
            .to_string()
    );
    env::storage_write(
        &StorageKeys::NewVersionCode.into_storage_key(),
        &env::input().unwrap(),
    );
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn upgrade_self() {
    use crate::constants::GAS_MIN_UPGRADE_LIMIT;
    use near_sdk::env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));

    // We are not able to access council members any other way so we have deserialize SC
    let contract: NewDaoContract = env::state_read().unwrap();

    // Currently only council member can call this
    //assert!(contract.council.contains(&env::predecessor_account_id())); //TODO FIX

    let current_acc = env::current_account_id().into_bytes();
    let method_name = "migrate".as_bytes().to_vec();
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
                    GAS_MIN_UPGRADE_LIMIT,
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
