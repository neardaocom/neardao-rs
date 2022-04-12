use near_sdk::serde_json;

use crate::{
    storage::StorageBucket,
    types::{
        error::{ProcessingError, SourceError},
        DataType, DataTypeDef,
    },
    workflow::{
        expression::Expression,
        types::{ArgSrc, FnCallMetadata, ValidatorRef, ValidatorType, ValueContainer},
    },
    Consts,
};

/// Validates inputs by validator_exprs.
/// Returns `Err(())` in case input/Wf structure is bad.
pub fn validate<T: std::convert::AsRef<[DataType]>>(
    sources: &ValueContainer<T>,
    validator_refs: &[ValidatorRef],
    validator_exprs: &[Expression],
    metadata: &[FnCallMetadata],
    user_input: &[Vec<DataType>],
) -> Result<bool, ProcessingError> {
    for v_ref in validator_refs.iter() {
        let validator = validator_exprs
            .get(v_ref.val_id as usize)
            .ok_or_else(|| SourceError::InvalidArgId)?;
        let inputs: &[DataType] = &user_input[v_ref.obj_id as usize];

        match v_ref.v_type {
            ValidatorType::Simple => {
                let inputs: &[DataType] = &user_input
                    .get(v_ref.obj_id as usize)
                    .ok_or_else(|| ProcessingError::UserInput(v_ref.obj_id))?;
                if !validator.bind_and_eval(sources, inputs)?.try_into_bool()? {
                    return Ok(false);
                }
            }
            //TODO validate by pos??
            ValidatorType::Collection => {
                let obj_len = metadata
                    .get(v_ref.obj_id as usize)
                    .ok_or_else(|| SourceError::InvalidArgId)?
                    .arg_names
                    .len();
                let collection_size_total = inputs.len();

                // collection number of values must be multiple of collection object argument count
                if collection_size_total % obj_len != 0 {
                    return Err(ProcessingError::UserInput(v_ref.obj_id));
                }

                // apply validator for each obj in collection
                for (i, _) in inputs.iter().enumerate().step_by(obj_len) {
                    if !validator
                        .bind_and_eval(sources, &inputs[i..i + obj_len])?
                        .try_into_bool()?
                    {
                        return Ok(false);
                    }
                }
            }
        }
    }

    Ok(true)
}

/// Binds values from template's sources/storage to replace those in `user_input`.
/// Schema is defined by `source_metadata` values.
/// Returns `Err(())` in case input/Wf structure is bad.
pub fn bind_from_sources<T: std::convert::AsRef<[DataType]>>(
    source_metadata: &[Vec<ArgSrc>],
    sources: &ValueContainer<T>,
    expressions: &[Expression],
    mut user_input: &mut Vec<Vec<DataType>>,
    metadata_pos: usize,
) -> Result<(), ProcessingError> {
    let mut result_args = Vec::with_capacity(
        source_metadata
            .get(metadata_pos)
            .ok_or_else(|| SourceError::InvalidArgId)?
            .len(),
    );

    for arg_type in source_metadata[metadata_pos].iter() {
        match arg_type {
            ArgSrc::User(arg_pos) => {
                // Way to check index exists so it does not panics at next step.
                let _ = user_input
                    .get(metadata_pos)
                    .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
                    .get(*arg_pos as usize)
                    .ok_or_else(|| ProcessingError::UserInput(*arg_pos))?;

                result_args.push(std::mem::replace(
                    &mut user_input[metadata_pos][*arg_pos as usize],
                    DataType::Null,
                ))
            }

            ArgSrc::Expression(expr_id) => result_args.push(
                expressions
                    .get(*expr_id as usize)
                    .ok_or_else(|| SourceError::InvalidArgId)?
                    .bind_and_eval(
                        &sources,
                        user_input
                            .get(metadata_pos)
                            .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
                            .as_slice(),
                    )?,
            ),
            ArgSrc::Object(id) => {
                result_args.push(DataType::Null);

                bind_from_sources(
                    source_metadata,
                    sources,
                    expressions,
                    &mut user_input,
                    *id as usize,
                )?;
            }
            ArgSrc::VecObject(id) => {
                result_args.push(DataType::Null);

                bind_vec_obj_args(
                    source_metadata,
                    sources,
                    expressions,
                    &mut user_input,
                    *id as usize,
                )?;
            }
            _ => result_args.push(get_value_from_source(arg_type, &sources)?),
        }
    }

    std::mem::swap(&mut result_args, &mut user_input[metadata_pos]);

    Ok(())
}

