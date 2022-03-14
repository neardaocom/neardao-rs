use std::ops::Range;

use near_sdk::serde_json;

use crate::{
    storage::StorageBucket,
    types::{DataType, DataTypeDef},
    workflow::{
        expression::Expression,
        types::{ArgSrc, FnCallMetadata, ValidatorRef, ValidatorType, ValueContainer},
    },
    Consts,
};
/*
/// Validates user action input with validators
pub fn validate_args(
    sources: &ValueContainer<&[DataType]>,
    obj_validator: &[ValidatorRef],
    validator_expr: &[Expression],
    metadata: &[FnCallMetadata],
) -> bool {
    let mut next_args_vec_idx: usize = 0;

    for (v_id, validator) in obj_validator.iter().enumerate() {
        let expr = &validator_expr[v_id];
        match validator.v_type {
            // if structure contains vec<obj> then we check for each vec<obj>
            // this way we can validate values for each obj in vec<obj>
            ValidatorType::Collection => {
                let current_metadata = &metadata[*id as usize];
                let params_len = current_metadata.arg_types.len();
                let collection = &args_collections[next_args_vec_idx];

                // nothing to do user did not provide any args
                if collection.len() == 0 {
                    continue;
                }

                // collection number of values must be multiple of collection object argument count
                assert_eq!(
                    collection.len() % params_len,
                    0,
                    "Invalid collection structure"
                );

                // apply validator for each obj params in collection
                for (i, _) in collection.iter().enumerate().step_by(params_len) {
                    if !expr
                        .bind_and_eval(consts, storage, binds, &collection[i..i + params_len])
                        .try_into_bool()
                        .unwrap()
                    {
                        return false;
                    }
                }
                // we advance object type in collection so position of collection objects is important
                next_args_vec_idx += 1;
            }

            ValidatorType::Simple => {
                // else just check that object
                if !expr
                    .bind_and_eval(consts, storage, binds, &args[*id as usize])
                    .try_into_bool()
                    .unwrap()
                {
                    return false;
                }
            }
        }
    }

    true
}
 */

// TODO beautify
/// Binds values by defined schema input_defs.
///
/*
pub fn bind_args(
    input_defs: &[Vec<ArgSrc>],
    sources: &ValueContainer<&[DataType]>,
    expressions: &[Expression],
    mut args: &mut Vec<Vec<DataType>>,
    mut args_collections: &mut Vec<Vec<DataType>>,
    metadata_pos: usize,
    mut next_collection_obj_idx: usize,
) {
    //init result structures
    let mut result_args = Vec::with_capacity(input_defs[metadata_pos].len());
    let mut result_args_collections: Vec<Vec<DataType>> =
    Vec::with_capacity(args_collections.len());
    for _ in 0..args_collections.len() {
        result_args_collections.push(vec![]);
    }

    for (arg_pos, arg_type) in input_defs[metadata_pos].iter().enumerate() {
        match arg_type {
            ArgSrc::User(_) => result_args.push(std::mem::replace(
                &mut args[metadata_pos][arg_pos],
                DataType::Null,
            )),
            ArgSrc::Expression(expr_id) => result_args.push(
                expressions[*expr_id as usize]
                .bind_and_eval(&sources, args[metadata_pos].as_slice()),
            ),
            ArgSrc::Object(id) => {
                result_args.push(DataType::Null);

                bind_args(
                    input_defs,
                    sources,
                    expressions,
                    &mut args,
                    &mut args_collections,
                    *id as usize - next_collection_obj_idx,
                    next_collection_obj_idx,
                );
            }
            ArgSrc::VecObject(id) => {
                bind_vec_obj_args(
                    input_defs[*id as usize].as_slice(),
                    sources,
                    &mut args_collections.get_mut(next_collection_obj_idx).unwrap(),
                );
                result_args.push(DataType::Null);
                next_collection_obj_idx += 1;
            }
            _ => result_args.push(get_value_from_source(arg_type, &sources)),
        }
    }

    std::mem::swap(&mut result_args, &mut args[metadata_pos]);
}
    */

