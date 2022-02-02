#![allow(unreachable_patterns)]

use near_sdk::{env::sha256, CryptoHash};

mod standard_impl;

pub mod constants;
pub mod errors;
pub mod tags;

pub mod settings;
pub mod group;
pub mod media;
pub mod proposal;
pub mod release;
pub mod action;
pub mod callbacks;
pub mod internal;

pub mod core;
pub mod view;
//mod unit_tests;

pub(crate) type ProposalId = u32;
pub(crate) type TagId = u16;
/// Composite of "{receiver}_{function_call}" ident
pub(crate) type FnCallId = String;
pub(crate) type StorageKey = String;
pub(crate) type TagCategory = String;
pub(crate) type GroupId = u16; 
pub(crate) type GroupName = String;
pub(crate) type GroupStorageKey = u8;
pub(crate) type UUID = CryptoHash;
pub(crate) type CID = String; // IPFS address

/// Calculates votes as percents
#[inline]
pub fn calc_percent_u128_unchecked(value: u128, total: u128, decimals: u128) -> u8 {
    ((value / decimals) as f64 / (total / decimals) as f64 * 100.0).round() as u8
}

pub(crate) fn append(key: &[u8], suffix: &[u8]) -> Vec<u8> {
    [key, suffix].concat()
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