pub(crate) fn bind_vec_obj_args<T: std::convert::AsRef<[DataType]>>(
    source_metadata: &[Vec<ArgSrc>],
    sources: &ValueContainer<T>,
    expressions: &[Expression],
    mut user_input: &mut Vec<Vec<DataType>>,
    metadata_pos: usize,
) -> Result<(), ProcessingError> {
    let mut result_args = Vec::with_capacity(source_metadata.len());
    let mut obj_arg_pos = 0;
    let mut cycle_counter = 0;
    let obj_size = source_metadata
        .get(metadata_pos)
        .ok_or_else(|| SourceError::InvalidArgId)?
        .len();

    for _ in 0..user_input
        .get(metadata_pos)
        .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
        .len()
    {
        let arg_src = source_metadata
            .get(metadata_pos)
            .ok_or_else(|| SourceError::InvalidArgId)?
            .get(obj_arg_pos)
            .ok_or_else(|| SourceError::InvalidArgId)?;

        match arg_src {
            ArgSrc::User(arg_pos) => {
                let _ = user_input
                    .get(metadata_pos)
                    .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
                    .get(*arg_pos as usize + cycle_counter * obj_size)
                    .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?;

                result_args.push(std::mem::replace(
                    &mut user_input[metadata_pos][*arg_pos as usize + cycle_counter * obj_size],
                    DataType::Null,
                ))
            }
            ArgSrc::Expression(expr_id) => result_args.push(
                expressions
                    .get(*expr_id as usize)
                    .ok_or_else(|| SourceError::InvalidArgId)?
                    .bind_and_eval(
                        &sources,
                        user_input
                            .get(metadata_pos)
                            .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
                            .as_slice(),
                    )?,
            ),
            // VecObject can have object only as another VecObject
            ArgSrc::Object(_) => Err(ProcessingError::Unreachable)?,
            ArgSrc::VecObject(id) => {
                result_args.push(DataType::Null);

                bind_vec_obj_args(
                    source_metadata,
                    sources,
                    expressions,
                    &mut user_input,
                    *id as usize,
                )?;
            }
            _ => result_args.push(get_value_from_source(
                *&source_metadata
                    .get(metadata_pos)
                    .ok_or_else(|| SourceError::InvalidArgId)?
                    .get(obj_arg_pos)
                    .ok_or_else(|| SourceError::InvalidArgId)?,
                sources,
            )?),
        }

        // reset pos
        if obj_arg_pos == obj_size - 1 {
            obj_arg_pos = 0;
            cycle_counter += 1;
        } else {
            obj_arg_pos += 1;
        }
    }

    std::mem::swap(&mut result_args, &mut user_input[metadata_pos]);

    Ok(())
}