/*
/// Parses arguments to output JSON string by metadata schema
pub fn args_to_json(
    arg_values: &[Vec<DataType>],
    arg_collection_values: &[Vec<DataType>],
    metadata: &[FnCallMetadata],
    metadata_id: usize,
) -> String {
    let mut next_collection_obj_idx: usize = 0;
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
                args.push_str(
                    args_to_json(arg_values, arg_collection_values, metadata, *id as usize)
                        .as_str(),
                );
            }
            DataTypeDef::NullableObject(id) => {
                // check first elem in the obj array
                // if theres optional attribute with null on the 0th position, then this wont work
                if std::mem::discriminant(&arg_values[*id as usize][0])
                    == std::mem::discriminant(&DataType::Null)
                {
                    args.push_str("null");
                } else {
                    args.push_str(
                        args_to_json(arg_values, arg_collection_values, metadata, *id as usize)
                            .as_str(),
                    );
                }
            }
            DataTypeDef::VecObject(id) => {
                args.push('[');

                let metadata_id = *id as usize;
                let params_len = &metadata[metadata_id].arg_types.len();
                let collection = &arg_collection_values[next_collection_obj_idx];

                // serialize each object in collection on pos next_collection_obj_idx
                for (i, _) in collection.iter().enumerate().step_by(*params_len) {
                    args.push_str("{");
                    for j in i..*params_len {
                        args.push('"');
                        args.push_str(&metadata[*id as usize].arg_names[j]);
                        args.push('"');
                        args.push(':');
                        primitive_arg_to_json(
                            &mut args,
                            &metadata[*id as usize].arg_types[j],
                            &collection[j],
                        );
                        args.push(',');
                    }
                    args.pop();
                    args.push_str("}");
                    args.push(',');
                }
                args.pop();
                args.push(']');
                next_collection_obj_idx += 1;
            }
            _ => primitive_arg_to_json(
                &mut args,
                &metadata[metadata_id].arg_types[i],
                &arg_values[metadata_id][i],
            ),
        }
        args.push(',');
    }
    args.pop();
    args.push('}');
    args
}

/// Creates JSON representation of value by data_type_def and pushes it to the buffer
pub(crate) fn primitive_arg_to_json(
    buffer: &mut String,
    data_type_def: &DataTypeDef,
    value: &DataType,
) {
    match data_type_def {
        DataTypeDef::String(opt) => match (opt, value) {
            (true, DataType::String(v)) | (false, DataType::String(v)) => {
                buffer.push('"');
                buffer.push_str(v.as_str());
                buffer.push('"');
            }
            (true, DataType::Null) => buffer.push_str("null"),
            _ => panic!("Invalid type during serializing json"),
        },

        DataTypeDef::Bool(opt) => match (opt, value) {
            (true, DataType::Bool(v)) | (false, DataType::Bool(v)) => match v {
                true => buffer.push_str("true"),
                false => buffer.push_str("false"),
            },
            (true, DataType::Null) => buffer.push_str("null"),
            _ => panic!("Invalid type during parsing"),
        },
        DataTypeDef::U8(opt) | DataTypeDef::U16(opt) | DataTypeDef::U32(opt) => {
            match (opt, value) {
                (true, DataType::U8(v)) | (false, DataType::U8(v)) => {
                    buffer.push_str(&*v.to_string());
                }
                (true, DataType::U16(v)) | (false, DataType::U16(v)) => {
                    buffer.push_str(&*v.to_string());
                }
                (true, DataType::U32(v)) | (false, DataType::U32(v)) => {
                    buffer.push_str(&*v.to_string());
                }
                (true, DataType::Null) => {
                    buffer.push_str("null");
                }
                _ => panic!("Invalid type during parsing"),
            }
        }
        DataTypeDef::U64(opt) | DataTypeDef::U128(opt) => match (opt, value) {
            (true, DataType::U64(v)) | (false, DataType::U64(v)) => {
                buffer.push('"');
                buffer.push_str(&*v.0.to_string());
                buffer.push('"');
            }
            (true, DataType::U128(v)) | (false, DataType::U128(v)) => {
                buffer.push('"');
                buffer.push_str(&*v.0.to_string());
                buffer.push('"');
            }
            (true, DataType::Null) => {
                buffer.push_str(serde_json::to_string(&DataType::Null).unwrap().as_str())
            }
            _ => panic!("Invalid type during parsing"),
        },

        // Assuming no API expects something like Option<Vec<_>>, but instead Vec<_> is just empty
        DataTypeDef::VecString
        | DataTypeDef::VecU8
        | DataTypeDef::VecU16
        | DataTypeDef::VecU32
        | DataTypeDef::VecU64
        | DataTypeDef::VecU128 => {
            buffer.push('[');

            match value {
                DataType::VecString(v) => {
                    for e in v.iter() {
                        buffer.push('"');
                        buffer.push_str(e);
                        buffer.push('"');
                        buffer.push(',');
                    }
                    buffer.pop();
                }
                DataType::VecU128(v) => {
                    for e in v.iter() {
                        buffer.push('"');
                        buffer.push_str(&*e.0.to_string());
                        buffer.push('"');
                        buffer.push(',');
                    }
                    buffer.pop();
                }
                DataType::VecU64(v) => {
                    for e in v.iter() {
                        buffer.push('"');
                        buffer.push_str(&*e.0.to_string());
                        buffer.push('"');
                        buffer.push(',');
                    }
                    buffer.pop();
                }
                DataType::VecU8(v) => {
                    for e in v.iter() {
                        buffer.push_str(&*e.to_string());
                        buffer.push(',');
                    }
                    buffer.pop();
                }
                DataType::VecU16(v) => {
                    for e in v.iter() {
                        buffer.push_str(&*e.to_string());
                        buffer.push(',');
                    }
                    buffer.pop();
                }
                DataType::VecU32(v) => {
                    for e in v.iter() {
                        buffer.push_str(&*e.to_string());
                        buffer.push(',');
                    }
                    buffer.pop();
                }
                _ => unreachable!(),
            }
            buffer.push(']');
        }
        _ => panic!("Invalid primitive type"),
    }
} */

