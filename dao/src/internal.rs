use std::{collections::HashMap, convert::TryFrom};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, UnorderedSet},
    env::{self},
    json_types::U128,
    log,
    serde::{Deserialize, Serialize},
    AccountId, Balance, IntoStorageKey, Promise,
};

use crate::{
    action::{FnCallDefinition, FnCallMetadata},
    append,
    callbacks::ext_self,
    constants::{
        ACC_REF_FINANCE, ACC_SKYWARD_FINANCE, ACC_WNEAR, DEFAULT_DOC_CAT, DEPOSIT_STANDARD_STORAGE,
        GROUP_PREFIX, GROUP_RELEASE_PREFIX,
    },
    core::{Contract, StorageKeyWrapper},
    errors::{
        ERR_DISTRIBUTION_ACC_EMPTY, ERR_GROUP_NOT_FOUND, ERR_LOCK_AMOUNT_ABOVE,
        ERR_STORAGE_BUCKET_EXISTS,
    },
    group::{Group, GroupInput},
    media::Media,
    proposal::Proposal,
    release::{Release, ReleaseDb, ReleaseModel, ReleaseModelInput, VReleaseDb, VReleaseModel},
    settings::{DaoSettings, VoteSettings},
    tags::{TagInput, Tags},
    GroupId, ProposalId,
};
use library::{
    storage::StorageBucket,
    workflow::{
        ActivityRight, VoteScenario, Instance, InstanceState, Settings,
        Template,
    },
};

impl Contract {
    #[inline]
    pub fn init_dao_settings(&mut self, settings: DaoSettings) {
        self.settings.set(&settings.into());
    }
    #[inline]
    pub fn init_vote_settings(&mut self, settings: Vec<VoteSettings>) {
        self.vote_settings
            .set(&settings.into_iter().map(|v| v.into()).collect());
    }

    #[inline]
    pub fn init_tags(&mut self, tags: Vec<TagInput>) {
        for i in tags.into_iter() {
            let mut tags = Tags::new();
            tags.insert(i.values);
            self.tags.insert(&i.category, &tags);
        }
    }

    #[inline]
    pub fn init_groups(&mut self, groups: Vec<GroupInput>) {
        for g in groups.into_iter() {
            self.add_group(g);
        }

        assert!(
            self.ft_total_supply >= self.ft_total_locked,
            "{}",
            ERR_LOCK_AMOUNT_ABOVE
        );
    }

    #[inline]
    pub fn init_media(&mut self, media: Vec<Media>) {
        for (i, m) in media.iter().enumerate() {
            self.media.insert(&(i as u32), m);
        }

        self.media_last_id = media.len() as u32;
    }

    #[inline]
    pub fn init_function_calls(
        &mut self,
        calls: Vec<FnCallDefinition>,
        metadata: Vec<Vec<FnCallMetadata>>,
    ) {
        for (i, c) in calls.iter().enumerate() {
            let key = format!("{}_{}", c.name, c.receiver); //TODO replace format! with push_str
            self.function_calls.insert(&key, &c);
            self.function_call_metadata.insert(&key, &metadata[i]);
        }
    }

    #[inline]
    pub fn init_workflows(
        &mut self,
        mut workflows: Vec<Template>,
        mut workflow_template_settings: Vec<Vec<Settings>>,
    ) {
        // Each workflow must have at least one setting
        assert_eq!(workflows.len(), workflow_template_settings.len());
        for _ in 0..workflows.len() {
            self.workflow_last_id += 1;
            self.workflow_template.insert(
                &self.workflow_last_id,
                &(
                    workflows.pop().unwrap(),
                    workflow_template_settings.pop().unwrap(),
                ),
            );
        }
    }

