#![allow(unreachable_patterns)]

use near_sdk::{env::sha256, CryptoHash};

mod standard_impl;

pub mod core;
pub mod view;

pub mod action;
pub mod config;
pub mod file;
pub mod proposal;
pub mod release;
pub mod vote_policy;

mod unit_tests;

pub(crate) const CID_MAX_LENGTH: u8 = 64; 

pub(crate) type UUID = CryptoHash;
pub(crate) type CID = String; // IPFS address

pub fn generate_uuid(input: &[u8]) -> UUID {
    let mut tmp = [0u8; 32];
    tmp.copy_from_slice(&sha256(input));
    tmp
}
