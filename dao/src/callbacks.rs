use library::types::FnCallMetadata;
use library::workflow::{ActionResult, Template, TemplateSettings};
use library::FnCallId;
use near_sdk::serde_json;
use near_sdk::{env, ext_contract, near_bindgen, PromiseResult};

use crate::core::*;
use crate::errors::*;
use library::{types::DataType, workflow::Postprocessing};

#[ext_contract(ext_self)]
trait ExtSelf {
    fn postprocess(
        &mut self,
        instance_id: u32,
        storage_key: String,
        postprocessing: Option<Postprocessing>,
        inner_value: Option<DataType>,
        must_succeed: bool,
    ) -> ActionResult;

    fn store_workflow(&mut self, instance_id: u32, settings: Vec<TemplateSettings>)
        -> ActionResult;
}

#[near_bindgen]
impl Contract {
    // TODO finish error handling
    #[private]
    /// Private callback to check Promise result.
    /// If there is postprocessing, then also processes result or just save provided value to the storage.
    pub fn postprocess(
        &mut self,
        instance_id: u32,
        storage_key: String,
        postprocessing: Option<Postprocessing>,
        inner_value: Option<DataType>,
        must_succeed: bool,
    ) -> ActionResult {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        let result: bool = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => match postprocessing {
                Some(p) => {
                    let mut bucket = self.storage.get(&storage_key).unwrap();

                    let key = p.storage_key.clone();
                    if let Some(val) = &p.postprocess(val, inner_value, &mut bucket) {
                        bucket.add_data(&key, val);
                    }

                    self.storage.insert(&storage_key, &bucket);
                    true
                }
                None => true,
            },
            PromiseResult::Failed => false,
        };

        match result {
            true => ActionResult::Ok,
            false => self.postprocessing_fail_update(instance_id, must_succeed),
        }
    }

    #[private]
    pub fn store_workflow(
        &mut self,
        instance_id: u32,
        settings: Vec<TemplateSettings>,
    ) -> ActionResult {
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
                ActionResult::Ok
            }
            PromiseResult::Failed => self.postprocessing_fail_update(instance_id, true),
        }
    }
}
