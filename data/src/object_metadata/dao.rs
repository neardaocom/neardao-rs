//! Metadata definition of objects for DAO actions.

use library::{
    types::datatype::Datatype,
    workflow::types::{DaoActionIdent, ObjectMetadata},
};

pub fn member_roles_metadata() -> ObjectMetadata {
    ObjectMetadata {
        arg_names: vec!["name".into(), "members".into()],
        arg_types: vec![Datatype::String(false), Datatype::VecString],
    }
}

pub fn group_member_metadata() -> ObjectMetadata {
    ObjectMetadata {
        arg_names: vec!["account_id".into(), "tags".into()],
        arg_types: vec![Datatype::String(false), Datatype::VecU64],
    }
}
pub fn asset_near() -> ObjectMetadata {
    ObjectMetadata {
        arg_names: vec!["near".into()],
        arg_types: vec![Datatype::String(true)],
    }
}

pub fn asset_ft() -> ObjectMetadata {
    ObjectMetadata {
        arg_names: vec!["ft.account_id".into(), "ft.decimals".into()],
        arg_types: vec![Datatype::String(false), Datatype::U64(false)],
    }
}

pub fn action_idents_with_metadata() -> Vec<(DaoActionIdent, Vec<ObjectMetadata>)> {
    vec![
        (
            DaoActionIdent::TreasuryAddPartition,
            vec![
                ObjectMetadata {
                    arg_names: vec!["name".into(), "assets".into()],
                    arg_types: vec![Datatype::String(false), Datatype::VecObject(1)],
                },
                ObjectMetadata {
                    arg_names: vec!["asset_id".into(), "unlocking".into()],
                    arg_types: vec![Datatype::Enum(vec![2, 3]), Datatype::Object(4)],
                },
                asset_near(),
                asset_ft(),
                ObjectMetadata {
                    arg_names: vec!["amount_init_unlock".into(), "lock".into()],
                    arg_types: vec![Datatype::U64(false), Datatype::OptionalObject(5)],
                },
                ObjectMetadata {
                    arg_names: vec![
                        "amount_total_lock".into(),
                        "start_from".into(),
                        "duration".into(),
                        "periods".into(),
                    ],
                    arg_types: vec![
                        Datatype::U64(false),
                        Datatype::U64(false),
                        Datatype::U64(false),
                        Datatype::VecObject(6),
                    ],
                },
                ObjectMetadata {
                    arg_names: vec!["type".into(), "duration".into(), "amount".into()],
                    arg_types: vec![
                        Datatype::String(false),
                        Datatype::U64(false),
                        Datatype::U64(false),
                    ],
                },
            ],
        ),
        (
            DaoActionIdent::RewardAdd,
            vec![
                ObjectMetadata {
                    arg_names: vec![
                        "group_id".into(),
                        "role_id".into(),
                        "partition_id".into(),
                        "type".into(),
                        "time_valid_from".into(),
                        "time_valid_to".into(),
                        "reward_amounts".into(),
                    ],
                    arg_types: vec![
                        Datatype::U64(false),
                        Datatype::U64(false),
                        Datatype::U64(false),
                        Datatype::Enum(vec![1, 2]),
                        Datatype::U64(false),
                        Datatype::U64(false),
                        Datatype::VecTuple(3),
                    ],
                },
                ObjectMetadata {
                    arg_names: vec!["wage.unit_seconds".into()],
                    arg_types: vec![Datatype::U64(false)],
                },
                ObjectMetadata {
                    arg_names: vec!["user_activity.activity_ids".into()],
                    arg_types: vec![Datatype::VecU64],
                },
                ObjectMetadata {
                    arg_names: vec![],
                    arg_types: vec![Datatype::Enum(vec![4, 5]), Datatype::U128(false)],
                },
                asset_near(),
                asset_ft(),
            ],
        ),
        (
            DaoActionIdent::GroupAdd,
            vec![
                ObjectMetadata {
                    arg_names: vec!["settings".into(), "members".into(), "member_roles".into()],
                    arg_types: vec![
                        Datatype::Object(1),
                        Datatype::VecObject(2),
                        Datatype::VecObject(3),
                    ],
                },
                ObjectMetadata {
                    arg_names: vec!["name".into(), "leader".into(), "parent_group".into()],
                    arg_types: vec![
                        Datatype::String(false),
                        Datatype::String(true),
                        Datatype::U64(false),
                    ],
                },
                group_member_metadata(),
                member_roles_metadata(),
            ],
        ),
        (
            DaoActionIdent::GroupAddMembers,
            vec![
                ObjectMetadata {
                    arg_names: vec!["id".into(), "members".into(), "member_roles".into()],
                    arg_types: vec![
                        Datatype::U64(false),
                        Datatype::VecObject(1),
                        Datatype::VecObject(2),
                    ],
                },
                group_member_metadata(),
                member_roles_metadata(),
            ],
        ),
        (
            DaoActionIdent::GroupRemoveMembers,
            vec![ObjectMetadata {
                arg_names: vec!["id".into(), "members".into()],
                arg_types: vec![Datatype::U64(false), Datatype::VecString],
            }],
        ),
        (
            DaoActionIdent::GroupRemoveRoles,
            vec![ObjectMetadata {
                arg_names: vec!["id".into(), "role_ids".into()],
                arg_types: vec![Datatype::U64(false), Datatype::VecString],
            }],
        ),
        (
            DaoActionIdent::GroupRemoveMemberRoles,
            vec![
                ObjectMetadata {
                    arg_names: vec!["id".into(), "member_roles".into()],
                    arg_types: vec![Datatype::U64(false), Datatype::VecObject(1)],
                },
                member_roles_metadata(),
            ],
        ),
    ]
}
