use crate::storage::DataType;
use near_sdk::borsh::{self, BorshDeserialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::Serialize;
use near_sdk::{env, IntoStorageKey};
use near_sdk::{json_types::U128, near_bindgen, AccountId};

use crate::constants::{
    DEPOSIT_ADD_PROPOSAL, DEPOSIT_VOTE, GAS_ADD_PROPOSAL, GAS_FINISH_PROPOSAL, PROPOSAL_KIND_COUNT,
    VERSION,
};
use crate::group::{Group, GroupMember, GroupOutput};
use crate::proposal::VProposal;
use crate::release::{ReleaseDb, ReleaseModel};
use crate::settings::{DaoSettings, VoteSettings};
use crate::storage::StorageData;
use crate::tags::Tags;
use crate::{core::*, GroupId, GroupName, StorageKey};
use crate::{TagCategory, CID};

/*
#[near_bindgen]
impl DaoContract {
    pub fn statistics_ft(&self) -> StatsFT {
        StatsFT {
            total_supply: self.ft_total_supply,
            decimals: self.ft_metadata.get().unwrap().decimals,
            total_distributed: self.ft_total_distributed,
            council_ft_stats: self.release_db.get(&TokenGroup::Council).unwrap().into(),
            council_release_model: self
                .release_config
                .get(&TokenGroup::Council)
                .unwrap()
                .into(),
            public_ft_stats: self.release_db.get(&TokenGroup::Public).unwrap().into(),
            public_release_model: self.release_config.get(&TokenGroup::Public).unwrap().into(),
            storage_locked_near: U128::from(
                env::storage_byte_cost() * env::storage_usage() as u128,
            ),
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
            council_share_percent: config.council_share,
            registered_user_count: self.ft.registered_accounts_count,
            council_rights: self.group_rights.get(&TokenGroup::Council),
        }
    }

    pub fn registered_user_count(&self) -> u32 {
        self.ft.registered_accounts_count
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
        let config = match self.config.get().unwrap() {
            VConfig::Curr(c) => c,
            _ => unreachable!(),
        };

        DaoConfig {
            version: VERSION,
            lang: config.lang,
            slogan: config.slogan,
            description: config.description,
            council_share: config.council_share,
            vote_spam_threshold: config.vote_spam_threshold,
        }
    }

    /*     pub fn group_members(self, group: TokenGroup) -> Vec<AccountId> {
        match group {
            TokenGroup::Council => {
                self.council.to_vec()
            },
            TokenGroup::Public => {
                unimplemented!()
            }
        }
    } */

    pub fn doc_files(self) -> DocFileMetadata {
        DocFileMetadata {
            files: self.doc_metadata.to_vec(),
            map: self.mappers.get(&MapperKind::Doc).unwrap(),
        }
    }

    /// Returns sha256 hash of downloaded new version for update
    pub fn version_hash(self) -> Option<Base64VecU8> {
        let code = env::storage_read(&StorageKeys::NewVersionCode.into_storage_key())?;
        Some(Base64VecU8::from(env::sha256(&code)))
    }

    pub fn vote_policies(self) -> Vec<(ProposalKindIdent, VoteConfig)> {
        let mut vec: Vec<(ProposalKindIdent, VoteConfig)> =
            Vec::with_capacity(PROPOSAL_KIND_COUNT.into());

        vec.push((
            ProposalKindIdent::Pay,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::Pay)
                    .unwrap(),
            ),
        ));
        vec.push((
            ProposalKindIdent::AddMember,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::AddMember)
                    .unwrap(),
            ),
        ));
        vec.push((
            ProposalKindIdent::RemoveMember,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::RemoveMember)
                    .unwrap(),
            ),
        ));
        vec.push((
            ProposalKindIdent::GeneralProposal,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::GeneralProposal)
                    .unwrap(),
            ),
        ));
        vec.push((
            ProposalKindIdent::AddDocFile,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::AddDocFile)
                    .unwrap(),
            ),
        ));
        vec.push((
            ProposalKindIdent::InvalidateFile,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::InvalidateFile)
                    .unwrap(),
            ),
        ));
        vec.push((
            ProposalKindIdent::DistributeFT,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::DistributeFT)
                    .unwrap(),
            ),
        ));
        vec.push((
            ProposalKindIdent::RightForActionCall,
            VoteConfig::from(
                self.vote_policy_config
                    .get(&ProposalKindIdent::RightForActionCall)
                    .unwrap(),
            ),
        ));

        vec
    }

    pub fn ref_pools(self) -> Vec<u32> {
        self.ref_pools.get().unwrap()
    }

    pub fn skyward_auctions(self) -> Vec<u64> {
        self.skyward_auctions.get().unwrap()
    }
}
*/

// ------------ NEW
#[near_bindgen]
impl NewDaoContract {
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

    pub fn dao_settings(self) -> DaoSettings {
        self.settings.get().unwrap().into()
    }

    pub fn vote_settings(self) -> Vec<VoteSettings> {
        self.vote_settings
            .get()
            .unwrap()
            .into_iter()
            .map(|s| s.into())
            .collect()
    }

    pub fn groups(self) -> Vec<GroupOutput> {
        self.groups
            .to_vec()
            .into_iter()
            .map(|(id, group)| GroupOutput::from_group(id, group))
            .collect()
    }

    pub fn group_names(self) -> Vec<GroupName> {
        self.groups
            .values_as_vector()
            .to_vec()
            .into_iter()
            .map(|g| g.settings.name)
            .collect()
    }

    pub fn group_members(self, id: GroupId) -> Option<Vec<GroupMember>> {
        self.groups
            .get(&id)
            .map(|group| group.members.get_members())
    }

    pub fn tags(self, category: TagCategory) -> Option<Tags> {
        self.tags.get(&category)
    }

    pub fn storage_bucket_all(self, bucket_id: String) -> Option<Vec<StorageData>> {
        self.storage.get(&bucket_id).map(|bucket| {
            bucket
                .get_all_data()
                .into_iter()
                .map(|(_, data)| data)
                .collect()
        })
    }

    pub fn storage_data(self, bucket_id: StorageKey, data_id: String) -> Option<StorageData> {
        self.storage
            .get(&bucket_id)
            .map(|bucket| bucket.get_data(&data_id))
            .flatten()
    }

    pub fn storage_data_as_vec_u8(
        self,
        bucket_id: StorageKey,
        data_id: String,
    ) -> Option<Result<Vec<u8>, String>> {
        let storage_data = self
            .storage
            .get(&bucket_id)
            .map(|bucket| bucket.get_data(&data_id))
            .flatten();

        storage_data.map(|s| s.try_into_vec_u8())
    }

    pub fn storage_data_as_string(
        self,
        bucket_id: StorageKey,
        data_id: String,
    ) -> Option<Result<String, String>> {
        let storage_data = self
            .storage
            .get(&bucket_id)
            .map(|bucket| bucket.get_data(&data_id))
            .flatten();

        storage_data.map(|s| s.try_into_string())
    }

    pub fn storage_data_as_vec_string(
        self,
        bucket_id: StorageKey,
        data_id: String,
    ) -> Option<Result<Vec<String>, String>> {
        let storage_data = self
            .storage
            .get(&bucket_id)
            .map(|bucket| bucket.get_data(&data_id))
            .flatten();

        storage_data.map(|s| s.try_into_vec_string())
    }
}
