#![allow(unreachable_patterns)]
#![allow(clippy::too_many_arguments)] // TODO: Solve.

use library::workflow::{
    instance::Instance,
    settings::{ProposeSettings, TemplateSettings},
    template::Template,
};
use proposal::Proposal;
mod unit_tests;

pub mod constants;
pub mod error;
pub(crate) mod helper;
pub mod tags;

pub mod action;
pub mod callbacks;
pub mod delegation;
pub mod group;
pub mod internal;
pub mod media;
pub mod proposal;
pub mod settings;
pub mod token_lock;

pub mod core;
pub mod view;

pub(crate) type ProposalId = u32;
pub(crate) type TagId = u16;
pub(crate) type StorageKey = String;
pub(crate) type TagCategory = String;
pub(crate) type GroupId = u16;
pub(crate) type GroupName = String;
pub(crate) type VoteTotalPossible = u128;
pub(crate) type Votes = [u128; 3];
pub(crate) type CalculatedVoteResults = (VoteTotalPossible, Votes);
pub(crate) type ProposalWf = (Proposal, Template, TemplateSettings);
#[allow(dead_code)]
pub(crate) type InstanceWf = (Instance, ProposeSettings);

/// Calculates votes as percents
#[inline]
pub(crate) fn calc_percent_u128_unchecked(value: u128, total: u128, decimals: u128) -> u8 {
    ((value / decimals) as f64 / (total / decimals) as f64 * 100.0).round() as u8
}

#[macro_export]
macro_rules! derive_into_versioned {
    ($from:ident, $for:ident) => {
        impl From<$from> for $for {
            fn from(input: $from) -> Self {
                $for::Curr(input)
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
                    $from::Curr(c) => c,
                    _ => unreachable!(),
                }
            }
        }
    };
}