//TODO return type result
pub fn validate<T: std::convert::AsRef<[DataType]>>(
    sources: &ValueContainer<T>,
    validator_refs: &[ValidatorRef],
    validator_exprs: &[Expression],
    metadata: &[FnCallMetadata],
    user_input: &[Vec<DataType>],
) -> bool {
    for v_ref in validator_refs.iter() {
        let validator = validator_exprs.get(v_ref.val_id as usize).unwrap();
        let inputs: &[DataType] = &user_input[v_ref.obj_id as usize];

        match v_ref.v_type {
            ValidatorType::Simple => {
                let inputs: &[DataType] = &user_input[v_ref.obj_id as usize];
                if !validator
                    .bind_and_eval(sources, inputs)
                    .try_into_bool()
                    .unwrap()
                {
                    return false;
                }
            }
            //TODO validate by pos??
            ValidatorType::Collection => {
                let obj_len = metadata.get(v_ref.obj_id as usize).unwrap().arg_names.len();
                let collection_size_total = inputs.len();

                // collection number of values must be multiple of collection object argument count
                assert_eq!(
                    collection_size_total % obj_len,
                    0,
                    "Invalid collection structure"
                );

                // apply validator for each obj in collection
                for (i, _) in inputs.iter().enumerate().step_by(obj_len) {
                    if !validator
                        .bind_and_eval(sources, &inputs[i..i + obj_len])
                        .try_into_bool()
                        .unwrap()
                    {
                        return false;
                    }
                }
            }
        }
    }

    true
}

