use std::ops::Add;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"),derive(Clone, Debug, PartialEq, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum ReleaseModelInput {
    None,
    Linear {
        from: Option<u32>, //None means from the time when dao was created
        duration: u32,
    },
}

#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"),derive(Clone, Debug, PartialEq))]
pub enum VReleaseModel {
    Curr(ReleaseModel)
}

#[derive(Serialize, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"),derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ReleaseModel {
    None,
    Linear { duration: u32, release_end: u32},
}

impl From<VReleaseModel> for ReleaseModel {
    fn from(release_model: VReleaseModel) -> Self {
        match release_model {
            VReleaseModel::Curr(m) => m,
            _ => unimplemented!(),
        }
    }
}

impl ReleaseModel {
    pub fn from_input(release_model: ReleaseModelInput, current_time: u32) -> Self {
        match release_model {
            ReleaseModelInput::None => ReleaseModel::None,
            ReleaseModelInput::Linear{ from, duration} => {

                let release_end = match from {
                    Some(t) => t.add(duration),
                    None => current_time.add(duration),
                };

                ReleaseModel::Linear {
                    duration, release_end
                }
            },
        }
    }

    /// Calculates amount of tokens to be released when called.
    /// No checks included.
    #[allow(unused)]
    pub fn release(&self, total: u32, init_distribution: u32, unlocked: u32, current_time: u32) -> u32 {
        match self {
            ReleaseModel::None => 0,
            ReleaseModel::Linear{duration, release_end} => 
            {
                if current_time >= *release_end {
                    return total - init_distribution;
                }
                ((total - init_distribution) as u64 * ( 100.0 - (*release_end as f64 - current_time as f64) * 100.0 / *duration as f64).round() as u64 / 100) as u32
            }
        }
    }
}


#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"),derive(Clone, Debug, PartialEq))]
pub enum VReleaseDb {
    Curr(ReleaseDb)
}

#[derive(Serialize,BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"),derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ReleaseDb {
    pub total: u32,
    pub unlocked: u32,
    pub init_distribution: u32,
    pub distributed: u32,
}

impl ReleaseDb {
    /// Assumption: total >= unlocked >= init_distribution !
    pub fn new(total: u32, unlocked: u32, init_distribution: u32) -> Self {
        ReleaseDb {
            total,
            unlocked,
            init_distribution,
            distributed: init_distribution,
        }
    }
}

impl From<VReleaseDb> for ReleaseDb {
    fn from(release_model: VReleaseDb) -> Self {
        match release_model {
            VReleaseDb::Curr(m) => m,
            _ => unimplemented!(),
        }
    }
}

//impl VReleaseModel {
//    pub fn get_release_amount(
//        &mut self,
//        current_time: u64,
//        total_supply: u32,
//        init_distribution: u32,
//        already_released: u32,
//    ) -> u32 {
//        match self {
//            Self::Voting => 0,
//            _ => unimplemented!()
//        }
//    }
//}
