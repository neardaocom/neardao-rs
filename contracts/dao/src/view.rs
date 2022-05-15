use library::types::datatype::Value;
use library::workflow::instance::Instance;
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::MethodName;
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, AccountId, Balance};

use crate::group::{GroupMember, GroupOutput};
use crate::internal::utils::current_timestamp_sec;
use crate::proposal::VersionedProposal;
use crate::reward::Reward;
use crate::role::UserRoles;
use crate::settings::Settings;
use crate::tags::Tags;
use crate::treasury::TreasuryPartition;
use crate::wallet::Wallet;
use crate::{core::*, GroupId, GroupName, StorageKey};
use crate::{TagCategory, TimestampSec};

#[near_bindgen]
impl Contract {
    /// Returns total delegated stake.
    pub fn delegation_total_supply(&self) -> U128 {
        U128(self.total_delegation_amount)
    }

    /// Returns total user vote weight.
    pub fn user_vote_weight(&self, account_id: AccountId) -> U128 {
        self.get_user_weight(&account_id).into()
    }

    /// Searches for next tick looks ahead up to `search_max_ticks` ticks.
    pub fn next_tick(&self, search_max_ticks: usize) -> Option<TimestampSec> {
        let mut next_tick = None;
        let mut tick = self.last_tick;
        for _ in 0..search_max_ticks {
            tick += self.tick_interval;
            if self.events.get(&tick).is_some() {
                next_tick = Some(tick);
                break;
            }
        }
        next_tick
    }

    /// Return general statitstics about DAO.
    pub fn statistics(self) -> Statistics {
        Statistics {
            staking_id: self.staking_id,
            token_id: self.token_id,
            total_delegation_amount: self.total_delegation_amount,
            total_delegators_count: self.total_delegators_count,
            ft_total_supply: self.ft_total_supply,
            decimals: self.decimals,
            total_members_count: self.total_members_count,
            total_account_balance: env::account_balance(),
            free_account_balance: env::account_balance()
                - env::storage_usage() as u128 * env::storage_byte_cost(),
        }
    }

    pub fn proposal(&self, id: u32) -> Option<(VersionedProposal, Option<Vec<TemplateSettings>>)> {
        self.proposals
            .get(&id)
            .map(|p| (p, self.proposed_workflow_settings.get(&id)))
    }

    pub fn proposals(&self, from_id: u64, limit: u64) -> Vec<(u32, VersionedProposal)> {
        let keys = self.proposals.keys_as_vector();
        let values = self.proposals.values_as_vector();
        (from_id..std::cmp::min(from_id + limit, self.proposals.len()))
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap()))
            .collect()
    }

    pub fn dao_settings(self) -> Settings {
        self.settings.get().unwrap().into()
    }

    pub fn standard_fncall_names(self) -> Vec<MethodName> {
        self.standard_function_call_metadata.keys().collect()
    }

    pub fn wf_template(self, id: u16) -> Option<(Template, Vec<TemplateSettings>)> {
        self.workflow_template.get(&id)
    }

    pub fn wf_templates(self) -> Vec<(u16, (Template, Vec<TemplateSettings>))> {
        self.workflow_template.to_vec()
    }

    pub fn wf_instance(self, proposal_id: u32) -> Option<Instance> {
        self.workflow_instance.get(&proposal_id)
    }

    pub fn wf_propose_settings(self, proposal_id: u32) -> Option<ProposeSettings> {
        self.workflow_propose_settings.get(&proposal_id)
    }

    pub fn wf_instances(self) -> Vec<Option<Instance>> {
        (1..=self.proposal_last_id)
            .into_iter()
            .map(|i| self.workflow_instance.get(&i))
            .collect()
    }

    // For debugging purposes only.
    pub fn debug_log(self) -> Vec<String> {
        self.debug_log
    }
    // For integration tests.
    pub fn current_timestamp(self) -> u64 {
        current_timestamp_sec()
    }

    pub fn view_reward(self, reward_id: u16) -> Reward {
        self.rewards.get(&reward_id).unwrap().into()
    }
    pub fn reward_list(self, from_id: u16, limit: u16) -> Vec<(u16, Reward)> {
        let mut rewards = Vec::with_capacity(self.reward_last_id as usize);
        for i in from_id..std::cmp::min(self.reward_last_id, limit) {
            if let Some(reward) = self.rewards.get(&i) {
                rewards.push((i, reward.into()));
            }
        }
        rewards
    }
    pub fn partition_list(self, from_id: u16, limit: u16) -> Vec<(u16, TreasuryPartition)> {
        let mut partitions = Vec::with_capacity(self.reward_last_id as usize);
        for i in from_id..std::cmp::min(self.partition_last_id, limit) {
            if let Some(partition) = self.treasury_partition.get(&i) {
                partitions.push((i, partition.into()));
            }
        }
        partitions
    }
    pub fn partition(&self, id: u16) -> Option<TreasuryPartition> {
        self.treasury_partition.get(&id).map(|p| p.into())
    }
    pub fn view_wallet(self, account_id: AccountId) -> Wallet {
        self.wallets.get(&account_id).unwrap().into()
    }
    pub fn view_user_roles(self, account_id: AccountId) -> UserRoles {
        self.user_roles.get(&account_id).unwrap()
    }

    #[allow(unused_variables)]
    pub fn check_transition(
        self,
        proposal_id: u32,
        args: Vec<Value>,
        activity_id: u8,
        transition_id: Option<u8>,
    ) -> bool {
        unimplemented!()
    }

    pub fn groups(self) -> Vec<GroupOutput> {
        self.groups
            .to_vec()
            .into_iter()
            .map(|(id, group)| GroupOutput::from_group(id, group))
            .collect()
    }

    pub fn group(self, id: u16) -> Option<GroupOutput> {
        self.groups
            .get(&id)
            .map(|group| GroupOutput::from_group(id, group))
    }

    pub fn group_names(&self) -> Vec<GroupName> {
        self.groups
            .values_as_vector()
            .to_vec()
            .into_iter()
            .map(|g| g.settings.name)
            .collect()
    }

    pub fn group_members(&self, id: GroupId) -> Option<Vec<GroupMember>> {
        self.groups
            .get(&id)
            .map(|group| group.members.get_members())
    }

    pub fn tags(self, category: TagCategory) -> Option<Tags> {
        self.tags.get(&category)
    }

    pub fn storage_bucket_data_all(self, bucket_id: StorageKey) -> Option<Vec<(String, Value)>> {
        self.storage
            .get(&bucket_id)
            .map(|bucket| bucket.get_all_data())
    }

    pub fn storage_buckets(self) -> Vec<StorageKey> {
        self.storage.keys_as_vector().to_vec()
    }

    pub fn storage_bucket_data(self, bucket_id: StorageKey, data_id: String) -> Option<Value> {
        self.storage
            .get(&bucket_id)
            .map(|bucket| bucket.get_data(&data_id))
            .flatten()
    }

    pub fn wf_log(self, proposal_id: u32) -> Option<Vec<ActivityLog>> {
        self.workflow_activity_log.get(&proposal_id)
    }
}

#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Statistics {
    pub staking_id: AccountId,
    pub token_id: AccountId,
    pub total_delegation_amount: Balance,
    pub total_delegators_count: u32,
    pub ft_total_supply: u32,
    pub decimals: u8,
    pub total_members_count: u32,
    pub total_account_balance: u128,
    pub free_account_balance: u128,
}
