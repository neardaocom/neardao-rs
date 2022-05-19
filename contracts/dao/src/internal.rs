use near_sdk::{env, require, AccountId, Balance};

use crate::{
    constants::{C_CURRENT_TIMESTAMP_SECS, C_DAO_ID, C_PREDECESSOR},
    core::{ActivityLog, Contract},
    error::ERR_LOCK_AMOUNT_OVERFLOW,
    group::GroupInput,
    internal::utils::current_timestamp_sec,
    proposal::Proposal,
    settings::Settings,
    tags::{TagInput, Tags},
    treasury::{TreasuryPartition, TreasuryPartitionInput},
    ProposalId, ProposalWf,
};
use library::{
    storage::StorageBucket,
    types::{consts::Consts, datatype::Value},
    workflow::{
        action::TemplateAction,
        activity::Terminality,
        postprocessing::Postprocessing,
        settings::{ProposeSettings, TemplateSettings},
        template::Template,
        types::ObjectMetadata,
    },
    FnCallId, MethodName,
};

// TODO: Refactoring.

impl Contract {
    #[inline]
    pub fn init_dao_settings(&mut self, settings: Settings) {
        self.settings.set(&settings.into());
    }

    #[inline]
    pub fn init_tags(&mut self, tags: Vec<TagInput>) {
        self.tags.insert(&"group".into(), &Tags::new());
        self.tags.insert(&"global".into(), &Tags::new());

        for i in tags.into_iter() {
            let mut tags = Tags::new();
            tags.insert(i.values);
            self.tags.insert(&i.category, &tags);
        }
    }

    #[inline]
    pub fn init_groups(&mut self, groups: Vec<GroupInput>) {
        for g in groups.into_iter() {
            self.group_add(g);
        }

        assert!(
            self.ft_total_supply >= self.ft_total_locked,
            "{}",
            ERR_LOCK_AMOUNT_OVERFLOW
        );
    }

    /// Register fncalls and their metadata
    /// when dao is being created.
    /// Existing are overwriten.
    /// No checks included.
    pub fn init_function_calls(
        &mut self,
        calls: Vec<FnCallId>,
        metadata: Vec<Vec<ObjectMetadata>>,
    ) {
        for (i, c) in calls.iter().enumerate() {
            self.function_call_metadata.insert(c, &metadata[i]);
        }
    }

    /// Version of `init_function_calls` method but for standard interfaces.
    pub fn init_standard_function_calls(
        &mut self,
        calls: Vec<MethodName>,
        metadata: Vec<Vec<ObjectMetadata>>,
    ) {
        for (i, c) in calls.iter().enumerate() {
            self.standard_function_call_metadata.insert(c, &metadata[i]);
        }
    }

    /// Add provided workflow templates with its template settings
    /// when dao is being created.
    #[inline]
    pub fn init_workflows(
        &mut self,
        mut workflows: Vec<Template>,
        mut workflow_template_settings: Vec<Vec<TemplateSettings>>,
    ) {
        assert!(workflows.len() > 0);
        assert!(
            workflows.get(0).unwrap().code == "wf_add",
            "{}",
            "First workflow must be WfAdd (code: wf_add)"
        );

        let len = workflows.len();
        for i in 0..len {
            self.workflow_template.insert(
                &((len - i) as u16),
                &(
                    workflows.pop().unwrap(),
                    workflow_template_settings.pop().unwrap(),
                ),
            );
        }

        self.workflow_last_id += len as u16;
    }
    #[inline]
    pub fn init_treasury_partitions(&mut self, partitions: Vec<TreasuryPartitionInput>) {
        for partition in partitions {
            let treasury_partititon = TreasuryPartition::try_from(partition)
                .expect("Invalid TreasuryPartitionInput object.");
            self.partition_add(treasury_partititon);
        }
    }

    pub fn get_workflow_and_proposal(&self, proposal_id: u32) -> ProposalWf {
        let proposal = Proposal::from(self.proposals.get(&proposal_id).expect("Unknown proposal"));
        let (wft, mut wfs) = self.workflow_template.get(&proposal.workflow_id).unwrap();
        let settings = wfs.swap_remove(proposal.workflow_settings_id as usize);

        (proposal, wft, settings)
    }

