use library::functions::math::calculate_percent_u128;
use library::workflow::instance::Instance;
use library::workflow::settings::{ProposeSettings, TemplateSettings};
use library::workflow::types::{ActivityRight, VoteScenario};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, AccountId};
use std::collections::HashMap;

use crate::error::ERR_GROUP_NOT_FOUND;
use crate::media::{Media, ResourceType};
use crate::reward::RewardActivity;
use crate::{
    calc_percent_u128_unchecked, core::*, CalculatedVoteResults, VoteTotalPossible, Votes,
};
use crate::{ResourceId, TimestampSec};

pub const PROPOSAL_DESC_MAX_LENGTH: usize = 256;

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum VersionedProposal {
    Current(Proposal),
}

impl From<VersionedProposal> for Proposal {
    fn from(fm: VersionedProposal) -> Self {
        match fm {
            VersionedProposal::Current(p) => p,
            _ => unimplemented!(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Clone)]
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
    pub votes: HashMap<AccountId, u8>,
    pub state: ProposalState,
    pub workflow_id: u16,
    pub workflow_settings_id: u8,
}

impl Proposal {
    #[inline]
    pub fn new(
        desc: ResourceId,
        created: u64,
        created_by: AccountId,
        workflow_id: u16,
        workflow_settings_id: u8,
    ) -> Self {
        Proposal {
            desc,
            created,
            created_by,
            votes: HashMap::new(),
            state: ProposalState::InProgress,
            workflow_id,
            workflow_settings_id,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalContent {
    Media(Media),
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn proposal_create(
        &mut self,
        desc: ResourceType, // TODO: Optional
        template_id: u16,
        template_settings_id: u8,
        propose_settings: ProposeSettings,
        template_settings: Option<Vec<TemplateSettings>>,
    ) -> u32 {
        let caller = env::predecessor_account_id();
        let (wft, wfs) = self.workflow_template.get(&template_id).unwrap();
        let settings = wfs
            .get(template_settings_id as usize)
            .expect("Undefined settings id");
        assert!(env::attached_deposit() >= settings.deposit_propose.unwrap_or_else(|| 0.into()).0);
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
        // TODO: Implement resource provider.
        let proposal = Proposal::new(
            0,
            env::block_timestamp() / 10u64.pow(9) / 60 * 60 + 60, // Rounded up to minutes
            caller,
            template_id,
            template_settings_id,
        );
        if wft.need_storage {
            if let Some(ref key) = propose_settings.storage_key {
                assert!(
                    self.storage.get(key).is_none(),
                    "Storage key already exists."
                );
            } else {
                panic!("Template requires storage, but no key was provided.");
            }
        }
        // TODO: Refactor.
        // Check that proposal binds have valid structure.
        //self.assert_valid_proposal_binds_structure(
        //    propose_settings.binds.as_slice(),
        //    wft.activities.as_slice(),
        //);
        self.proposals.insert(
            &self.proposal_last_id,
            &VersionedProposal::Current(proposal),
        );
        self.workflow_propose_settings
            .insert(&self.proposal_last_id, &propose_settings);
        // TODO: Croncat registration to finish proposal
        self.proposal_last_id
    }

    #[payable]
    pub fn proposal_vote(&mut self, id: u32, vote: u8) -> VoteResult {
        if vote > 2 {
            return VoteResult::InvalidVote;
        }
        let caller = env::predecessor_account_id();
        let (mut proposal, _, wfs) = self.get_workflow_and_proposal(id);
        assert!(
            env::attached_deposit() >= wfs.deposit_vote.unwrap_or_else(|| 0.into()).0,
            "{}",
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
        self.proposals
            .insert(&id, &VersionedProposal::Current(proposal));
        VoteResult::Ok
    }

    pub fn proposal_finish(&mut self, id: u32) -> ProposalState {
        let caller = env::predecessor_account_id();
        let (mut proposal, wft, wfs) = self.get_workflow_and_proposal(id);
        let mut instance =
            Instance::new(proposal.workflow_id, wft.activities.len(), wft.end.clone());
        let propose_settings = self.workflow_propose_settings.get(&id).unwrap();
        let new_state = match proposal.state {
            ProposalState::InProgress => {
                if proposal.created + wfs.duration as u64 > env::block_timestamp() / 10u64.pow(9) {
                    None
                } else {
                    let vote_result = self.eval_votes(&proposal.votes, &wfs);
                    if matches!(vote_result, ProposalState::Accepted) {
                        instance.init_running(
                            wft.transitions.as_slice(),
                            wfs.transition_limits.as_slice(),
                        );
                        if let Some(ref storage_key) = propose_settings.storage_key {
                            self.storage_bucket_add(storage_key);
                        }
                    }
                    self.register_executed_activity(
                        &caller,
                        RewardActivity::AcceptedProposal.into(),
                    );
                    Some(vote_result)
                }
            }
            _ => None,
        };

        match new_state {
            Some(state) => {
                self.workflow_instance.insert(&id, &instance);
                proposal.state = state.clone();
                self.proposals
                    .insert(&id, &VersionedProposal::Current(proposal));

                if wft.auto_exec {
                    //TODO: Dispatch wf execution with Croncat.
                }

                state
            }
            None => proposal.state,
        }
    }
}

impl Contract {
    /// Evaluates vote results by scenario and type of voters.
    /// Returns tuple (max_possible_amount,vote_results)
    #[allow(unused)]
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
                            max_possible_amount = group.members.members_count() as u128;
                        }
                        None => panic!("{}", ERR_GROUP_NOT_FOUND),
                    },
                    ActivityRight::GroupMember(_, _)
                    | ActivityRight::Account(_)
                    | ActivityRight::GroupLeader(_) => {
                        max_possible_amount = 1;
                    }
                    ActivityRight::TokenHolder => {
                        unimplemented!()
                    }
                    // If member exists in 2 groups, then he is accounted twice.
                    ActivityRight::Member => {
                        max_possible_amount = self.total_members_count as u128;
                    }
                    ActivityRight::GroupRole(g, r) => match self.groups.get(g) {
                        Some(group) => {
                            max_possible_amount =
                                group.get_members_accounts_by_role(*r).len() as u128;
                        }
                        None => panic!("{}", ERR_GROUP_NOT_FOUND),
                    },
                }

                for vote_value in votes.values() {
                    vote_result[*vote_value as usize] += 1;
                }
            }
            VoteScenario::TokenWeighted => match vote_target {
                ActivityRight::Anyone | ActivityRight::TokenHolder => unimplemented!(),
                // This is expensive scenario
                ActivityRight::Member => unimplemented!(),
                ActivityRight::Group(gid) => unimplemented!(),
                ActivityRight::GroupRole(gid, rid) => unimplemented!(),
                ActivityRight::GroupMember(_, _)
                | ActivityRight::Account(_)
                | ActivityRight::GroupLeader(_) => {
                    max_possible_amount = 1;
                    for vote_value in votes.values() {
                        vote_result[*vote_value as usize] += 1;
                    }
                }
            },
        }

