#![allow(unreachable_patterns)]

use near_sdk::{env::sha256, CryptoHash};

mod standard_impl;

pub mod constants;
pub mod errors;
pub mod internal;
pub mod core;
pub mod callbacks;
pub mod view;

pub mod action;
pub mod config;
pub mod file;
pub mod proposal;
pub mod release;
pub mod vote_policy;

mod unit_tests;

pub(crate) type UUID = CryptoHash;
pub(crate) type CID = String; // IPFS address

pub fn generate_uuid(input: &[u8]) -> UUID {
    let mut tmp = [0u8; 32];
    tmp.copy_from_slice(&sha256(input));
    tmp
}

/// Calculates votes as percents
#[inline]
pub fn calc_percent_u128_unchecked(value: u128, total: u128, decimals: u128) -> u8 {
    ((value / decimals) as f64 / (total / decimals) as f64 * 100.0).round() as u8
}