    /// Add new storage bucket to the storage.
    /// Panic if storage bucket already exists.
    pub fn storage_bucket_add(&mut self, bucket_id: &str) {
        let bucket = StorageBucket::new(utils::get_bucket_id(bucket_id));
        require!(
            self.storage
                .insert(&bucket_id.to_owned(), &bucket)
                .is_none(),
            "Storage bucket already exists.",
        );
    }

    /// Closure which might be required in workflow.
    /// Returns DAO's specific values which cannot be known ahead of time.
    pub fn dao_consts(&self) -> impl Consts {
        DaoConsts::default()
    }

    /// Action logging method.
    /// Will be moved to indexer when its ready.
    pub fn log_action(
        &mut self,
        proposal_id: ProposalId,
        caller: &AccountId,
        action_id: u8,
        args: &[Vec<Value>],
        args_collections: Option<&[Vec<Value>]>,
    ) {
        let mut logs = self
            .workflow_activity_log
            .get(&proposal_id)
            .unwrap_or_else(|| Vec::with_capacity(1));

        logs.push(ActivityLog {
            caller: caller.to_owned(),
            action_id,
            timestamp: env::block_timestamp() / 10u64.pow(9),
            args: args.to_vec(),
            args_collections: args_collections.map(|a| a.to_vec()),
        });

        self.workflow_activity_log.insert(&proposal_id, &logs);
    }
}

pub mod utils {
    use library::functions::utils::{
        into_storage_key_wrapper_str, into_storage_key_wrapper_u16, StorageKeyWrapper,
    };
    use near_sdk::env;

    use crate::{
        constants::{GROUP_RELEASE_PREFIX, STORAGE_BUCKET_PREFIX},
        GroupId, TimestampSec,
    };

    pub fn get_group_key(id: GroupId) -> StorageKeyWrapper {
        into_storage_key_wrapper_u16(GROUP_RELEASE_PREFIX, id)
    }

    pub fn get_bucket_id(id: &str) -> StorageKeyWrapper {
        into_storage_key_wrapper_str(STORAGE_BUCKET_PREFIX, id)
    }

    pub fn current_timestamp_sec() -> TimestampSec {
        env::block_timestamp() / 10u64.pow(9)
    }
}

#[derive(Default)]
pub struct DaoConsts;
impl Consts for DaoConsts {
    fn get(&self, key: u8) -> Option<Value> {
        match key {
            C_DAO_ID => Some(Value::String(env::current_account_id().to_string())),
            C_CURRENT_TIMESTAMP_SECS => Some(Value::U64(current_timestamp_sec())),
            C_PREDECESSOR => Some(Value::String(env::predecessor_account_id().to_string())),
            _ => unimplemented!(),
        }
    }
}

// TODO: Remove Debug in production.
/// Helper struct used during activity execution.
#[derive(Debug)]
pub struct ActivityContext {
    pub caller: AccountId,
    pub proposal_id: u32,
    pub activity_id: usize,
    pub attached_deposit: Balance,
    pub proposal_settings: ProposeSettings,
    pub actions_done_before: u8,
    pub actions_done_now: u8,
    pub activity_postprocessing: Option<Postprocessing>,
    pub terminal: Terminality,
    pub actions: Vec<TemplateAction>,
    pub optional_actions: u8,
}

impl ActivityContext {
    pub fn new(
        proposal_id: u32,
        activity_id: usize,
        caller: AccountId,
        attached_deposit: Balance,
        proposal_settings: ProposeSettings,
        actions_done: u8,
        activity_postprocessing: Option<Postprocessing>,
        terminal: Terminality,
        actions: Vec<TemplateAction>,
    ) -> Self {
        Self {
            caller,
            proposal_id,
            activity_id,
            attached_deposit,
            proposal_settings,
            actions_done_before: actions_done,
            actions_done_now: actions_done,
            activity_postprocessing,
            terminal,
            actions,
            optional_actions: 0,
        }
    }

    /// Actions done during activity execution.
    /// In case of FnCall its count of dispatched promises.
    pub fn actions_done(&self) -> u8 {
        self.actions_done_now - self.actions_done_before
    }

    pub fn set_next_action_done(&mut self) {
        self.actions_done_now += 1;
    }

    pub fn set_next_optional_action_done(&mut self) {
        self.optional_actions += 1;
    }

    pub fn actions_count(&self) -> u8 {
        self.actions.len() as u8
    }

    pub fn activity_autofinish(&self) -> bool {
        self.terminal == Terminality::Automatic
    }

    pub fn all_actions_done(&self) -> bool {
        self.actions_done_now == self.actions_count()
    }
}
