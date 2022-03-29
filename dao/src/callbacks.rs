use library::types::DataType;
use library::workflow::activity::Postprocessing;
use library::workflow::settings::TemplateSettings;
use library::workflow::template::Template;
use library::workflow::types::{ActivityResult, FnCallMetadata};
use library::FnCallId;
use near_sdk::serde_json;
use near_sdk::{env, ext_contract, near_bindgen, PromiseResult};

use crate::core::*;
use crate::error::*;

#[ext_contract(ext_self)]
trait ExtSelf {
    fn postprocess(
        &mut self,
        instance_id: u32,
        activity_id: u8,
        action_id: u8,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
        must_succeed: bool,
        wf_finish: bool,
    ) -> ActivityResult;

    /*     fn store_workflow(
        &mut self,
        instance_id: u32,
        settings: Vec<TemplateSettings>,
    ) -> ActivityResult; */
}

#[near_bindgen]
impl Contract {
    // TODO finish error handling
    #[private]
    /// Private callback to check Promise result.
    /// If there's postprocessing, then it's executed.
    /// Postprocessing always requires storage.
    /// Unwrapping is OK as it's been checked before dispatching this promise.
    pub fn postprocess(
        &mut self,
        instance_id: u32,
        activity_id: u8,
        action_id: u8,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
        must_succeed: bool,
        wf_finish: bool,
    ) -> Result<(), ActionError> {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => self.postprocessing_success(
                instance_id,
                activity_id,
                action_id,
                storage_key,
                postprocessing,
                wf_finish,
                val,
            ),
            PromiseResult::Failed => {
                self.postprocessing_failed(instance_id, activity_id, action_id, must_succeed)
            }
        }
    }
    /*
    #[private]
    pub fn store_workflow(
        &mut self,
        instance_id: u32,
        settings: Vec<TemplateSettings>,
    ) -> ActivityResult {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                let (workflow, fncalls, fncall_metadata): (
                    Template,
                    Vec<FnCallId>,
                    Vec<Vec<FnCallMetadata>>,
                ) = serde_json::from_slice(&val).unwrap();
                self.workflow_last_id += 1;
                self.workflow_template
                    .insert(&self.workflow_last_id, &(workflow, settings));
                self.init_function_calls(fncalls, fncall_metadata);
                ActivityResult::Ok
            }
            PromiseResult::Failed => self.postprocessing_failed(instance_id, true),
        }
    } */
}
