use near_sdk::{IntoStorageKey, env};
use near_sdk::json_types::{Base64VecU8};
use near_sdk::serde::Serialize;
use near_sdk::{json_types::U128, near_bindgen, AccountId};

use crate::CID;
use crate::action::MemberGroup;
use crate::config::VConfig;
use crate::file::{VFileMetadata};
use crate::proposal::{ProposalKindIdent, VProposal};
use crate::vote_policy::{self, VoteConfig};
use crate::{core::*, proposal::Proposal};

#[near_bindgen]
impl DaoContract {
    pub fn statistics_ft(&self) -> StatsFT {
        StatsFT {
            total_supply: self.total_supply,
            init_distribution: self.init_distribution,
            decimals: self.ft_metadata.get().unwrap().decimals,
            total_released: U128::from(self.already_released_ft),
            free: U128::from(self.free_ft),
            council_ft_shared: self.release_db[INDEX_RELEASED_COUNCIL as usize],
            community_ft_shared: self.release_db[INDEX_RELEASED_COMMUNITY as usize],
            foundation_ft_shared: self.release_db[INDEX_RELEASED_FOUNDATION as usize],
            parent_shared: self.release_db[INDEX_RELEASED_PARENT as usize],
            owner_shared: self.release_db[INDEX_RELEASED_FACTORY_ACC as usize],
        }
    }

    pub fn statistics_members(self) -> StatsMembers {

        let config = match self.config.get().unwrap() {
            VConfig::Curr(c) => c,
            _ => unreachable!(),
        };

        StatsMembers {
            factory_acc: self.factory_acc,
            council: self.council.to_vec(),
            foundation: self.foundation.to_vec(),
            community: self.community.to_vec(),
            council_share_percent: config.council_share,
            foundation_share_percent:config.foundation_share.unwrap_or_default(),
            community_share_percent: config.community_share.unwrap_or_default(),
            council_ft_shared: self.release_db[INDEX_RELEASED_COUNCIL as usize],
            community_ft_shared: self.release_db[INDEX_RELEASED_COMMUNITY as usize],
            foundation_ft_shared: self.release_db[INDEX_RELEASED_FOUNDATION as usize],
            registered_user_count: self.registered_accounts_count,
        }
    }

    pub fn registered_user_count(&self) -> u32 {
        self.registered_accounts_count
    }

    pub fn proposal(&self, proposal_id: u32) -> Option<VProposal> {
        self.proposals.get(&proposal_id)
    }

    pub fn proposals(&self, from_index: u64, limit: u64) -> Vec<(u32, VProposal)> {
        let keys = self.proposals.keys_as_vector();
        let values = self.proposals.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, self.proposals.len()))
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap()))
            .collect()
    }

    pub fn dao_fees(self) -> DaoFee {
        DaoFee {
            min_gas_add_proposal: U128::from(GAS_ADD_PROPOSAL as u128),
            min_gas_vote: U128::from(GAS_VOTE as u128),
            min_gas_finish_proposal: U128::from(GAS_FINISH_PROPOSAL as u128),
            min_yocto_near_add_proposal: U128::from(DEPOSIT_ADD_PROPOSAL),
            min_yocto_near_vote: U128::from(DEPOSIT_VOTE),
        }
    }

    pub fn dao_config(self) -> DaoConfig {

        let config = match self.config.get().unwrap(){
            VConfig::Curr(c) => c,
            _ => unreachable!(),
        };

        DaoConfig {
            version: VERSION,
            lang: config.lang,
            slogan: config.slogan,
            description: config.description,
            council_share: config.council_share,
            foundation_share: config.foundation_share,
            community_share: config.community_share,
            vote_spam_threshold: config.vote_spam_threshold,
        }
    }

    pub fn payments(self) -> Vec<RegularPayment> {
        self.regular_payments.to_vec()
    }

    pub fn group_members(self, group: MemberGroup) -> Vec<AccountId> {
        match group {
            MemberGroup::Council => self.council.to_vec(),
            MemberGroup::Foundation => self.foundation.to_vec(),
            MemberGroup::Community => self.community.to_vec(),
            _ => unimplemented!(),
        }
    }

    pub fn doc_files(self) -> DocFileMetadata {
        DocFileMetadata {
            files: self.doc_metadata.to_vec(),
            map: self.mappers.get(&MapperKind::Doc).unwrap()
        }
    }

    /// Returns sha256 hash of downloaded new version for update
    pub fn version_hash(self) -> Option<Base64VecU8> {
        let code = env::storage_read(&StorageKeys::NewVersionCode.into_storage_key())?;
        Some(Base64VecU8::from(env::sha256(&code)))
    }

    pub fn vote_policies(self) -> Vec<(ProposalKindIdent,VoteConfig)> {
        let mut vec: Vec<(ProposalKindIdent, VoteConfig)> = Vec::with_capacity(PROPOSAL_KIND_COUNT.into());

        vec.push((ProposalKindIdent::Pay, VoteConfig::from(self.vote_policy_config.get(&ProposalKindIdent::Pay).unwrap())));
        vec.push((ProposalKindIdent::AddMember, VoteConfig::from(self.vote_policy_config.get(&ProposalKindIdent::AddMember).unwrap())));
        vec.push((ProposalKindIdent::RemoveMember, VoteConfig::from(self.vote_policy_config.get(&ProposalKindIdent::RemoveMember).unwrap())));
        vec.push((ProposalKindIdent::RegularPayment, VoteConfig::from(self.vote_policy_config.get(&ProposalKindIdent::RegularPayment).unwrap())));
        vec.push((ProposalKindIdent::GeneralProposal, VoteConfig::from(self.vote_policy_config.get(&ProposalKindIdent::GeneralProposal).unwrap())));
        vec.push((ProposalKindIdent::AddDocFile, VoteConfig::from(self.vote_policy_config.get(&ProposalKindIdent::AddDocFile).unwrap())));
        vec.push((ProposalKindIdent::InvalidateFile, VoteConfig::from(self.vote_policy_config.get(&ProposalKindIdent::InvalidateFile).unwrap())));

        vec
    }
}
#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct StatsFT {
    pub total_supply: u32,
    pub init_distribution: u32,
    pub decimals: u8,
    pub total_released: U128,
    pub free: U128,
    pub council_ft_shared: u32,
    pub community_ft_shared: u32,
    pub foundation_ft_shared: u32,
    pub parent_shared: u32,
    pub owner_shared: u32,
}

#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct StatsMembers {
    factory_acc: AccountId,
    council: Vec<AccountId>,
    foundation: Vec<AccountId>,
    community: Vec<AccountId>,
    council_share_percent: u8,
    foundation_share_percent: u8,
    community_share_percent: u8,
    council_ft_shared: u32,
    community_ft_shared: u32,
    foundation_ft_shared: u32,
    registered_user_count: u32,
}

#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoFee {
    min_gas_add_proposal: U128,
    min_gas_vote: U128,
    min_gas_finish_proposal: U128,
    min_yocto_near_add_proposal: U128,
    min_yocto_near_vote: U128,
}

#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct DaoConfig {
    pub version: u8,
    pub lang: String,
    pub slogan: String,
    pub description: String,
    pub council_share: u8,
    pub foundation_share: Option<u8>,
    pub community_share: Option<u8>,
    pub vote_spam_threshold: u8,
}
#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct DocFileMetadata {
    files: Vec<(CID, VFileMetadata)>,
    map: Mapper,
}