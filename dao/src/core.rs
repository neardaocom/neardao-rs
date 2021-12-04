use std::convert::TryFrom;
use std::u128;

use crate::standard_impl::ft::FungibleToken;
use crate::standard_impl::ft_metadata::FungibleTokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env::{self},
    log, near_bindgen, AccountId, Balance, BorshStorageKey, IntoStorageKey, PanicOnDefault,
    Promise, PromiseOrValue,
};

use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;

use crate::config::*;
use crate::file::{FileType, VFileMetadata};
use crate::proposal::*;
use crate::release::{ReleaseDb, ReleaseModel, ReleaseModelInput, VReleaseDb, VReleaseModel};
use crate::vote_policy::{VVoteConfig, VoteConfig, VoteConfigInput};
use crate::{action::*, CID_MAX_LENGTH};

near_sdk::setup_alloc!();

//TODO: With each upgrade +1 !!! TODO safe-auto inc mechanism
pub const VERSION: u8 = 1;

pub const GAS_MIN_DOWNLOAD_LIMIT: u64 = 200_000_000_000_000;
pub const GAS_MIN_UPGRADE_LIMIT: u64 = 100_000_000_000_000;

pub const GAS_ADD_PROPOSAL: u64 = 100_000_000_000_000;
pub const GAS_FINISH_PROPOSAL: u64 = 100_000_000_000_000;
pub const GAS_VOTE: u64 = 10_000_000_000_000;
pub const DEPOSIT_ADD_PROPOSAL: u128 = 500_000_000_000_000_000_000_000; // 0.5 N
pub const DEPOSIT_VOTE: u128 = 1_250_000_000_000_000_000_000; // 0.00125 N

pub const INDEX_RELEASED_COUNCIL: u8 = 0;
pub const INDEX_RELEASED_COMMUNITY: u8 = 1;
pub const INDEX_RELEASED_FOUNDATION: u8 = 2;
pub const INDEX_RELEASED_PARENT: u8 = 3;
pub const INDEX_RELEASED_FACTORY_ACC: u8 = 4;

pub const METADATA_MAX_DECIMALS: u8 = 28;

pub const MAX_FT_TOTAL_SUPPLY: u32 = 1_000_000_000;

// Must match count of proposal variants
pub const PROPOSAL_KIND_COUNT: u8 = 8;

