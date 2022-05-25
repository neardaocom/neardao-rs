#![allow(unreachable_patterns)]

use library::workflow::{
    instance::Instance,
    settings::{ProposeSettings, TemplateSettings},
    template::Template,
};
use proposal::Proposal;
mod unit_tests;

pub mod constants;
pub mod tags;

pub mod delegation;
pub mod group;
pub mod internal;
pub mod media;
pub mod proposal;
pub mod role;
pub mod settings;
pub mod workflow;

pub mod core;
pub mod receiver;
pub mod reward;
pub mod treasury;
pub mod view;
pub mod wallet;

/// Timestamp in seconds.
pub(crate) type TimestampSec = u64;
pub(crate) type ProposalId = u32;
pub(crate) type TagId = u16;
pub(crate) type StorageKey = String;
pub(crate) type TagCategory = String;
/// GroupId = 0 is reserved for "guest" role.
pub(crate) type GroupId = u16;
pub(crate) type VoteTotalPossible = u128;
pub(crate) type Votes = [u128; 3];
pub(crate) type CalculatedVoteResults = (VoteTotalPossible, Votes);
pub(crate) type ProposalWf = (Proposal, Template, TemplateSettings);
#[allow(dead_code)]
pub(crate) type InstanceWf = (Instance, ProposeSettings);
pub(crate) type RoleId = u16;
pub(crate) type RewardId = u16;
/// Id of the resource on the resource provider.
pub(crate) type ResourceId = u32;
pub type TokenId = String;
pub type ApprovalId = Option<u64>;

#[macro_export]
macro_rules! derive_into_versioned {
    ($from:ident, $for:ident) => {
        impl From<$from> for $for {
            fn from(input: $from) -> Self {
                $for::Current(input)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_from_versioned {
    ($from:ident, $for:ident) => {
        impl From<$from> for $for {
            fn from(input: $from) -> Self {
                match input {
                    $from::Current(c) => c,
                    _ => unreachable!(),
                }
            }
        }
    };
}