/// Binds values from template's sources/storage to user provided input.
pub fn bind_from_sources<T: std::convert::AsRef<[DataType]>>(
    source_metadata: &[Vec<ArgSrc>],
    sources: &ValueContainer<T>,
    expressions: &[Expression],
    mut user_input: &mut Vec<Vec<DataType>>,
    metadata_pos: usize,
) {
    let mut result_args = Vec::with_capacity(source_metadata[metadata_pos].len());

    for arg_type in source_metadata[metadata_pos].iter() {
        match arg_type {
            ArgSrc::User(arg_pos) => result_args.push(std::mem::replace(
                &mut user_input[metadata_pos][*arg_pos as usize],
                DataType::Null,
            )),
            ArgSrc::Expression(expr_id) => result_args.push(
                expressions[*expr_id as usize]
                    .bind_and_eval(&sources, user_input[metadata_pos].as_slice()),
            ),
            ArgSrc::Object(id) => {
                result_args.push(DataType::Null);

                bind_from_sources(
                    source_metadata,
                    sources,
                    expressions,
                    &mut user_input,
                    *id as usize,
                );
            }
            ArgSrc::VecObject(id) => {
                result_args.push(DataType::Null);

                bind_vec_obj_args(
                    source_metadata,
                    sources,
                    expressions,
                    &mut user_input,
                    *id as usize,
                );
            }
            _ => result_args.push(get_value_from_source(arg_type, &sources)),
        }
    }

    std::mem::swap(&mut result_args, &mut user_input[metadata_pos]);
}

pub(crate) fn bind_vec_obj_args<T: std::convert::AsRef<[DataType]>>(
    source_metadata: &[Vec<ArgSrc>],
    sources: &ValueContainer<T>,
    expressions: &[Expression],
    mut user_input: &mut Vec<Vec<DataType>>,
    metadata_pos: usize,
) {
    let mut result_args = Vec::with_capacity(source_metadata.len());
    let mut obj_arg_pos = 0;
    let mut cycle_counter = 0;
    let obj_size = source_metadata[metadata_pos].len();

    //init result structures
    for i in 0..user_input[metadata_pos].len() {
        match source_metadata[metadata_pos][obj_arg_pos] {
            ArgSrc::User(arg_pos) => result_args.push(std::mem::replace(
                &mut user_input[metadata_pos][arg_pos as usize + cycle_counter * obj_size],
                DataType::Null,
            )),
            ArgSrc::Expression(expr_id) => result_args.push(
                expressions[expr_id as usize]
                    .bind_and_eval(&sources, user_input[metadata_pos].as_slice()),
            ),
            // VecObject can have object only as another VecObject
            ArgSrc::Object(_) => {
                unreachable!()
            }
            ArgSrc::VecObject(id) => {
                result_args.push(DataType::Null);

                bind_vec_obj_args(
                    source_metadata,
                    sources,
                    expressions,
                    &mut user_input,
                    id as usize,
                );
            }
            _ => result_args.push(get_value_from_source(
                &source_metadata[metadata_pos][obj_arg_pos],
                &sources,
            )),
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
}

pub(crate) fn get_value_from_source<T: std::convert::AsRef<[DataType]>>(
    arg_src: &ArgSrc,
    container: &ValueContainer<T>,
) -> DataType {
    match arg_src {
        ArgSrc::ConstsTpl(id) => container.tpl_consts.as_ref()[*id as usize].clone(),
        ArgSrc::ConstsSettings(id) => container.settings_consts.as_ref()[*id as usize].clone(),
        ArgSrc::ConstProps(id) => container.proposal_consts.as_ref()[*id as usize].clone(),
        ArgSrc::Storage(key) => container.storage.get_data(&key).unwrap(),
        ArgSrc::Const(const_id) => (container.dao_consts)(*const_id),
        _ => {
            unimplemented!()
        }
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
                args: vec![ArgSrc::ConstProps(0), ArgSrc::User(1)],
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

        let storage = StorageBucket::new(b"abc".to_vec());

        let dao_consts = Box::new(|id: u8| match id {
            0 => DataType::String("neardao.near".into()),
            _ => unimplemented!(),
        });

        let value_source = ValueContainer {
            dao_consts: &dao_consts,
            tpl_consts: &tpl_consts,
            settings_consts: &settings_consts,
            proposal_consts: &proposal_consts,
            storage: &storage,
        };

        assert!(validate(
            &value_source,
            validator_refs.as_slice(),
            validators.as_slice(),
            metadata.as_slice(),
            user_input.as_slice(),
        ));

        /* ------------------ Binding ------------------ */

        let source_metadata = vec![
            vec![ArgSrc::Object(1), ArgSrc::ConstProps(1)],
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
            vec![ArgSrc::ConstProps(2), ArgSrc::User(1)],
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
        );

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
