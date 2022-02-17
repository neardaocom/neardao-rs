use library::types::DataType;
use library::workflow::{Instance, ProposeSettings, Template, TemplateSettings};
use near_sdk::near_bindgen;
use near_sdk::serde::Serialize;

use crate::group::{GroupMember, GroupOutput};
use crate::media::Media;
use crate::proposal::VProposal;
use crate::settings::DaoSettings;
use crate::tags::Tags;
use crate::TagCategory;
use crate::{core::*, GroupId, GroupName, StorageKey};

#[near_bindgen]
impl Contract {
    pub fn stats(self) -> Stats {
        Stats {
            ft_total_supply: self.ft_total_supply,
            ft_total_locked: self.ft_total_locked,
            ft_total_distributed: self.ft_total_distributed,
            ft_token_holders_count: self.ft.token_holders_count,
            total_members_count: self.total_members_count,
        }
    }

    pub fn proposal(&self, proposal_id: u32) -> Option<(VProposal, Option<Vec<TemplateSettings>>)> {
        self.proposals
            .get(&proposal_id)
            .map(|p| (p, self.proposed_workflow_settings.get(&proposal_id)))
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

    pub fn wf_template(self, id: u16) -> Option<(Template, Vec<TemplateSettings>)> {
        self.workflow_template.get(&id)
    }

    pub fn wf_templates(self) -> Vec<(u16, (Template, Vec<TemplateSettings>))> {
        self.workflow_template.to_vec()
    }

    pub fn wf_instance(self, proposal_id: u32) -> Option<(Instance, ProposeSettings)> {
        self.workflow_instance.get(&proposal_id)
    }

    pub fn wf_instances(self) -> Vec<Option<(Instance, ProposeSettings)>> {
        (1..=self.proposal_last_id)
            .into_iter()
            .map(|i| self.workflow_instance.get(&i))
            .collect()
    }

    pub fn check_transition(
        self,
        proposal_id: u32,
        args: Vec<DataType>,
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

    pub fn media(self, id: u32) -> Option<Media> {
        self.media.get(&id)
    }

    pub fn media_list(self) -> Vec<Option<Media>> {
        let mut media = Vec::with_capacity(self.media_last_id as usize);
        for i in 1..=self.media_last_id {
            media.push(self.media.get(&i))
        }
        media
    }

    pub fn tags(self, category: TagCategory) -> Option<Tags> {
        self.tags.get(&category)
    }

    pub fn storage_bucket_data_all(self, bucket_id: StorageKey) -> Option<Vec<DataType>> {
        self.storage.get(&bucket_id).map(|bucket| {
            bucket
                .get_all_data()
                .into_iter()
                .map(|(_, data)| data)
                .collect()
        })
    }

    pub fn storage_buckets(self) -> Vec<StorageKey> {
        self.storage.keys_as_vector().to_vec()
    }

    pub fn storage_bucket_data(self, bucket_id: StorageKey, data_id: String) -> Option<DataType> {
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
pub struct Stats {
    pub ft_total_supply: u32,
    pub ft_total_locked: u32,
    pub ft_total_distributed: u32,
    pub ft_token_holders_count: u32,
    pub total_members_count: u32,
}