pub const DEFAULT_DOC_CAT: &str = "basic";

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    FT,
    FTMetadata,
    Proposals,
    ProposalConfig,
    Council,
    VConfig,
    Foundation,
    Community,
    ReleaseConfig,
    ReleaseDb,
    RegularPayment,
    DocMetadata,
    Mappers,
    NewVersionCode,
    FactoryAcc,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DaoContract {
    pub factory_acc: String,
    pub config: LazyOption<VConfig>,
    pub council: UnorderedSet<AccountId>,
    pub foundation: UnorderedSet<AccountId>,
    pub community: UnorderedSet<AccountId>,
    pub ft_metadata: LazyOption<FungibleTokenMetadata>,
    pub ft: FungibleToken,
    pub ft_total_supply: u32,
    pub ft_total_distributed: u32,
    pub decimal_const: u128,
    pub proposals: UnorderedMap<u32, VProposal>,
    pub proposal_count: u32,
    pub release_config: LookupMap<TokenGroup, VReleaseModel>, //TODO merge with DB ?
    pub release_db: LookupMap<TokenGroup, VReleaseDb>,
    pub vote_policy_config: LookupMap<ProposalKindIdent, VVoteConfig>,
    pub regular_payments: UnorderedSet<RegularPayment>,
    pub doc_metadata: UnorderedMap<String, VFileMetadata>,
    pub mappers: UnorderedMap<MapperKind, Mapper>,
}

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
        let mut dao: DaoContract = env::state_read().expect("Failed to migrate");

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

        dao
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
            foundation: UnorderedSet::new(StorageKeys::Foundation),
            community: UnorderedSet::new(StorageKeys::Community),
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
            regular_payments: UnorderedSet::new(StorageKeys::RegularPayment),
            doc_metadata: UnorderedMap::new(StorageKeys::DocMetadata),
            mappers: UnorderedMap::new(StorageKeys::Mappers),
        };

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
        assert!(env::attached_deposit() >= DEPOSIT_ADD_PROPOSAL);
        assert!(env::prepaid_gas() >= GAS_ADD_PROPOSAL);
        //assert!(self.ft.accounts.contains_key(&env::predecessor_account_id())); TODO:
        if !self
            .ft
            .accounts
            .contains_key(&env::predecessor_account_id())
        {
            self.storage_deposit(
                Some(ValidAccountId::try_from(env::predecessor_account_id()).unwrap()),
                None,
            );
        }

        proposal_input.assert_valid();

        let vote_policy = VoteConfig::from(
            self.vote_policy_config
                .get(&ProposalKindIdent::get_ident_from(&tx_input))
                .expect("Invalid proposal input"),
        );

        let tx = self
            .create_tx(
                tx_input,
                env::predecessor_account_id(),
                env::block_timestamp(),
            )
            .unwrap();

        self.proposal_count += 1;

        let proposal = Proposal::new(
            self.proposal_count,
            env::predecessor_account_id(),
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
        assert!(env::prepaid_gas() >= GAS_VOTE);
        assert!(env::attached_deposit() >= DEPOSIT_VOTE);
        //assert!(self.ft.accounts.contains_key(&env::predecessor_account_id())); TODO:
        if !self
            .ft
            .accounts
            .contains_key(&env::predecessor_account_id())
        {
            self.storage_deposit(
                Some(ValidAccountId::try_from(env::predecessor_account_id()).unwrap()),
                None,
            );
        }

        assert!(env::predecessor_account_id() != self.factory_acc);

        let mut proposal =
            Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));

        if proposal.status != ProposalStatus::InProgress
            || proposal.duration_to <= env::block_timestamp()
        {
            return VoteResult::VoteEnded;
        }

        if vote_kind > 2 {
            return VoteResult::InvalidVote;
        }

        if proposal.vote_only_once && proposal.votes.contains_key(&env::predecessor_account_id()) {
            return VoteResult::AlreadyVoted;
        }

        proposal
            .votes
            .insert(env::predecessor_account_id(), vote_kind);

        self.proposals
            .insert(&proposal_id, &VProposal::Curr(proposal));
        VoteResult::Ok
    }

    pub fn finish_proposal(&mut self, proposal_id: u32) -> ProposalStatus {
        assert!(env::prepaid_gas() >= GAS_FINISH_PROPOSAL);
        let mut proposal =
            Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));

        let new_status = match &proposal.status {
            &ProposalStatus::InProgress => {
                if env::block_timestamp() < proposal.duration_to {
                    None
                } else {
                    // count votes
                    let mut votes = vec![0 as u128; 3];
                    for (voter, vote_value) in proposal.votes.iter() {
                        votes[*vote_value as usize] += self.ft.accounts.get(voter).unwrap();
                    }

                    let total_voted_amount: u128 = votes.iter().sum();

                    // we need to read config just because of spam TH value - could be moved to voting ??
                    let config = Config::from(self.config.get().unwrap());

                    // check spam
                    if self::calc_percent_u128(votes[0], total_voted_amount, self.decimal_const)
                        >= config.vote_spam_threshold
                    {
                        Some(ProposalStatus::Spam)
                    } else if self::calc_percent_u128(
                        total_voted_amount,
                        self.ft_total_distributed as u128 * self.decimal_const,
                        self.decimal_const,
                    ) < proposal.quorum
                    {
                        // not enough quorum
                        Some(ProposalStatus::Invalid)
                    } else if self::calc_percent_u128(
                        votes[1],
                        total_voted_amount,
                        self.decimal_const,
                    ) < proposal.approve_threshold
                    {
                        // not enough voters to accept
                        Some(ProposalStatus::Rejected)
                    } else {
                        // proposal is accepted, try to execute transaction
                        if let Err(errors) = self.execute_tx(
                            &proposal.transactions,
                            env::attached_deposit(),
                            env::account_balance(),
                            env::block_timestamp(),
                        ) {
                            log!("errors: {:?}", errors);
                            Some(ProposalStatus::Invalid)
                        } else {
                            Some(ProposalStatus::Accepted)
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

    //TODO implement on receiving contract based on wanted functionality
    // sender_id - who sent the tokens
    // env::predeccesor_account_id - token acc that confirms sender_id transfered this amount of FT to this account
    // receiver - this acc should register it
    pub fn ft_on_transfer(&self, sender_id: String, amount: U128, msg: String) -> String {
        "Not implemented yet".into()
    }

    /// For dev/testing purposes only
    #[private]
    pub fn clean_self(&mut self) {
        env::storage_remove(&StorageKeys::NewVersionCode.into_storage_key());
    }

    /// For dev/testing purposes only
    #[private]
    pub fn delete_self(self) -> Promise {
        Promise::new(env::current_account_id()).delete_account(self.factory_acc)
    }
}

#[inline]
pub fn assert_valid_init_config(config: &ConfigInput) {
    assert!(
        config.council_share.unwrap()
            + config.community_share.unwrap_or_default()
            + config.foundation_share.unwrap_or_default()
            <= 100
    );
    assert!(config.vote_spam_threshold.unwrap_or_default() <= 100);
    assert!(config.description.as_ref().unwrap().len() > 0);
}

#[inline]
pub fn assert_valid_founders(founders: &mut Vec<AccountId>) {
    let founders_len_before_dedup = founders.len();
    founders.sort();
    founders.dedup();
    assert_eq!(founders_len_before_dedup, founders.len());
}

impl DaoContract {
    pub fn setup_voting_policy(&mut self, configs: Vec<VoteConfigInput>) {
        for p in configs.into_iter() {
            assert!(
                self.vote_policy_config
                    .insert(
                        &p.proposal_kind.clone(),
                        &VVoteConfig::Curr(VoteConfig::try_from(p).unwrap())
                    )
                    .is_none(),
                "{}",
                "Duplicate voting policy"
            );
        }
    }

    pub fn setup_release_models(
        &mut self,
        release_config: Vec<(TokenGroup, ReleaseModelInput)>,
        founders_distribution: u32,
    ) {
        let config: Config = self.config.get().unwrap().into();

        for (group, model) in release_config.into_iter() {
            let release_model =
                ReleaseModel::from_input(model, (env::block_timestamp() / 10u64.pow(9)) as u32);

            let release_db;
            match group {
                TokenGroup::Council => {
                    release_db = if release_model == ReleaseModel::None {
                        let total = (config.council_share as u64 * self.ft_total_supply as u64
                            / 100) as u32;
                        ReleaseDb::new(total, total, founders_distribution)
                    } else {
                        ReleaseDb::new(
                            (config.council_share as u64 * self.ft_total_supply as u64 / 100)
                                as u32,
                            founders_distribution,
                            founders_distribution,
                        )
                    };
                }
                TokenGroup::Foundation => {
                    release_db = if release_model == ReleaseModel::None {
                        let total = (config.foundation_share.unwrap_or_default() as u64
                            * self.ft_total_supply as u64
                            / 100) as u32;
                        ReleaseDb::new(total, total, 0)
                    } else {
                        ReleaseDb::new(
                            (config.foundation_share.unwrap_or_default() as u64
                                * self.ft_total_supply as u64
                                / 100) as u32,
                            0,
                            0,
                        )
                    };
                }
                TokenGroup::Community => {
                    release_db = if release_model == ReleaseModel::None {
                        let total = (config.community_share.unwrap_or_default() as u64
                            * self.ft_total_supply as u64
                            / 100) as u32;
                        ReleaseDb::new(total, total, 0)
                    } else {
                        ReleaseDb::new(
                            (config.community_share.unwrap_or_default() as u64
                                * self.ft_total_supply as u64
                                / 100) as u32,
                            0,
                            0,
                        )
                    };
                }
                _ => env::panic(b"Cannot set Release model for Public"),
            }

            self.release_db
                .insert(&group, &VReleaseDb::Curr(release_db));
            self.release_config
                .insert(&group, &VReleaseModel::Curr(release_model));
        }

        // We set dao release
        let ft_amount = ((100
            - config.council_share as u64
            - config.foundation_share.unwrap_or_default() as u64
            - config.community_share.unwrap_or_default() as u64)
            * self.ft_total_supply as u64
            / 100) as u32;

        // dao itself has all tokens unlocked from the beginning
        self.release_db.insert(
            &TokenGroup::Public,
            &VReleaseDb::Curr(ReleaseDb::new(ft_amount, ft_amount, 0)),
        );
        self.release_config.insert(
            &TokenGroup::Public,
            &VReleaseModel::Curr(ReleaseModel::None),
        );
    }

    pub fn init_mappers(&mut self) {
        self.mappers.insert(
            &MapperKind::Doc,
            &Mapper::Doc {
                tags: [].into(),
                categories: [DEFAULT_DOC_CAT.into()].into(),
            },
        );
    }

    // Assumed user cannot unregister with non-zero amount of FT
    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        self.council.remove(&account_id);
        self.community.remove(&account_id);
        self.foundation.remove(&account_id);

        log!(
            "Closed @{} and all it's FT: {} were transfered back to the contract",
            account_id,
            balance
        );
    }

    //TODO: Tests
    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        self.ft.internal_deposit(&env::current_account_id(), amount);

        self.council.remove(&account_id);
        self.community.remove(&account_id);
        self.foundation.remove(&account_id);

        log!(
            "Account @{} deleted and all it's FT: {} were transfered back to the contract",
            account_id,
            amount
        );
    }

    /// Validates all actions and tries to execute transaction
    pub fn execute_tx(
        &mut self,
        tx: &ActionTx,
        attached_deposit: u128,
        current_balance: u128,
        current_block_timestamp: u64,
    ) -> Result<(), Vec<ActionExecutionError>> {
        let mut errors: Vec<ActionExecutionError> = Vec::new();

        // Checks if all actions might be successfully executed
        self.validate_tx_before_execution(
            tx,
            current_balance,
            attached_deposit,
            current_block_timestamp,
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(errors);
        }

        // All actions should be executed now without any error
        for action in tx.actions.iter() {
            self.execute_action(action);
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn validate_tx_before_execution(
        &self,
        tx: &ActionTx,
        current_balance: u128,
        attached_deposit: u128,
        current_block_timestamp: u64,
        errors: &mut Vec<ActionExecutionError>,
    ) {
        for action in tx.actions.iter() {
            match action {
                Action::SendNear {
                    account_id,
                    amount_near,
                } => {
                    if current_balance < *amount_near {
                        errors.push(ActionExecutionError::NotEnoughNears);
                    }
                }
                Action::AddMember { account_id, group } => {}
                Action::RemoveMember { account_id, group } => {}
                Action::RegularPayment {
                    account_id,
                    amount_near,
                    since,
                    until,
                    period,
                } => {
                    if *since <= current_block_timestamp || *until <= current_block_timestamp {
                        errors.push(ActionExecutionError::InvalidTimeInputs);
                    }
                }
                Action::GeneralProposal { title } => {}
                Action::AddFile {
                    cid,
                    ftype,
                    metadata,
                    new_category,
                    new_tags,
                } => match ftype {
                    FileType::Doc => {
                        if self.doc_metadata.get(cid).is_some() {
                            errors.push(ActionExecutionError::CIDExists);
                        }
                    }
                    _ => unimplemented!(),
                },
                Action::InvalidateFile { cid } => {}
                Action::DistributeFT {
                    amount,
                    from_group,
                    accounts,
                } => {
                    let db: ReleaseDb = self.release_db.get(&from_group).unwrap().into();

                    if db.unlocked - db.distributed < *amount {
                        errors.push(ActionExecutionError::NotEnoughFT);
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn execute_action(&mut self, action: &Action) {
        match action {
            Action::SendNear {
                account_id,
                amount_near,
            } => {
                Promise::new(account_id.into()).transfer(*amount_near);
            }
            Action::AddMember { account_id, group } => {
                if !self.ft.accounts.contains_key(account_id) {
                    self.ft.internal_register_account(account_id);
                }

                match group {
                    TokenGroup::Council => {
                        self.council.insert(account_id);
                    }
                    TokenGroup::Foundation => {
                        self.foundation.insert(account_id);
                    }
                    TokenGroup::Community => {
                        self.community.insert(account_id);
                    }
                    TokenGroup::Public => (),
                }
            }
            Action::RemoveMember { account_id, group } => match group {
                TokenGroup::Council => {
                    self.council.remove(account_id);
                }
                TokenGroup::Foundation => {
                    self.foundation.remove(account_id);
                }
                TokenGroup::Community => {
                    self.community.remove(account_id);
                }
                TokenGroup::Public => unreachable!(),
            },
            Action::RegularPayment {
                account_id,
                amount_near,
                since,
                until,
                period,
            } => {
                self.regular_payments.insert(&RegularPayment {
                    account_id: account_id.to_owned(),
                    amount_near: *amount_near,
                    next: *since,
                    end: *until,
                    period: period.to_nanos(),
                });
            }
            Action::GeneralProposal { title } => {}
            Action::AddFile {
                cid,
                ftype,
                metadata,
                new_category,
                new_tags,
            } => {
                match ftype {
                    FileType::Doc => {
                        match self.mappers.get(&MapperKind::Doc).unwrap() {
                            Mapper::Doc {
                                mut tags,
                                mut categories,
                            } => {
                                let mut new_metadata = match metadata {
                                    VFileMetadata::Curr(fm) => fm.clone(),
                                    _ => unreachable!(),
                                };
                                if new_category.is_some() {
                                    if let Some(idx) =
                                        categories.iter().enumerate().find_map(|(i, s)| {
                                            s.eq(new_category.as_ref().unwrap()).then(|| i)
                                        })
                                    {
                                        new_metadata.category = idx as u8;
                                    } else {
                                        categories.push(new_category.clone().unwrap());
                                        new_metadata.category = categories.len() as u8 - 1;
                                    }
                                }

                                if new_tags.len() > 0 {
                                    // Check any of the new tags exist
                                    for nt in new_tags {
                                        if tags
                                            .iter()
                                            .enumerate()
                                            .find_map(|(i, s)| s.eq(nt).then(|| i))
                                            .is_none()
                                        {
                                            tags.push(nt.clone());
                                            new_metadata.tags.push(tags.len() as u8 - 1);
                                        }
                                    }
                                }

                                self.doc_metadata
                                    .insert(cid, &VFileMetadata::Curr(new_metadata));
                                self.mappers
                                    .insert(&MapperKind::Doc, &Mapper::Doc { tags, categories });
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            Action::InvalidateFile { cid } => {
                let mut metadata = match self.doc_metadata.get(&cid.clone()).unwrap() {
                    VFileMetadata::Curr(fm) => fm,
                    _ => unreachable!(),
                };

                if metadata.valid == true {
                    metadata.valid = false;
                    self.doc_metadata
                        .insert(&cid.clone(), &VFileMetadata::Curr(metadata));
                }
            }
            Action::DistributeFT {
                amount,
                from_group,
                accounts,
            } => {
                let mut db: ReleaseDb = self.release_db.get(&from_group).unwrap().into();
                let amount_per_account = *amount / accounts.len() as u32;

                for acc in accounts.iter() {
                    if !self.ft.accounts.contains_key(acc) {
                        self.ft.internal_register_account(acc);
                    }

                    self.ft.internal_transfer(
                        &env::current_account_id(),
                        &acc,
                        amount_per_account as u128 * self.decimal_const,
                        None,
                    );
                }

                self.ft_total_distributed += amount_per_account * accounts.len() as u32;
                db.distributed += amount_per_account * accounts.len() as u32;
                self.release_db.insert(from_group, &VReleaseDb::Curr(db));
            }
            _ => unimplemented!(),
        }
    }

    #[allow(unused)]
    pub fn create_tx(
        &self,
        tx_input: TxInput,
        caller: AccountId,
        current_block_timestamp: u64,
    ) -> Result<ActionTx, Vec<&'static str>> {
        let mut actions = Vec::with_capacity(2);
        let mut errors = Vec::with_capacity(2);
        let config = Config::from(self.config.get().unwrap());

        match tx_input {
            TxInput::Pay {
                account_id,
                amount_near,
            } => {
                actions.push(Action::SendNear {
                    account_id,
                    amount_near: amount_near.0,
                });
            }
            TxInput::AddMember { account_id, group } => {
                match group {
                    TokenGroup::Council => {
                        if self.council.contains(&account_id) {
                            errors.push("User is already in group");
                        }
                    }
                    TokenGroup::Foundation => {
                        if config.foundation_share.is_none() {
                            errors.push("Group is not permitted");
                        } else if self.foundation.contains(&account_id) {
                            errors.push("User is already in group");
                        }
                    }
                    TokenGroup::Community => {
                        if config.community_share.is_none() {
                            errors.push("Group is not permitted");
                        } else if self.community.contains(&account_id) {
                            errors.push("User is already in group");
                        }
                    }
                    TokenGroup::Public => {
                        if self.ft.accounts.contains_key(&account_id) {
                            errors.push("User is already in group");
                        }
                    }
                }

                if errors.is_empty() {
                    actions.push(Action::AddMember {
                        account_id,
                        group: group,
                    });
                }
            }
            TxInput::RemoveMember { account_id, group } => {
                match group {
                    TokenGroup::Council => {
                        if !self.council.contains(&account_id) {
                            errors.push("User is not in group");
                        }
                    }
                    TokenGroup::Foundation => {
                        if config.foundation_share.is_none() {
                            errors.push("Group is not permitted");
                        } else if !self.foundation.contains(&account_id) {
                            errors.push("User is not in group");
                        }
                    }
                    TokenGroup::Community => {
                        if config.community_share.is_none() {
                            errors.push("Group is not permitted");
                        } else if !self.community.contains(&account_id) {
                            errors.push("User is not in group");
                        }
                    }
                    TokenGroup::Public => {
                        errors.push("Remove from public group is forbidden");
                    }
                }

                if errors.is_empty() {
                    actions.push(Action::RemoveMember {
                        account_id,
                        group: group,
                    });
                }
            }
            TxInput::RegularPayment {
                account_id,
                amount_near,
                since,
                until,
                period,
            } => {
                if until <= since
                    || until <= current_block_timestamp
                    || since <= current_block_timestamp
                {
                    errors.push("Invalid time range")
                } else {
                    actions.push(Action::RegularPayment {
                        account_id,
                        amount_near: amount_near.into(),
                        since,
                        until,
                        period,
                    });
                }
            }
            TxInput::GeneralProposal { title } => {
                //TODO limit title length ??
                actions.push(Action::GeneralProposal { title });
            }
            TxInput::AddDocFile {
                cid,
                metadata,
                new_category,
                new_tags,
            } => {
                //TODO check precise length, not range
                if cid.len() > CID_MAX_LENGTH.into() {
                    errors.push("Invalid CID length");
                } else if self.doc_metadata.get(&cid).is_some() {
                    errors.push("Metadata already exists");
                } else if new_category.is_some()
                    && new_category.as_ref().map(|s| s.len()).unwrap() == 0
                {
                    errors.push("Category cannot be empty string");
                } else {
                    //TODO tags check ??
                    actions.push(Action::AddFile {
                        cid,
                        metadata,
                        ftype: FileType::Doc,
                        new_category,
                        new_tags,
                    });
                }
            }
            TxInput::InvalidateFile { cid } => {
                if self.doc_metadata.get(&cid).is_none() {
                    errors.push("Metadata does not exist");
                } else {
                    actions.push(Action::InvalidateFile { cid });
                }
            }
            TxInput::DistributeFT {
                total_amount,
                from_group,
                accounts,
            } => {
                let db: ReleaseDb = self.release_db.get(&from_group).unwrap().into();

                if db.unlocked - db.distributed < total_amount {
                    errors.push("Not enough FT in the group's treasury");
                } else {
                    actions.push(Action::DistributeFT {
                        amount: total_amount,
                        from_group,
                        accounts,
                    });
                }
            }
            _ => unimplemented!(),
        }

        if errors.is_empty() {
            return Ok(ActionTx { actions });
        } else {
            return Err(errors);
        }
    }
}

/// Calculates votes as percents
/// No bound checks implemented
#[inline]
pub fn calc_percent_u128(value: u128, total: u128, decimal_const: u128) -> u8 {
    ((value / decimal_const) as f64 / (total / decimal_const) as f64 * 100.0).round() as u8
}

/******************************************************************************
 *
 * Fungible Token (NEP-141)
 * https://nomicon.io/Standards/FungibleToken/Core.html
 *
 ******************************************************************************/

#[near_bindgen]
impl FungibleTokenCore for DaoContract {
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
impl FungibleTokenResolver for DaoContract {
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
            self.on_tokens_burned(sender_id, burned_amount);
        }
        used_amount.into()
    }
}
/******************************************************************************
 *
 * Storage Management (NEP-145)
 * https://nomicon.io/Standards/StorageManagement.html
 *
 ******************************************************************************/

#[near_bindgen]
impl StorageManagement for DaoContract {
    //TODO: solve
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
            self.on_account_closed(account_id, balance);
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

/// Triggers download new version from factory
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn download_new_version() {
    use env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    //assert_eq!(env::predecessor_account_id(), env::current_account_id());

    // We are not able to access council members any other way so we have deserialize SC
    let contract: DaoContract = env::state_read().unwrap();

    // Currently only council member can call this
    assert!(contract.council.contains(&env::predecessor_account_id()));

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

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn upgrade_self() {
    use env::BLOCKCHAIN_INTERFACE;

    env::setup_panic_hook();
    env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    //assert_eq!(env::predecessor_account_id(), env::current_account_id());

    // We are not able to access council members any other way so we have deserialize SC
    let contract: DaoContract = env::state_read().unwrap();

    // Currently only council member can call this
    assert!(contract.council.contains(&env::predecessor_account_id()));

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
    assert!(!env::storage_write(
        &StorageKeys::NewVersionCode.into_storage_key(),
        &env::input().unwrap()
    ));
}

//TODO migration ??
#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct RegularPayment {
    pub account_id: AccountId,
    pub amount_near: u128,
    pub next: u64,
    pub end: u64,
    pub period: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum MapperKind {
    Doc,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum Mapper {
    Doc {
        tags: Vec<String>,
        categories: Vec<String>,
    },
}
