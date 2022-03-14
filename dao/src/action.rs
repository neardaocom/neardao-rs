use std::collections::HashMap;

use library::types::DataType;
use library::workflow::types::ActionResult;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId};

use crate::proposal::{Proposal, ProposalContent};
use crate::release::ReleaseDb;
use crate::settings::DaoSettings;
use crate::tags::Tags;
use crate::{
    core::*,
    group::{GroupInput, GroupMember, GroupReleaseInput, GroupSettings},
    GroupId, ProposalId,
};
use crate::{TagCategory, TagId};

#[near_bindgen]
impl Contract {
/*     //TODO destructuring and validations for input objs
    pub fn group_add(
        &mut self,
        proposal_id: ProposalId,
        settings: GroupSettings,
        members: Vec<GroupMember>,
        token_lock: GroupReleaseInput,
    ) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::GroupAdd,
            None,
            None,
            &mut vec![vec![]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            self.add_group(GroupInput {
                settings,
                members,
                release: token_lock,
            });
        }

        result
    }
    pub fn group_remove(&mut self, proposal_id: ProposalId, id: GroupId) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::GroupRemove,
            None,
            None,
            &mut vec![vec![DataType::U16(id)]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.groups.remove(&id) {
                Some(mut group) => {
                    let release: ReleaseDb = group.remove_storage_data().data.into();
                    self.ft_total_locked -= release.total - release.init_distribution;
                    self.total_members_count -= group.members.members_count() as u32;
                }
                _ => (),
            }
        }

        result
    }

    //TODO destructuring and validations for input objs
    pub fn group_update(
        &mut self,
        proposal_id: ProposalId,
        id: GroupId,
        settings: GroupSettings,
    ) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::GroupUpdate,
            None,
            None,
            &mut vec![vec![DataType::U16(id)]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.groups.get(&id) {
                Some(mut group) => {
                    group.settings = settings;
                    self.groups.insert(&id, &group);
                }
                _ => (),
            }
        }

        result
    }

    //TODO destructuring and validations for input objs
    pub fn group_add_members(
        &mut self,
        proposal_id: ProposalId,
        id: GroupId,
        members: Vec<GroupMember>,
    ) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::GroupAddMembers,
            None,
            None,
            &mut vec![vec![DataType::U16(id)]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.groups.get(&id) {
                Some(mut group) => {
                    self.total_members_count += group.add_members(members);
                    self.groups.insert(&id, &group);
                }
                _ => (),
            }
        }

        result
    }

    pub fn group_remove_member(
        &mut self,
        proposal_id: ProposalId,
        id: GroupId,
        member: AccountId,
    ) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::GroupRemoveMember,
            None,
            None,
            &mut vec![vec![DataType::U16(id), DataType::String(member.clone())]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.groups.get(&id) {
                Some(mut group) => {
                    group.remove_member(member);
                    self.total_members_count -= 1;
                    self.groups.insert(&id, &group);
                }
                _ => (),
            }
        }

        result
    }

    #[allow(unused_variables)]
    pub fn settings_update(&mut self, proposal_id: ProposalId, settings: DaoSettings) {
        unimplemented!();
        /*
        assert_valid_dao_settings(&settings);
        self.settings.replace(&settings.into());
        */
    }

    pub fn media_add(&mut self, proposal_id: ProposalId) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::MediaAdd,
            None,
            None,
            &mut vec![vec![]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            let proposal: Proposal = self.proposals.get(&proposal_id).unwrap().into();

            let mut media = match proposal.content.unwrap() {
                ProposalContent::Media(m) => m,
            };

            media.proposal_id = proposal_id;
            self.media_last_id += 1;
            self.media.insert(&self.media_last_id, &media);
        }

        result
    }
    pub fn media_invalidate(&mut self, proposal_id: ProposalId, id: u32) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::MediaInvalidate,
            None,
            None,
            &mut vec![vec![DataType::U32(id)]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.media.get(&id) {
                Some(mut media) => {
                    media.valid = false;
                    self.media.insert(&id, &media);
                }
                _ => (),
            }
        }

        result
    }
    pub fn media_remove(&mut self, proposal_id: ProposalId, id: u32) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::MediaRemove,
            None,
            None,
            &mut vec![vec![DataType::U32(id)]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            self.media.remove(&id);
        }

        result
    }

    /// Returns tuple of start, end index for the new tags
    pub fn tag_add(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        tags: Vec<String>,
    ) -> Option<(TagId, TagId)> {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TagAdd,
            None,
            None,
            &mut vec![vec![
                DataType::String(category.clone()),
                DataType::VecString(tags.clone()),
            ]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            let mut t = self.tags.get(&category).unwrap_or(Tags::new());
            let ids = t.insert(tags);
            self.tags.insert(&category, &t);
            ids
        } else {
            None
        }
    }

    pub fn tag_edit(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        id: TagId,
        value: String,
    ) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TagAdd,
            None,
            None,
            &mut vec![vec![
                DataType::String(category.clone()),
                DataType::U16(id),
                DataType::String(value.clone()),
            ]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.tags.get(&category) {
                Some(mut t) => {
                    t.rename(id, value);
                    self.tags.insert(&category, &t);
                }
                None => (),
            }
        }
        result
    }

    pub fn tag_remove(
        &mut self,
        proposal_id: ProposalId,
        category: TagCategory,
        id: TagId,
    ) -> ActionResult {
        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TagAdd,
            None,
            None,
            &mut vec![vec![DataType::String(category.clone()), DataType::U16(id)]],
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            match self.tags.get(&category) {
                Some(mut t) => {
                    t.remove(id);
                    self.tags.insert(&category, &t);
                }
                None => (),
            }
        }
        result
    }

    pub fn ft_distribute(
        &mut self,
        proposal_id: ProposalId,
        group_id: u16,
        amount: u32,
        account_ids: Vec<AccountId>,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::U16(group_id),
            DataType::U32(amount),
            DataType::VecString(account_ids),
        ]];

        let result = self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::FtDistribute,
            None,
            None,
            &mut args,
            &mut vec![vec![]],
            None,
        );

        if result == ActionResult::Ok || result == ActionResult::Postprocessing {
            if let Some(mut group) = self
                .groups
                .get(&(args[0][0].clone().try_into_u128().unwrap() as u16))
            {
                let account_ids = args[0].pop().unwrap().try_into_vec_str().unwrap();
                let amount = args[0].pop().unwrap().try_into_u128().unwrap() as u32;
                match group.distribute_ft(amount) && account_ids.len() > 0 {
                    true => {
                        self.groups.insert(&group_id, &group);
                        self.distribute_ft(amount, &account_ids);
                    }
                    _ => (),
                }
            }
        }

        result
    }

    pub fn treasury_send_near(
        &mut self,
        proposal_id: ProposalId,
        receiver_id: AccountId,
        amount: U128,
    ) -> ActionResult {
        let mut args = vec![vec![DataType::String(receiver_id), DataType::U128(amount)]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TreasurySendNear,
            None,
            None,
            &mut args,
            &mut vec![vec![]],
            None,
        )
    }

    pub fn treasury_send_ft(
        &mut self,
        proposal_id: ProposalId,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(ft_account_id),
            DataType::String(receiver_id),
            DataType::U128(amount),
            DataType::String(memo.unwrap_or_default()),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TreasurySendFt,
            None,
            None,
            &mut args,
            &mut vec![vec![]],
            None,
        )
    }

    pub fn treasury_send_ft_contract(
        &mut self,
        proposal_id: ProposalId,
        ft_account_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(ft_account_id),
            DataType::String(receiver_id),
            DataType::U128(amount),
            DataType::String(memo.unwrap_or_default()),
            DataType::String(msg),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TreasurySendFtContract,
            None,
            None,
            &mut args,
            &mut vec![vec![]],
            None,
        )
    }

    //TODO check correct NFT usage
    pub fn treasury_send_nft(
        &mut self,
        proposal_id: ProposalId,
        nft_account_id: AccountId,
        receiver_id: String,
        nft_id: String,
        memo: Option<String>,
        approval_id: u32,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(nft_account_id),
            DataType::String(receiver_id),
            DataType::String(nft_id),
            DataType::String(memo.unwrap_or_default()),
            DataType::U32(approval_id),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TreasurySendNft,
            None,
            None,
            &mut args,
            &mut vec![vec![]],
            None,
        )
    }

    //TODO check correct NFT usage
    pub fn treasury_send_nft_contract(
        &mut self,
        proposal_id: ProposalId,
        nft_account_id: AccountId,
        receiver_id: String,
        nft_id: String,
        memo: Option<String>,
        approval_id: u32,
        msg: String,
    ) -> ActionResult {
        let mut args = vec![vec![
            DataType::String(nft_account_id),
            DataType::String(receiver_id),
            DataType::String(nft_id),
            DataType::String(memo.unwrap_or_default()),
            DataType::U32(approval_id),
            DataType::String(msg),
        ]];

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::TreasurySendNFtContract,
            None,
            None,
            &mut args,
            &mut vec![vec![]],
            None,
        )
    }

    /// Invokes custom function call
    #[allow(unused_mut)]
    pub fn fn_call(
        &mut self,
        proposal_id: ProposalId,
        fncall_receiver: AccountId,
        fncall_method: MethodName,
        mut arg_values: Vec<Vec<DataType>>,
        mut arg_values_collection: Vec<Vec<DataType>>,
    ) -> ActionResult {
        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::FnCall,
            None,
            Some((fncall_receiver, fncall_method)),
            &mut arg_values,
            &mut arg_values_collection,
            None,
        )
    }

    pub fn workflow_add(&mut self, proposal_id: ProposalId, workflow_id: u16) -> ActionResult {
        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::WorkflowAdd,
            None,
            None,
            &mut vec![vec![DataType::U16(workflow_id)]],
            &mut vec![],
            None,
        )
    }

    /// Represents custom event which does not invokes any action except transition in workflow and saving data to storage when needed.
    #[payable]
    #[allow(unused_mut)]
    pub fn event(
        &mut self,
        proposal_id: ProposalId,
        code: String,
        mut args: Vec<DataType>,
    ) -> ActionResult {
        // Oth value must always be predecessor
        args.insert(0, DataType::String(env::predecessor_account_id()));

        self.execute_action(
            env::predecessor_account_id(),
            proposal_id,
            ActionType::Event,
            Some(code),
            None,
            &mut vec![args],
            &mut vec![],
            Some(env::attached_deposit()),
        )
    } */
}
