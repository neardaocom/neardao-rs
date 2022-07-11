use library::functions::utils::calculate_percent_u128;
use library::workflow::action::InputSource;
use library::workflow::instance::Instance;
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::template::Template;
use library::workflow::types::{ActivityRight, VoteScenario};
use library::{derive_from_versioned, derive_into_versioned};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::panic_str;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, require, AccountId};
use std::collections::HashMap;

use crate::internal::utils::current_timestamp_sec;
use crate::media::Media;
use crate::reward::RewardActivity;
use crate::{contract::*, CalculatedVoteResults, VoteTotalPossible, Votes};
use crate::{ResourceId, TimestampSec};

pub const PROPOSAL_DESC_MAX_LENGTH: usize = 256;

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "lowercase")]
pub enum VersionedProposal {
    V1(Proposal),
}

derive_into_versioned!(Proposal, VersionedProposal, V1);
derive_from_versioned!(VersionedProposal, Proposal, V1);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum ProposalState {
    InProgress,
    /// Below quorum limit.
    Invalid,
    /// Above spam threshold.
    Spam,
    /// Below approve threshold.
    Rejected,
    Accepted,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VoteResult {
    Ok,
    AlreadyVoted,
    NoRights,
    InvalidVote,
    VoteEnded,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub desc: ResourceId,
    pub created: TimestampSec,
    pub created_by: AccountId,
    pub end: TimestampSec,
    pub votes: HashMap<AccountId, u8>,
    pub state: ProposalState,
    pub workflow_id: u16,
    pub workflow_settings_id: u8,
    pub voting_results: Vec<U128>,
}

impl Proposal {
    #[inline]
    pub fn new(
        desc: ResourceId,
        created: TimestampSec,
        created_by: AccountId,
        end: TimestampSec,
        workflow_id: u16,
        workflow_settings_id: u8,
    ) -> Self {
        Proposal {
            desc,
            created,
            created_by,
            end,
            votes: HashMap::new(),
            state: ProposalState::InProgress,
            workflow_id,
            workflow_settings_id,
            voting_results: vec![],
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Create proposal that allow to execute workflow once accepted.
    /// If proposing workflow for adding new workflow aka "wf_add"
    /// then `template_settings` must contain at least one `template_settings`
    /// that will be added to the workflow when downloaded from workflow provider.
    /// "wf_add" workflow is is supposed to have template_id 1.
    /// Panics if:
    /// - `template_id` does not refer to existing Template
    /// - `template_settings_id` does not refer to valid TemplateSetting for `template_id`
    /// - `template_settings_id` refer to valid TemplateSettings
    ///  but caller do not have rights to propose them
    /// - `propose_settings.activity_constants` length does not match proposed template activities length.
    /// - `template_id` == 1 (aka "wf_add") but `template_settings` is None
    /// - `propose_settings` contain no storage key but Template requires it or the storage_key already exists
    /// Caller is responsible to provide valid `propose_settings`. This is not checked.
    #[payable]
    pub fn proposal_create(
        &mut self,
        description: Option<Media>,
        template_id: u16,
        template_settings_id: u8,
        propose_settings: ProposeSettings,
        template_settings: Option<Vec<TemplateSettings>>,
        _scheduler_msg: Option<String>,
    ) -> u32 {
        let caller = env::predecessor_account_id();
        let (wft, wfs) = self
            .workflow_template
            .get(&template_id)
            .expect("Template not found.");
        let settings = wfs
            .get(template_settings_id as usize)
            .expect("Settings for template_settings_id not found.");
        require!(env::attached_deposit() >= settings.deposit_propose.unwrap_or_else(|| 0.into()).0);
        require!(
            self.check_rights(&settings.allowed_proposers, &caller),
            "No right to propose with the provided template_settings_id."
        );
        require!(
            propose_settings.activity_constants.len() == wft.activities.len(),
            "ProposeSettings activity_constants does not match template activites."
        );
        if matches!(settings.allowed_voters, ActivityRight::Member)
            && matches!(settings.scenario, VoteScenario::TokenWeighted)
        {
            panic_str(
                "scenario: `TokenWeighted` + allowed_voters: `Members` is not supported yet.",
            );
        }
        self.proposal_last_id += 1;
        if is_wf_add_scenario(&wft, &propose_settings) {
            require!(
                !template_settings
                    .as_ref()
                    .expect("Expected template settings for 'wf_add' proposal.")
                    .is_empty(),
                "Provided `template_settings` do not contain TemplateSettings."
            );
            self.proposed_workflow_settings
                .insert(&self.proposal_last_id, &template_settings.unwrap());
        }
        let created = env::block_timestamp() / 10u64.pow(9) / 60 * 60 + 60;
        let proposal = Proposal::new(
            0,
            created,
            caller,
            created + settings.duration as u64,
            template_id,
            template_settings_id,
        );
        if wft.need_storage {
            if let Some(ref key) = propose_settings.storage_key {
                require!(
                    self.storage.get(key).is_none(),
                    "Storage key already exists."
                );
            } else {
                panic_str("Template requires storage, but no key was provided.");
            }
        }
        self.proposals
            .insert(&self.proposal_last_id, &proposal.into());
        self.workflow_propose_settings
            .insert(&self.proposal_last_id, &propose_settings);
        if let Some(mut media) = description {
            media.proposal_id = Some(self.proposal_last_id);
            self.media_add(&media);
        }
        self.proposal_last_id
    }

    #[payable]
    pub fn proposal_vote(&mut self, id: u32, vote: u8) -> VoteResult {
        if vote > 2 {
            return VoteResult::InvalidVote;
        }
        let caller = env::predecessor_account_id();
        let (mut proposal, _, wfs) = self.get_workflow_and_proposal(id);
        require!(
            env::attached_deposit() >= wfs.deposit_vote.unwrap_or_else(|| 0.into()).0,
            "Not enough deposit."
        );
        let TemplateSettings {
            allowed_voters,
            duration,
            vote_only_once,
            ..
        } = wfs;
        if !self.check_rights(&[allowed_voters], &caller) {
            return VoteResult::NoRights;
        }
        if proposal.state != ProposalState::InProgress
            || proposal.created + (duration as u64) < env::block_timestamp() / 10u64.pow(9)
        {
            return VoteResult::VoteEnded;
        }
        if vote_only_once && proposal.votes.contains_key(&caller) {
            return VoteResult::AlreadyVoted;
        }
        self.register_executed_activity(&caller, RewardActivity::Vote.into());
        proposal.votes.insert(caller, vote);
        self.proposals.insert(&id, &VersionedProposal::V1(proposal));
        VoteResult::Ok
    }

    pub fn proposal_finish(&mut self, id: u32) -> ProposalState {
        let (mut proposal, wft, wfs) = self.get_workflow_and_proposal(id);
        let mut instance =
            Instance::new(proposal.workflow_id, wft.activities.len(), wft.end.clone());
        let propose_settings = self.workflow_propose_settings.get(&id).unwrap();
        let new_state = match proposal.state {
            ProposalState::InProgress => {
                if proposal.created + wfs.duration as u64 > current_timestamp_sec() {
                    None
                } else {
                    let (result_state, vote_results) = self.eval_votes(&proposal.votes, &wfs);
                    if matches!(result_state, ProposalState::Accepted) {
                        instance.init_running(
                            wft.transitions.as_slice(),
                            wfs.transition_limits.as_slice(),
                        );
                        if let Some(ref storage_key) = propose_settings.storage_key {
                            self.storage_bucket_add(storage_key);
                        }
                    }
                    self.register_executed_activity(
                        &proposal.created_by,
                        RewardActivity::AcceptedProposal.into(),
                    );
                    Some((result_state, vote_results))
                }
            }
            _ => None,
        };
        match new_state {
            Some((state, vote_results)) => {
                self.workflow_instance.insert(&id, &instance);
                proposal.state = state;
                proposal.voting_results = vec![
                    vote_results.0.into(),
                    vote_results.1[0].into(),
                    vote_results.1[1].into(),
                    vote_results.1[2].into(),
                ];
                self.proposals.insert(&id, &proposal.into());

                if wft.auto_exec {
                    //TODO: Dispatch wf execution with dao scheduler.
                }
                state
            }
            None => proposal.state,
        }
    }
}

impl Contract {
    /// Evaluate vote results by scenario and type of voters.
    /// Return tuple CalculatedVoteResults.
    /// Scenario: TokenWeighted + Voters: Members is not implemented!
    pub fn calculate_votes(
        &self,
        votes: &HashMap<AccountId, u8>,
        scenario: &VoteScenario,
        vote_target: &ActivityRight,
    ) -> CalculatedVoteResults {
        let mut vote_result: Votes = [0_u128; 3];
        let mut max_possible_amount: VoteTotalPossible = 0;
        match scenario {
            VoteScenario::Democratic => {
                match vote_target {
                    ActivityRight::Anyone => {
                        max_possible_amount = votes.len() as u128;
                    }
                    ActivityRight::Group(g) => match self.groups.get(g) {
                        Some(group) => {
                            max_possible_amount = group.members_count() as u128;
                        }
                        None => panic_str("group not found"),
                    },
                    ActivityRight::GroupMember(_, _)
                    | ActivityRight::Account(_)
                    | ActivityRight::GroupLeader(_) => {
                        max_possible_amount = 1;
                    }
                    ActivityRight::TokenHolder => {
                        max_possible_amount = self.total_delegators_count as u128;
                    }
                    ActivityRight::Member => {
                        max_possible_amount = self.total_members_count as u128;
                    }
                    ActivityRight::GroupRole(g, r) => match self.groups.get(g) {
                        Some(group) => {
                            max_possible_amount =
                                self.get_group_members_with_role(*g, &group, *r).len() as u128;
                        }
                        None => panic_str("group not found"),
                    },
                }

                if matches!(vote_target, ActivityRight::Member) {
                    for (voter, vote_value) in votes.iter() {
                        if self.user_roles.get(voter).is_some() {
                            vote_result[*vote_value as usize] += 1;
                        }
                    }
                } else {
                    for vote_value in votes.values() {
                        vote_result[*vote_value as usize] += 1;
                    }
                }
            }
            VoteScenario::TokenWeighted => match vote_target {
                ActivityRight::Anyone | ActivityRight::TokenHolder => {
                    max_possible_amount = self.total_delegation_amount;
                    for (voter, vote_value) in votes.iter() {
                        vote_result[*vote_value as usize] +=
                            self.delegations.get(voter).unwrap_or(0);
                    }
                }
                ActivityRight::Member => {
                    unreachable!()
                }
                ActivityRight::Group(g) => {
                    match self.groups.get(g) {
                        Some(group) => {
                            let members = group.get_members_accounts();
                            for member in members {
                                let member_vote_weight = self.delegations.get(&member).unwrap_or(0);
                                max_possible_amount += member_vote_weight;
                                if let Some(vote_value) = votes.get(&member) {
                                    vote_result[*vote_value as usize] += member_vote_weight;
                                }
                            }
                        }
                        None => panic_str("group not found"),
                    };
                }
                // Expensive scenario.
                ActivityRight::GroupRole(g, r) => {
                    match self.groups.get(g) {
                        Some(group) => {
                            let members = group.get_members_accounts();
                            for member in members {
                                let member_vote_weight = self.delegations.get(&member).unwrap_or(0);
                                // Group member always has role record, therefore unwraping is ok.
                                let member_roles = self.user_roles.get(&member).unwrap();
                                if member_roles.has_group_role(*g, *r) {
                                    max_possible_amount += member_vote_weight;
                                    if let Some(vote_value) = votes.get(&member) {
                                        vote_result[*vote_value as usize] += member_vote_weight;
                                    }
                                }
                            }
                        }
                        None => panic_str("group not found"),
                    };
                }
                ActivityRight::GroupMember(g, account_id) => {
                    match self.groups.get(g) {
                        Some(group) => {
                            if group.is_member(account_id) {
                                let member_vote_weight =
                                    self.delegations.get(account_id).unwrap_or(0);
                                max_possible_amount += member_vote_weight;
                                if let Some(vote_value) = votes.get(account_id) {
                                    vote_result[*vote_value as usize] += member_vote_weight;
                                }
                            }
                        }
                        None => panic_str("group not found"),
                    };
                }
                ActivityRight::Account(account_id) => {
                    let member_vote_weight = self.delegations.get(account_id).unwrap_or(0);
                    max_possible_amount += member_vote_weight;
                    if let Some(vote_value) = votes.get(account_id) {
                        vote_result[*vote_value as usize] += member_vote_weight;
                    }
                }
                ActivityRight::GroupLeader(g) => {
                    match self.groups.get(g) {
                        Some(group) => {
                            if let Some(leader) = group.group_leader() {
                                let member_vote_weight = self.delegations.get(leader).unwrap_or(0);
                                max_possible_amount += member_vote_weight;
                                if let Some(vote_value) = votes.get(leader) {
                                    vote_result[*vote_value as usize] += member_vote_weight;
                                }
                            }
                        }
                        None => panic_str("group not found"),
                    };
                }
            },
        }
        (max_possible_amount, vote_result)
    }

    /// Evaluates proposal voting according to vote settings.
    pub fn eval_votes(
        &self,
        proposal_votes: &HashMap<AccountId, u8>,
        settings: &TemplateSettings,
    ) -> (ProposalState, CalculatedVoteResults) {
        let (max_possible_amount, vote_results) =
            self.calculate_votes(proposal_votes, &settings.scenario, &settings.allowed_voters);
        let votes_sum = vote_results.iter().sum::<u128>();
        log!("Votes: {}, {:?}", max_possible_amount, vote_results);
        let state = if calculate_percent_u128(vote_results[0], max_possible_amount)
            >= settings.spam_threshold
        {
            log!(
                "spam th: {}, max_possible: {}, current: {}",
                settings.spam_threshold,
                max_possible_amount,
                vote_results[0],
            );
            ProposalState::Spam
        } else if calculate_percent_u128(votes_sum, max_possible_amount) < settings.quorum {
            log!(
                "quorum th: {}, max_possible: {}, current: {}",
                settings.quorum,
                max_possible_amount,
                votes_sum,
            );
            ProposalState::Invalid
        } else if calculate_percent_u128(vote_results[1], votes_sum) < settings.approve_threshold {
            log!(
                "appprove th: {}, max_possible: {}, current: {}",
                settings.approve_threshold,
                votes_sum,
                vote_results[1],
            );
            ProposalState::Rejected
        } else {
            ProposalState::Accepted
        };
        (state, (max_possible_amount, vote_results))
    }
}

/// Check if proposal with referenced template and provided propose_settings
/// requires TemplateSettings for the workflow "wf_add" scenario.
/// Assumptions this function uses:
/// - "wf_add" activity is only 1 activity its kind in the workflow and it has only 1 action.
/// - all activities have non-empty actions
/// - propose_settings have defined activity_constants for all activities.
fn is_wf_add_scenario(template: &Template, propose_settings: &ProposeSettings) -> bool {
    for (idx, a) in template.activities.iter().enumerate().skip(1) {
        let activity = a.activity_as_ref().unwrap();
        if activity.code.as_str() == "wf_add" {
            return !(matches!(
                activity
                    .actions
                    .get(0)
                    .expect("Invalid wf - empty actions in activity 0")
                    .input_source,
                InputSource::PropSettings
            ) && propose_settings
                .activity_constants
                .get(idx)
                .expect("Invalid ProposeSettings - missing constants for action")
                .is_none());
        }
    }
    false
}
