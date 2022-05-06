//! Contains all standard calls defined as `Vec<ObjectMetadata>`.
//!
//! Currently provided standards:
//! - NEP-141 (FT)
//! - NEP-171 (NFT - Core)
//! - NEP-145 (Storage management)

use crate::{types::datatype::Datatype, workflow::types::ObjectMetadata, MethodName};

// TODO: Could be solved with a macro.
pub const NEP_141_FT_TRANSFER: &str = "ft_transfer";
pub const NEP_141_FT_TRANSFER_CALL: &str = "ft_transfer_call";
pub const NEP_171_NFT_TRANSFER: &str = "nft_transfer";
pub const NEP_171_NFT_TRANSFER_CALL: &str = "nft_transfer_call";
pub const NEP_145_STORAGE_DEPOSIT: &str = "storage_deposit";
pub const NEP_145_STORAGE_WITHDRAW: &str = "storage_withdraw";
pub const NEP_145_STORAGE_UNREGISTER: &str = "storage_unregister";

// TODO: Could be solved with a macro.
pub fn standard_fn_call_methods() -> Vec<MethodName> {
    vec![
        NEP_141_FT_TRANSFER.into(),
        NEP_141_FT_TRANSFER_CALL.into(),
        NEP_171_NFT_TRANSFER.into(),
        NEP_171_NFT_TRANSFER_CALL.into(),
        NEP_145_STORAGE_DEPOSIT.into(),
        NEP_145_STORAGE_WITHDRAW.into(),
        NEP_145_STORAGE_UNREGISTER.into(),
    ]
}

// TODO: Could be solved with a macro.
pub fn standard_fn_call_metadatas() -> Vec<Vec<ObjectMetadata>> {
    vec![
        nep_141_ft_transfer(),
        nep_141_ft_transfer_call(),
        nep_171_nft_transfer(),
        nep_171_nft_transfer_call(),
        nep_145_storage_deposit(),
        nep_145_storage_withdraw(),
        nep_145_storage_unregister(),
    ]
}

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