    pub fn get_wf_and_proposal(
        &self,
        proposal_id: u32,
    ) -> (Proposal, Template, Settings) {
        let proposal =
            Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));
        let (wft, mut wfs) = self.workflow_template.get(&proposal.workflow_id).unwrap();
        let settings = wfs.swap_remove(proposal.workflow_settings_id as usize);

        (proposal, wft, settings)
    }

    // TODO unit tests
    pub fn check_rights(&self, rights: &Vec<ActivityRight>, account_id: &AccountId) -> bool {
        if rights.len() == 0 {
            return true;
        }

        for right in rights.iter() {
            match right {
                ActivityRight::Anyone => {
                    return true;
                }
                ActivityRight::Group(g) => match self.groups.get(g) {
                    Some(group) => match group.get_member_by_account(account_id) {
                        Some(_m) => return true,
                        None => continue,
                    },
                    _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                },
                ActivityRight::GroupMember(g, name) => {
                    if name != account_id {
                        continue;
                    }

                    match self.groups.get(g) {
                        Some(group) => match group.get_member_by_account(account_id) {
                            Some(_m) => return true,
                            None => continue,
                        },
                        _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                    }
                }
                ActivityRight::TokenHolder => match self.ft.accounts.get(account_id) {
                    Some(ft) if ft > 0 => {
                        return true;
                    }
                    _ => continue,
                },
                ActivityRight::GroupRole(g, r) => match self.groups.get(g) {
                    Some(group) => match group.get_member_by_account(account_id) {
                        Some(m) => match m.tags.into_iter().any(|t| t == *r) {
                            true => return true,
                            false => continue,
                        },
                        None => continue,
                    },
                    _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                },
                ActivityRight::GroupLeader(g) => match self.groups.get(g) {
                    Some(group) => match group.settings.leader == *account_id {
                        true => return true,
                        false => continue,
                    },
                    _ => panic!("{}", ERR_GROUP_NOT_FOUND),
                },
                ActivityRight::Member => {
                    match self.ft.accounts.get(account_id) {
                        Some(ft) if ft > 0 => {
                            return true;
                        }
                        _ => {
                            // Yep this is expensive...
                            // Iterate all groups and all members
                            let groups = self.groups.to_vec();

                            match groups
                                .into_iter()
                                .any(|(_, g)| g.get_member_by_account(account_id).is_some())
                            {
                                true => return true,
                                false => continue,
                            }
                        }
                    }
                }
                ActivityRight::Account(a) => match a == account_id {
                    true => return true,
                    false => continue,
                },
            }
        }
        false
    }

    // TODO test
    /// Evaluates vote results by scenario and type of voters.
    /// Returns tuple (total_voted_amount,vote_results)
    pub fn calculate_votes(
        &self,
        votes: &HashMap<String, u8>,
        scenario: &VoteScenario,
        vote_target: &ActivityRight,
    ) -> (u128, [u128; 3]) {
        // count votes
        let mut vote_result = [0 as u128; 3];
        let mut total_voted_amount: u128 = 0;
        match scenario {
            VoteScenario::Democratic => {
                match vote_target {
                    ActivityRight::Anyone => {
                        total_voted_amount = votes.len() as u128;
                    }
                    ActivityRight::Group(g) => match self.groups.get(&g) {
                        Some(group) => {
                            total_voted_amount = group.members.members_count() as u128;
                        }
                        None => panic!("{}", ERR_GROUP_NOT_FOUND),
                    },
                    ActivityRight::GroupMember(_, _)
                    | ActivityRight::Account(_)
                    | ActivityRight::GroupLeader(_) => {
                        total_voted_amount = 1;
                    }
                    ActivityRight::TokenHolder => todo!(),
                    ActivityRight::Member => todo!(),
                    ActivityRight::GroupRole(g, r) => match self.groups.get(&g) {
                        Some(group) => {
                            total_voted_amount = group.get_members_by_role(*r).len() as u128;
                        }
                        None => panic!("{}", ERR_GROUP_NOT_FOUND),
                    },
                }

                //calculate votes
                for vote_value in votes.values() {
                    vote_result[*vote_value as usize] += 1;
                }
            }
            VoteScenario::TokenWeighted => match vote_target {
                ActivityRight::Anyone | ActivityRight::TokenHolder | ActivityRight::Member => {
                    total_voted_amount = self.ft_total_distributed as u128 * self.decimal_const;

                    for (voter, vote_value) in votes.iter() {
                        vote_result[*vote_value as usize] +=
                            self.ft.accounts.get(voter).unwrap_or(0);
                    }
                }
                ActivityRight::Group(_) | ActivityRight::GroupRole(_, _) => {
                    for (voter, vote_value) in votes.iter() {
                        let value = self.ft.accounts.get(voter).unwrap_or(0);
                        vote_result[*vote_value as usize] += value;
                        total_voted_amount += value;
                    }
                }
                ActivityRight::GroupMember(_, _)
                | ActivityRight::Account(_)
                | ActivityRight::GroupLeader(_) => {
                    total_voted_amount = 1;
                    for vote_value in votes.values() {
                        vote_result[*vote_value as usize] += 1;
                    }
                }
            },
        }

        (total_voted_amount, vote_result)
    }

    pub fn find_current_workflow_activity(&self, proposal_id: u32) -> Option<Instance> {
        match self.workflow_instance.get(&proposal_id) {
            Some(i) => match i.state {
                InstanceState::Running => {
                    //i.current_action
                    None
                }
                _ => None,
            },
            None => None,
        }
    }

    pub fn storage_bucket_add(&mut self, bucket_id: &str) {
        let bucket = StorageBucket::new(utils::get_bucket_id(bucket_id));
        assert!(
            self.storage
                .insert(&bucket_id.to_owned(), &bucket)
                .is_none(),
            "{}",
            ERR_STORAGE_BUCKET_EXISTS
        );
    }

    pub fn add_group(&mut self, group: GroupInput) {
        self.ft_total_locked += group.release.amount;

        // Check if we can do init distribution
        if group.release.init_distribution > 0 {
            self.distribute_ft(
                group.release.init_distribution,
                &group
                    .members
                    .iter()
                    .map(|member| member.account_id.clone())
                    .collect::<Vec<AccountId>>(), //TODO optimalize
            );
        }

        let release: Release = group.release.into();

        // Create StorageKey for nested structure
        self.group_last_id += 1;
        let release_key = utils::get_group_key(self.group_last_id);

        self.groups.insert(
            &self.group_last_id,
            &Group::new(release_key, group.settings, group.members, release),
        );
    }

    /// Internally transfers FT from contract account all accounts equally
    /// Sets contract's ft_total_distributed property
    /// Panics if account_ids are empty vector
    pub fn distribute_ft(&mut self, amount: u32, account_ids: &[AccountId]) {
        assert!(account_ids.len() > 0, "{}", ERR_DISTRIBUTION_ACC_EMPTY);
        let amount_per_acc = (amount / account_ids.len() as u32) as u128 * self.decimal_const;
        self.ft_total_distributed += amount * account_ids.len() as u32;
        let contract_account_id = env::current_account_id();
        for acc in account_ids {
            // If not registered when distributing ft, we register them, assuming payment is solved by other mechanisms
            if !self.ft.accounts.contains_key(&acc) {
                self.ft.accounts.insert(&acc, &0);
            }

            self.ft
                .internal_transfer(&contract_account_id, acc, amount_per_acc, None);
        }
    }
}

pub mod utils {
    use crate::{
        append,
        constants::{GROUP_RELEASE_PREFIX, STORAGE_BUCKET_PREFIX},
        core::StorageKeyWrapper,
        GroupId,
    };

    pub fn get_group_key(id: GroupId) -> StorageKeyWrapper {
        append(GROUP_RELEASE_PREFIX, &id.to_le_bytes()).into()
    }

    pub fn get_bucket_id(id: &str) -> StorageKeyWrapper {
        append(STORAGE_BUCKET_PREFIX, id.as_bytes()).into()
    }
}
