use library::workflow::postprocessing::Postprocessing;
use near_sdk::{env, ext_contract, near_bindgen, PromiseResult};

use crate::core::*;
use crate::error::*;
#[ext_contract(ext_self)]
trait ExtSelf {
    #[allow(clippy::too_many_arguments)]
    fn postprocess(
        &mut self,
        instance_id: u32,
        action_id: u8,
        must_succeed: bool,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
    ) -> ActivityResult;
}

#[near_bindgen]
impl Contract {
    // TODO finish error handling
    /// Private callback to check Promise result.
    /// If there's postprocessing, then it's executed.
    /// Postprocessing always requires storage.
    /// Unwrapping is OK as it's been checked before dispatching this promise.
    #[allow(clippy::too_many_arguments)]
    #[private]
    pub fn postprocess(
        &mut self,
        instance_id: u32,
        action_id: u8,
        must_succeed: bool,
        storage_key: Option<String>,
        postprocessing: Option<Postprocessing>,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERR_PROMISE_INVALID_RESULTS_COUNT
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                self.debug_log.push(format!(
                    "promise log: SUCCESS instance_id: {}, action_id: {}; ",
                    instance_id, action_id
                ));
                self.postprocessing_success(
                    instance_id,
                    action_id,
                    storage_key,
                    postprocessing,
                    val,
                )
            }
            PromiseResult::Failed => {
                self.debug_log.push(format!(
                    "promise log: ERROR instance_id: {}, action_id: {}; ",
                    instance_id, action_id
                ));
                self.postprocessing_failed(instance_id, must_succeed)
            }
        }
    }
}
