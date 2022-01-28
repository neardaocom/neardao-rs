use near_sdk::{ext_contract, near_bindgen, env, PromiseResult};

use crate::core::*;
use crate::errors::*;

#[ext_contract(ext_self)]
trait ExtSelf {
    fn callback_insert_ref_pool(&mut self) -> u32;
    fn callback_insert_skyward_auction(&mut self) -> u64;
}

/*
#[near_bindgen]
impl DaoContract {
    #[private]
    pub fn callback_insert_ref_pool(&mut self) -> u32 {
        assert_eq!(env::promise_results_count(), 1, "{}", ERR_PROMISE_INVALID_RESULTS_COUNT);
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pool_id) = near_sdk::serde_json::from_slice::<u32>(&val) {
                    let mut pools = self.ref_pools.get().unwrap();
                    pools.push(pool_id);
                    self.ref_pools.set(&pools);
                    pool_id
                } else {
                    env::panic(ERR_PROMISE_INVALID_VALUE.as_bytes())
                }
            }
            PromiseResult::Failed => env::panic(ERR_PROMISE_FAILED.as_bytes()),
        }
    }

    #[private]
    pub fn callback_insert_skyward_auction(&mut self) -> u64 {
        assert_eq!(env::promise_results_count(), 1, "{}", ERR_PROMISE_INVALID_RESULTS_COUNT);
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(auction_id) = near_sdk::serde_json::from_slice::<u64>(&val) {
                    let mut auctions = self.skyward_auctions.get().unwrap();
                    auctions.push(auction_id);
                    self.skyward_auctions.set(&auctions);
                    auction_id
                } else {
                    env::panic(ERR_PROMISE_INVALID_VALUE.as_bytes())
                }
            }
            PromiseResult::Failed => env::panic(ERR_PROMISE_FAILED.as_bytes()),
        }
    }
}
*/