        (max_possible_amount, vote_result)
    }

    /// TODO: cross unit tests.
    /// Evaluates proposal voting according to vote settings.
    pub fn eval_votes(
        &self,
        proposal_votes: &HashMap<AccountId, u8>,
        settings: &TemplateSettings,
    ) -> ProposalState {
        let (max_possible_amount, vote_results) =
            self.calculate_votes(proposal_votes, &settings.scenario, &settings.allowed_voters);
        log!("Votes: {}, {:?}", max_possible_amount, vote_results);
        let decimals = if matches!(settings.scenario, VoteScenario::Democratic) {
            1
        } else {
            10u128.pow(self.decimals as u32)
        };
        if calculate_percent_u128(vote_results[0] * decimals, max_possible_amount * decimals)
            >= settings.spam_threshold
        {
            log!(
                "spam th: {}, max_possible: {}, current: {}",
                settings.spam_threshold,
                max_possible_amount * decimals,
                vote_results[0] * decimals,
            );
            ProposalState::Spam
        } else if calculate_percent_u128(
            vote_results.iter().sum::<u128>() * decimals,
            max_possible_amount * decimals,
        ) < settings.quorum
        {
            log!(
                "quorum th: {}, max_possible: {}, current: {}",
                settings.quorum,
                max_possible_amount * decimals,
                vote_results.iter().sum::<u128>() * decimals,
            );
            ProposalState::Invalid
        } else if calculate_percent_u128(
            vote_results[1] * decimals,
            vote_results.iter().sum::<u128>() * decimals,
        ) < settings.approve_threshold
        {
            log!(
                "appprove th: {}, max_possible: {}, current: {}",
                settings.approve_threshold,
                vote_results.iter().sum::<u128>() * decimals,
                vote_results[1] * decimals,
            );
            ProposalState::Rejected
        } else {
            ProposalState::Accepted
        }
    }
}