/// Fetch owned value from source defined by `arg_src`.
pub fn get_value_from_source<T: std::convert::AsRef<[DataType]>>(
    arg_src: &ArgSrc,
    container: &ValueContainer<T>,
) -> Result<DataType, SourceError> {
    match arg_src {
        ArgSrc::ConstsTpl(id) => {
            let value = container
                .tpl_consts
                .as_ref()
                .get(*id as usize)
                .ok_or_else(|| SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::ConstsSettings(id) => {
            let value = container
                .settings_consts
                .as_ref()
                .get(*id as usize)
                .ok_or_else(|| SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::ConstAction(id) => {
            let value = container
                .action_proposal_consts
                .ok_or_else(|| SourceError::SourceMissing)?
                .as_ref()
                .get(*id as usize)
                .ok_or_else(|| SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::ConstActivityShared(id) => {
            let value = container
                .activity_shared_consts
                .ok_or_else(|| SourceError::SourceMissing)?
                .as_ref()
                .get(*id as usize)
                .ok_or_else(|| SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::Storage(key) => {
            let value = container
                .storage
                .as_ref()
                .ok_or_else(|| SourceError::SourceMissing)?
                .get_data(&key)
                .ok_or_else(|| SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::GlobalStorage(key) => {
            let value = container
                .global_storage
                .get_data(&key)
                .ok_or_else(|| SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::Const(const_id) => {
            Ok((container.dao_consts)(*const_id).ok_or_else(|| SourceError::InvalidArgId)?)
        }
        _ => Err(SourceError::InvalidSourceVariant),
    }
}

/// Serializes JSON string by metadata schema.
pub fn serialize_to_json(
    user_input: &[Vec<DataType>],
    metadata: &[FnCallMetadata],
    metadata_id: usize,
) -> String {
    // Create raw json object string
    let mut args = String::with_capacity(128);
    args.push('{');
    for i in 0..metadata[metadata_id].arg_names.len() {
        args.push('"');
        args.push_str(metadata[metadata_id].arg_names[i].as_str()); //json attribute
        args.push('"');
        args.push(':');
        match &metadata[metadata_id].arg_types[i] {
            DataTypeDef::Object(id) => {
                args.push_str(serialize_to_json(user_input, metadata, *id as usize).as_str());
            }
            DataTypeDef::NullableObject(id) => {
                // check first elem in the obj array
                // if theres optional attribute with null on the 0th position, then this wont work
                if &user_input[*id as usize][0] == &DataType::Null {
                    args.push_str("null");
                } else {
                    args.push_str(serialize_to_json(user_input, metadata, *id as usize).as_str());
                }
            }
            DataTypeDef::VecObject(id) => {
                colection_to_json(&mut args, &user_input, metadata, *id as usize);
            }
            _ => primitive_arg_to_json(
                &mut args,
                &metadata[metadata_id].arg_types[i],
                &user_input[metadata_id][i],
            ),
        }
        args.push(',');
    }
    args.pop();
    args.push('}');
    args
}

/// Serializes collection of objects to JSON.
pub(crate) fn colection_to_json(
    buf: &mut String,
    user_input: &[Vec<DataType>],
    metadata: &[FnCallMetadata],
    metadata_id: usize,
) {
    let obj_size = metadata[metadata_id].arg_types.len();
    buf.push('[');
    if user_input[metadata_id].len() > 0 {
        for (i, _) in user_input[metadata_id].iter().enumerate().step_by(obj_size) {
            buf.push_str("{");
            for j in 0..obj_size {
                buf.push('"');
                buf.push_str(&metadata[metadata_id].arg_names[j]);
                buf.push('"');
                buf.push(':');
                match &metadata[metadata_id].arg_types[j] {
                    // Cannot use defaul obj serialization coz structure is different. Each Object in VecObj is technically VecObj.
                    DataTypeDef::VecObject(id) => {
                        collection_obj_to_json(
                            buf,
                            user_input,
                            metadata,
                            *id as usize,
                            i / obj_size, // Need to know current object pos.
                        );
                    }
                    _ => primitive_arg_to_json(
                        buf,
                        &metadata[metadata_id].arg_types[j],
                        &user_input[metadata_id][j + i],
                    ),
                }
                buf.push(',');
            }
            buf.pop();
            buf.push_str("}");
            buf.push(',');
        }
        buf.pop();
    }
    buf.push(']');
}

/// Serializes collection of object to JSON.
/// Because of schema this has to be separate method.
pub(crate) fn collection_obj_to_json(
    buf: &mut String,
    user_input: &[Vec<DataType>],
    metadata: &[FnCallMetadata],
    metadata_id: usize,
    pos: usize,
) {
    buf.push('{');
    let mut counter = 0;
    let obj_size = metadata[metadata_id].arg_names.len();
    for i in 0 + pos * obj_size..pos * obj_size + obj_size {
        buf.push('"');
        buf.push_str(metadata[metadata_id].arg_names[counter].as_str());
        buf.push('"');
        buf.push(':');
        match &metadata[metadata_id].arg_types[counter] {
            DataTypeDef::VecObject(id) => {
                collection_obj_to_json(buf, user_input, metadata, *id as usize, pos);
            }
            _ => primitive_arg_to_json(
                buf,
                &metadata[metadata_id].arg_types[counter],
                &user_input[metadata_id][i],
            ),
        }
        buf.push(',');
        counter += 1;
    }
    buf.pop();
    buf.push('}');
}

/// Creates JSON representation of value by data_type_def and pushes it to the buffer
pub(crate) fn primitive_arg_to_json(
    buf: &mut String,
    datatype_defs: &DataTypeDef,
    value: &DataType,
) {
    match datatype_defs {
        DataTypeDef::String(opt) => match (opt, value) {
            (true, DataType::String(v)) | (false, DataType::String(v)) => {
                buf.push('"');
                buf.push_str(v.as_str());
                buf.push('"');
            }
            (true, DataType::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during serializing json"),
        },

        DataTypeDef::Bool(opt) => match (opt, value) {
            (true, DataType::Bool(v)) | (false, DataType::Bool(v)) => match v {
                true => buf.push_str("true"),
                false => buf.push_str("false"),
            },
            (true, DataType::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during parsing"),
        },
        DataTypeDef::U64(opt) => match (opt, value) {
            (true, DataType::U64(v)) | (false, DataType::U64(v)) => {
                buf.push_str(&*v.to_string());
            }
            (true, DataType::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during parsing"),
        },
        DataTypeDef::U128(opt) => match (opt, value) {
            (true, DataType::U128(v)) | (false, DataType::U128(v)) => {
                buf.push('"');
                buf.push_str(&*v.0.to_string());
                buf.push('"');
            }
            (true, DataType::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during parsing"),
        },

        // Assuming no API expects something like Option<Vec<_>>, but instead Vec<_> is just empty
        DataTypeDef::VecString | DataTypeDef::VecU128 => {
            buf.push('[');

            match value {
                DataType::VecString(v) => {
                    for e in v.iter() {
                        buf.push('"');
                        buf.push_str(e);
                        buf.push('"');
                        buf.push(',');
                    }
                    buf.pop();
                }
                DataType::VecU128(v) => {
                    for e in v.iter() {
                        buf.push('"');
                        buf.push_str(&*e.0.to_string());
                        buf.push('"');
                        buf.push(',');
                    }
                    buf.pop();
                }
                DataType::VecU64(v) => {
                    for e in v.iter() {
                        buf.push_str(&*e.to_string());
                        buf.push(',');
                    }
                    buf.pop();
                }
                _ => unreachable!(),
            }
            buf.push(']');
        }
        _ => panic!("Invalid primitive type"),
    }
}

#[cfg(test)]
mod test {

    use std::convert::TryFrom;

    use near_sdk::{
        json_types::{ValidAccountId, WrappedBalance, U128},
        serde::{Deserialize, Serialize},
        serde_json,
    };

    use crate::{
        functions::{bind_from_sources, serialize_to_json, validate},
        interpreter::expression::{EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        storage::StorageBucket,
        types::{DataType, DataTypeDef},
        workflow::{
            expression::Expression,
            types::{ArgSrc, FnCallMetadata, ValidatorRef, ValidatorType, ValueContainer},
        },
    };

    /******  Skyward sale_create structures  ******/

    type BasicPoints = u16;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SaleCreateInput {
        pub sale: SaleInput,
        pub sale_info: Option<String>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SaleInput {
        pub title: String,
        pub url: Option<String>,
        pub permissions_contract_id: Option<ValidAccountId>,
        pub out_tokens: Vec<SaleInputOutToken>,
        pub in_token_account_id: ValidAccountId,
        pub start_time: U128,
        pub duration: U128,
        pub meta: MetaInfo,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct MetaInfo {
        reason: String,
        timestamp: u64,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SaleInputOutToken {
        pub token_account_id: ValidAccountId,
        pub balance: WrappedBalance,
        pub referral_bpt: Option<BasicPoints>,
        pub shares: ShareInfo,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct ShareInfo {
        user: String,
        amount: u32,
    }

    /****** TEST CASES ******/

    // TODO: Test with nullable object.

    #[test]
    fn full_scenario_validation_binding_serialization_complex_1() {
        let metadata = vec![
            FnCallMetadata {
                arg_names: vec!["sale".into(), "sale_info".into()],
                arg_types: vec![DataTypeDef::Object(1), DataTypeDef::String(true)],
            },
            FnCallMetadata {
                arg_names: vec![
                    "title".into(),
                    "url".into(),
                    "permissions_contract_id".into(),
                    "out_tokens".into(),
                    "in_token_account_id".into(),
                    "start_time".into(),
                    "duration".into(),
                    "meta".into(),
                ],
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::String(true),
                    DataTypeDef::String(true),
                    DataTypeDef::VecObject(3),
                    DataTypeDef::String(false),
                    DataTypeDef::U128(false),
                    DataTypeDef::U128(false),
                    DataTypeDef::Object(2),
                ],
            },
            FnCallMetadata {
                arg_names: vec!["reason".into(), "timestamp".into()],
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::U64(false)],
            },
            FnCallMetadata {
                arg_names: vec![
                    "token_account_id".into(),
                    "balance".into(),
                    "referral_bpt".into(),
                    "shares".into(),
                ],
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::U128(false),
                    DataTypeDef::U64(true),
                    DataTypeDef::VecObject(4),
                ],
            },
            FnCallMetadata {
                arg_names: vec!["user".into(), "amount".into()],
                arg_types: vec![DataTypeDef::String(false), DataTypeDef::U64(false)],
            },
        ];

        let tpl_consts = vec![
            DataType::String("neardao.testnet".into()),
            DataType::String("neardao.near".into()),
        ];
        let settings_consts = vec![DataType::U128(U128::from(1000))];
        let proposal_consts = vec![
            DataType::U64(500),
            DataType::String("info binded".into()),
            DataType::String("testing binded".into()),
        ];
        let expressions = vec![];
        let validator_refs = vec![
            ValidatorRef {
                v_type: ValidatorType::Simple,
                obj_id: 1,
                val_id: 0,
            },
            ValidatorRef {
                v_type: ValidatorType::Collection,
                obj_id: 3,
                val_id: 1,
            },
            ValidatorRef {
                v_type: ValidatorType::Collection,
                obj_id: 4,
                val_id: 2,
            },
        ];
        let validators = vec![
            // validates first obj
            Expression {
                args: vec![ArgSrc::ConstsTpl(0), ArgSrc::User(2)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Eqs),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
            // validates first collection obj
            Expression {
                args: vec![ArgSrc::ConstsSettings(0), ArgSrc::User(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
            // validates second collection obj
            Expression {
                args: vec![ArgSrc::ConstAction(0), ArgSrc::User(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
        ];

        let mut user_input = vec![
            vec![DataType::Null, DataType::String("test".into())],
            vec![
                DataType::String("Neardao token auction".into()),
                DataType::String("www.neardao.com".into()),
                DataType::String("neardao.testnet".into()),
                DataType::Null,
                DataType::String("wrap.near".into()),
                DataType::U128(0.into()),
                DataType::U128(3600.into()),
                DataType::Null,
            ],
            vec![DataType::String("testing".into()), DataType::U64(420)],
            vec![
                DataType::String("neardao.testnet".into()),
                DataType::U128(U128::from(999)), // 1000 is limit
                DataType::Null,
                DataType::Null,
                DataType::String("neardao.testnet".into()),
                DataType::U128(U128::from(991)), // 1000 is limit
                DataType::Null,
                DataType::Null,
                DataType::String("neardao.testnet".into()),
                DataType::U128(U128::from(991)), // 1000 is limit
                DataType::Null,
                DataType::Null,
            ],
            vec![
                DataType::String("petr.near".into()),
                DataType::U64(123), // 500 is limit
                DataType::String("david.near".into()),
                DataType::U64(456), // 500 is limit
                DataType::String("tomas.near".into()),
                DataType::U64(456), // 500 is limit
            ],
        ];

        let mut global_storage = StorageBucket::new(b"global".to_vec());
        let activity_shared_consts: Vec<DataType> = vec![];

        let dao_consts = Box::new(|id: u8| match id {
            0 => Some(DataType::String("neardao.near".into())),
            _ => None,
        });

        let value_source = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &tpl_consts,
            settings_consts: &settings_consts,
            activity_shared_consts: Some(&activity_shared_consts),
            action_proposal_consts: Some(&proposal_consts),
            storage: None,
            global_storage: &mut global_storage,
        };

        assert!(validate(
            &value_source,
            validator_refs.as_slice(),
            validators.as_slice(),
            metadata.as_slice(),
            user_input.as_slice(),
        )
        .expect("Validation failed."));

        /* ------------------ Binding ------------------ */

        let source_metadata = vec![
            vec![ArgSrc::Object(1), ArgSrc::ConstAction(1)],
            vec![
                ArgSrc::User(0),
                ArgSrc::User(1),
                ArgSrc::User(2),
                ArgSrc::VecObject(3),
                ArgSrc::User(4),
                ArgSrc::User(5),
                ArgSrc::User(6),
                ArgSrc::Object(2),
            ],
            vec![ArgSrc::ConstAction(2), ArgSrc::User(1)],
            vec![
                ArgSrc::ConstsTpl(1),
                ArgSrc::User(1),
                ArgSrc::User(2),
                ArgSrc::VecObject(4),
            ],
            vec![ArgSrc::User(0), ArgSrc::User(1)],
        ];

        bind_from_sources(
            &source_metadata,
            &value_source,
            &expressions,
            &mut user_input,
            0,
        )
        .expect("Binding failed");

        let expected_binded_inputs = vec![
            vec![DataType::Null, DataType::String("info binded".into())],
            vec![
                DataType::String("Neardao token auction".into()),
                DataType::String("www.neardao.com".into()),
                DataType::String("neardao.testnet".into()),
                DataType::Null,
                DataType::String("wrap.near".into()),
                DataType::U128(0.into()),
                DataType::U128(3600.into()),
                DataType::Null,
            ],
            vec![
                DataType::String("testing binded".into()),
                DataType::U64(420),
            ],
            vec![
                DataType::String("neardao.near".into()),
                DataType::U128(U128::from(999)), // 1000 is limit
                DataType::Null,
                DataType::Null,
                DataType::String("neardao.near".into()),
                DataType::U128(U128::from(991)), // 1000 is limit
                DataType::Null,
                DataType::Null,
                DataType::String("neardao.near".into()),
                DataType::U128(U128::from(991)), // 1000 is limit
                DataType::Null,
                DataType::Null,
            ],
            vec![
                DataType::String("petr.near".into()),
                DataType::U64(123), // 500 is limit
                DataType::String("david.near".into()),
                DataType::U64(456), // 500 is limit
                DataType::String("tomas.near".into()),
                DataType::U64(456), // 500 is limit
            ],
        ];

        assert_eq!(user_input, expected_binded_inputs);

        /* ------------------ Serializing to JSON ------------------ */

        let out_tokens_1 = SaleInputOutToken {
            token_account_id: ValidAccountId::try_from("neardao.near").unwrap(),
            balance: 999.into(),
            referral_bpt: None,
            shares: ShareInfo {
                user: "petr.near".into(),
                amount: 123,
            },
        };

        let out_tokens_2 = SaleInputOutToken {
            token_account_id: ValidAccountId::try_from("neardao.near").unwrap(),
            balance: 991.into(),
            referral_bpt: None,
            shares: ShareInfo {
                user: "david.near".into(),
                amount: 456,
            },
        };

        let out_tokens_3 = SaleInputOutToken {
            token_account_id: ValidAccountId::try_from("neardao.near").unwrap(),
            balance: 991.into(),
            referral_bpt: None,
            shares: ShareInfo {
                user: "tomas.near".into(),
                amount: 456,
            },
        };

        let sale_input = SaleInput {
            title: "Neardao token auction".into(),
            url: Some("www.neardao.com".into()),
            permissions_contract_id: Some(ValidAccountId::try_from("neardao.testnet").unwrap()),
            out_tokens: vec![out_tokens_1, out_tokens_2, out_tokens_3],
            in_token_account_id: ValidAccountId::try_from("wrap.near").unwrap(),
            start_time: 0.into(),
            duration: 3600.into(),
            meta: MetaInfo {
                reason: "testing binded".into(),
                timestamp: 420,
            },
        };

        let sale_create_input = SaleCreateInput {
            sale: sale_input,
            sale_info: Some("info binded".into()),
        };

        let result_json_string = serialize_to_json(user_input.as_slice(), metadata.as_slice(), 0);
        let expected_json_string = serde_json::to_string(&sale_create_input).unwrap();
        assert_eq!(result_json_string, expected_json_string);
        assert_eq!(
            serde_json::from_str::<SaleCreateInput>(&result_json_string).unwrap(),
            sale_create_input
        );
    }
}
