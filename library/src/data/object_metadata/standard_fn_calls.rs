//! Contains all standard calls defined as `Vec<ObjectMetadata>`.
//!
//! Currently provided standards:
//! - NEP-141 (FT)
//! - NEP-171 (NFT - Core)
//! - NEP-145 (Storage management)

use crate::{types::datatype::Datatype, workflow::types::ObjectMetadata};

pub fn nep_141_ft_transfer() -> Vec<ObjectMetadata> {
    vec![ObjectMetadata {
        arg_names: vec!["receiver_id".into(), "amount".into(), "memo".into()],
        arg_types: vec![
            Datatype::String(false),
            Datatype::U128(false),
            Datatype::String(true),
        ],
    }]
}

pub fn nep_141_ft_transfer_call() -> Vec<ObjectMetadata> {
    vec![ObjectMetadata {
        arg_names: vec![
            "receiver_id".into(),
            "amount".into(),
            "memo".into(),
            "msg".into(),
        ],
        arg_types: vec![
            Datatype::String(false),
            Datatype::U128(false),
            Datatype::String(true),
            Datatype::String(false),
        ],
    }]
}

pub fn nep_171_nft_transfer() -> Vec<ObjectMetadata> {
    vec![ObjectMetadata {
        arg_names: vec![
            "receiver_id".into(),
            "token_id".into(),
            "approval_id".into(),
            "memo".into(),
        ],
        arg_types: vec![
            Datatype::String(false),
            Datatype::String(false),
            Datatype::U64(true),
            Datatype::String(true),
        ],
    }]
}

pub fn nep_171_nft_transfer_call() -> Vec<ObjectMetadata> {
    vec![ObjectMetadata {
        arg_names: vec![
            "receiver_id".into(),
            "token_id".into(),
            "approval_id".into(),
            "memo".into(),
            "msg".into(),
        ],
        arg_types: vec![
            Datatype::String(false),
            Datatype::String(false),
            Datatype::U64(true),
            Datatype::String(true),
            Datatype::String(false),
        ],
    }]
}

pub fn nep_145_storage_deposit() -> Vec<ObjectMetadata> {
    vec![ObjectMetadata {
        arg_names: vec!["account_id".into(), "registration_only".into()],
        arg_types: vec![Datatype::String(false), Datatype::Bool(true)],
    }]
}

pub fn nep_145_storage_withdraw() -> Vec<ObjectMetadata> {
    vec![ObjectMetadata {
        arg_names: vec!["amount".into()],
        arg_types: vec![Datatype::U128(true)],
    }]
}

pub fn nep_145_storage_unregister() -> Vec<ObjectMetadata> {
    vec![ObjectMetadata {
        arg_names: vec!["force".into()],
        arg_types: vec![Datatype::String(false)],
    }]
}
