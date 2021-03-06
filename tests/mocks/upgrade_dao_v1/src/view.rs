use library::types::Value;
use library::workflow::instance::Instance;
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::MethodName;
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::{env, near_bindgen, AccountId};

use crate::constants::VERSION;
use crate::group::Group;
use crate::internal::utils::current_timestamp_sec;
use crate::media::Media;
use crate::proposal::VersionedProposal;
use crate::reward::Reward;
use crate::role::{Roles, UserRoles};
use crate::settings::Settings;
use crate::tags::Tags;
use crate::treasury::TreasuryPartition;
use crate::wallet::{ClaimableReward, ClaimableRewards, Wallet};
use crate::TagCategory;
use crate::{core::*, StorageKey};

#[near_bindgen]
impl Contract {
    /// Returns total delegated stake.
    pub fn delegation_total_supply(&self) -> U128 {
        U128(self.total_delegation_amount)
    }
    /// Returns total user vote weight.
    pub fn user_vote_weight(&self, account_id: AccountId) -> U128 {
        self.delegations.get(&account_id).unwrap_or_default().into()
    }

    /// Return general statitstics about DAO.
    pub fn statistics(self) -> Statistics {
        Statistics {
            version: VERSION,
            total_delegation_amount: self.total_delegation_amount.into(),
            total_delegators_count: self.total_delegators_count,
            ft_total_supply: self.ft_total_supply,
            decimals: self.decimals,
            total_members_count: self.total_members_count,
            total_account_balance: U128(env::account_balance()),
            free_account_balance: U128(
                env::account_balance() - env::storage_usage() as u128 * env::storage_byte_cost(),
            ),
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

    pub fn settings(self) -> Settings {
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
    pub fn wf_add_proposed_template_settings(
        self,
        proposal_id: u32,
    ) -> Option<Vec<TemplateSettings>> {
        self.proposed_workflow_settings.get(&proposal_id)
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

    pub fn reward(&self, id: u16) -> Option<Reward> {
        self.rewards.get(&id).map(|r| r.into())
    }
    pub fn reward_list(self, from_id: u16, limit: u16) -> Vec<(u16, Reward)> {
        let mut rewards = Vec::with_capacity(self.reward_last_id as usize);
        for i in from_id..std::cmp::min(self.reward_last_id + 1, limit) {
            if let Some(reward) = self.rewards.get(&i) {
                rewards.push((i, reward.into()));
            }
        }
        rewards
    }
    pub fn partition_list(self, from_id: u16, limit: u16) -> Vec<(u16, TreasuryPartition)> {
        let mut partitions = Vec::with_capacity(self.reward_last_id as usize);
        for i in from_id..std::cmp::min(self.partition_last_id + 1, limit) {
            if let Some(partition) = self.treasury_partition.get(&i) {
                partitions.push((i, partition.into()));
            }
        }
        partitions
    }
    pub fn partition(&self, id: u16) -> Option<TreasuryPartition> {
        self.treasury_partition.get(&id).map(|p| p.into())
    }
    pub fn wallet(&self, account_id: AccountId) -> Option<Wallet> {
        self.wallets.get(&account_id).map(|w| w.into())
    }
    pub fn user_roles(&self, account_id: AccountId) -> Option<UserRoles> {
        self.user_roles.get(&account_id)
    }
    pub fn group_roles(&self, id: u16) -> Option<Roles> {
        self.group_roles.get(&id)
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

    pub fn groups(self) -> Vec<(u16, Group)> {
        self.groups.to_vec()
    }

    pub fn group(&self, id: u16) -> Option<Group> {
        self.groups.get(&id)
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

    pub fn wf_log(self, proposal_id: u32) -> Option<Vec<ActionLog>> {
        self.workflow_activity_log.get(&proposal_id)
    }
    /// Calculate claimable rewards for `account_id`.
    pub fn claimable_rewards(&self, account_id: AccountId) -> ClaimableRewards {
        let wallet: Wallet = self
            .wallets
            .get(&account_id)
            .expect("Wallet not found.")
            .into();
        let mut claimable_rewards = Vec::with_capacity(4);
        let current_timestamp = current_timestamp_sec();
        for wallet_reward in wallet.rewards() {
            if let Some(versioned_reward) = self.rewards.get(&wallet_reward.reward_id()) {
                let reward: Reward = versioned_reward.into();
                for (asset, _) in reward.reward_amounts().into_iter() {
                    let (amount, _) = Contract::internal_claimable_reward_asset(
                        &wallet,
                        wallet_reward.reward_id(),
                        &reward,
                        &asset,
                        current_timestamp,
                    );
                    claimable_rewards.push(ClaimableReward {
                        asset: asset.clone(),
                        reward_id: wallet_reward.reward_id(),
                        amount: amount.into(),
                        partition_id: reward.partition_id,
                    });
                }
            }
        }
        ClaimableRewards {
            claimable_rewards,
            failed_withdraws: wallet
                .failed_withdraws()
                .to_vec()
                .into_iter()
                .map(|(a, v)| (a, v.into()))
                .collect(),
        }
    }

    pub fn media(self, id: u32) -> Option<Media> {
        self.media.get(&id)
    }
    pub fn media_list(self, from_id: u32, limit: u32) -> Vec<(u32, Media)> {
        let mut media_list = Vec::with_capacity(self.media_last_id as usize);
        for i in from_id..std::cmp::min(self.media_last_id + 1, limit) {
            if let Some(media) = self.media.get(&i) {
                media_list.push((i, media.into()));
            }
        }
        media_list
    }
}

#[derive(Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Statistics {
    pub version: u8,
    pub total_delegation_amount: U128,
    pub total_delegators_count: u32,
    pub ft_total_supply: u32,
    pub decimals: u8,
    pub total_members_count: u32,
    pub total_account_balance: U128,
    pub free_account_balance: U128,
}
