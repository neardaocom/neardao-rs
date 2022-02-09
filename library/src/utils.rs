use near_sdk::{serde_json, Metadata};

use crate::{
    storage::StorageBucket,
    types::{DataType, DataTypeDef, FnCallMetadata, ValidatorType},
    workflow::{ArgType, Expression},
};

/// Validates args by validators
/// Only FnCalls require
pub fn validate_args(
    binds: &[DataType],
    obj_validator: &[ValidatorType],
    validator_expr: &[Expression],
    storage: &StorageBucket,
    args: &[Vec<DataType>],
    args_collections: &[Vec<DataType>],
    metadata: &[FnCallMetadata],
) -> bool {
    let mut next_args_vec_idx: usize = 0;

    for (v_id, validator_type) in obj_validator.iter().enumerate() {
        let expr = &validator_expr[v_id];
        match validator_type {
            // if structure contains vec<obj> then we check for each vec<obj>
            // this way we can validate values for each obj in vec<obj>
            ValidatorType::Collection(_) => {
                let current_metadata = &metadata[validator_type.get_id() as usize];
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
                        .bind_and_eval(storage, binds, &collection[i..i + params_len])
                        .try_into_bool()
                        .unwrap()
                    {
                        return false;
                    }
                }
                // we advance object type in collection so position of collection objects is important
                next_args_vec_idx += 1;
            }

            ValidatorType::Primitive(id) => {
                // else just check that object
                if !expr
                    .bind_and_eval(storage, binds, &args[*id as usize])
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

// TODO beautify
/// Binds values by defined schema input_defs.
pub fn bind_args(
    consts: &dyn Fn(u8) -> DataType, //dao specific values
    binds: &[DataType],
    input_defs: &[Vec<ArgType>],
    storage: &StorageBucket,
    mut args: &mut Vec<Vec<DataType>>,
    mut args_collections: &mut Vec<Vec<DataType>>,
    //metadata: &[FnCallMetadata],
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
            ArgType::Free => result_args.push(std::mem::replace(
                &mut args[metadata_pos][arg_pos],
                DataType::Null,
            )),
            ArgType::Bind(id) => result_args.push(binds[*id as usize].clone()),
            ArgType::Storage(key) => {
                result_args.push(storage.get_data(key).unwrap());
            }
            ArgType::Const(const_id) => result_args.push(consts(*const_id)),
            ArgType::Expression(expr) => unimplemented!(),
            ArgType::Object(id) => {
                result_args.push(DataType::Null);

                bind_args(
                    consts,
                    binds,
                    input_defs,
                    &storage,
                    &mut args,
                    &mut args_collections,
                    //metadata,
                    *id as usize - next_collection_obj_idx,
                    next_collection_obj_idx,
                );
            }

            ArgType::VecObject(id) => {
                bind_vec_obj_args(
                    consts,
                    binds,
                    input_defs[*id as usize].as_slice(),
                    storage,
                    &mut args_collections.get_mut(next_collection_obj_idx).unwrap(),
                );
                result_args.push(DataType::Null);
                next_collection_obj_idx += 1;
            }
        }
    }

    std::mem::swap(&mut result_args, &mut args[metadata_pos]);
}

pub(crate) fn bind_vec_obj_args(
    consts: &dyn Fn(u8) -> DataType,
    binds: &[DataType],
    input_defs: &[ArgType],
    storage: &StorageBucket,
    mut args_collections: &mut Vec<DataType>,
) {
    let mut result_args = Vec::with_capacity(input_defs.len());
    let mut input_defs_pos = 0;

    for (_arg_pos, data) in args_collections.iter().enumerate() {
        match &input_defs[input_defs_pos] {
            ArgType::Free => result_args.push(data.clone()),
            ArgType::Bind(id) => result_args.push(binds[*id as usize].clone()),
            ArgType::Storage(key) => {
                result_args.push(storage.get_data(&key).unwrap());
            }
            ArgType::Const(const_id) => result_args.push(consts(*const_id)),
            _ => unimplemented!(),
        }

        // reset arg type def
        if input_defs_pos == input_defs.len() - 1 {
            input_defs_pos = 0;
        } else {
            input_defs_pos += 1;
        }
    }

    std::mem::swap(&mut result_args, &mut args_collections);
}

/// Parses arguments to output JSON string by metadata schema
pub fn args_to_json(
    arg_values: &[Vec<DataType>],
    arg_collection_values: &[Vec<DataType>],
    metadata: &[FnCallMetadata],
    metadata_id: usize,
    //    collection_pos: Option<usize>,
) -> String {
    let mut next_collection_obj_idx: usize = 0;
    // Create raw json object string
    let mut args = String::with_capacity(128);
    args.push('{');

    /*     let source = match collection_pos {
        Some(pos) => &[vec![arg_collection_values[pos]]],
        None => arg_values,
    }; */

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
                //let metadata = &metadata[metadata_id];
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

/// Creates JSON representation of value by data_type_def and pushes it buffer
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
}

#[cfg(test)]
mod test {

    use std::convert::TryFrom;

    use near_sdk::{
        json_types::{
            ValidAccountId, WrappedBalance, WrappedDuration, WrappedTimestamp, U128, U64,
        },
        serde::{Deserialize, Serialize},
        serde_json,
    };

    use crate::{
        expression::{EExpr, EOp, ExprTerm, Op, RelOp, TExpr},
        storage::StorageBucket,
        types::{DataType, DataTypeDef, FnCallMetadata},
        utils::{args_to_json, bind_args, validate_args, ValidatorType},
        workflow::{ArgType, ExprArg, Expression},
    };

    /******  Skyward sale_create structures  ******/
    type BasicPoints = u16;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SaleInput {
        pub title: String,
        pub url: Option<String>,
        pub permissions_contract_id: Option<ValidAccountId>,

        pub out_tokens: Vec<SaleInputOutToken>,

        pub in_token_account_id: ValidAccountId,

        pub start_time: WrappedTimestamp,
        pub duration: WrappedDuration,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SaleInputOutToken {
        pub token_account_id: ValidAccountId,
        pub balance: WrappedBalance,
        pub referral_bpt: Option<BasicPoints>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct SaleCreateArgOjb {
        pub sale: SaleInput,
    }

    /****** TEST CASES ******/

    #[test]
    fn full_scenario_mapping_skyward_sale_create() {
        /*
        Testcase object structure - Skyward - sale_create input:
        {sale:
            {
                title: String
                url: String (optional)
                permissions_contract_id: String (optional)
                out_tokens: [
                    {
                        token_account_id: String
                        balance: u128
                        referral_bpt: u16 (optional)
                    }
                ]
                in_token_account_id: String
                start_time: u64,
                duration: u64,
            }
        }

        Metadata mapping:

        types: [String, U128, obj(1)] [String, VecObj(2), NullableObj(3)] [String, VecString] [String] //defines object schema
        values: ["from.near", "1000", null] ["dao.near", null] ["420"]                  //object values
        obj_collections: [ [amount,["msg1", "msg2"], amount,["msg1", "msg2"]] ] //object arrays

        obj_validator = [obj_id] in schema, pos is same for its validator
        validator_expr = which value(pos) in the object provided by user to validate

        */

        let metadata = vec![
            FnCallMetadata {
                arg_names: vec!["sale".into()],
                arg_types: vec![DataTypeDef::Object(1)],
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
                ],
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::String(true),
                    DataTypeDef::String(true),
                    DataTypeDef::VecObject(2),
                    DataTypeDef::String(false),
                    DataTypeDef::U64(false),
                    DataTypeDef::U64(false),
                ],
            },
            FnCallMetadata {
                arg_names: vec![
                    "token_account_id".into(),
                    "balance".into(),
                    "referral_bpt".into(),
                ],
                arg_types: vec![
                    DataTypeDef::String(false),
                    DataTypeDef::U128(false),
                    DataTypeDef::U16(true),
                ],
            },
        ];

        let binds = vec![DataType::U128(U128::from(1000))];
        let obj_validators = vec![ValidatorType::Collection(2)];
        let validator_expr = vec![
            // validates first object
            Expression {
                args: vec![ExprArg::Bind(0), ExprArg::User(1)],
                expr: EExpr::Boolean(TExpr {
                    operators: vec![Op {
                        op_type: EOp::Rel(RelOp::Gt),
                        operands_ids: [0, 1],
                    }],
                    terms: vec![ExprTerm::Arg(0), ExprTerm::Arg(1)],
                }),
            },
        ];

        let mut user_inputs = vec![
            vec![DataType::Null],
            vec![
                DataType::String("Neardao token auction".into()),
                DataType::String("www.neardao.com".into()),
                DataType::String("neardao.testnet".into()),
                DataType::Null,
                DataType::String("wrap.near".into()),
                DataType::U64(0.into()),
                DataType::U64(3600.into()),
            ],
        ];
        let mut user_collection_inputs = vec![vec![
            DataType::String("neardao.testnet".into()),
            DataType::U128(U128::from(999)), // 1000 is limit
            DataType::Null,
        ]];

        let storage = StorageBucket::new(b"abc".to_vec());

        assert!(validate_args(
            binds.as_slice(),
            obj_validators.as_slice(),
            validator_expr.as_slice(),
            &storage,
            user_inputs.as_slice(),
            user_collection_inputs.as_slice(),
            metadata.as_slice(),
        ));

        /* ------------------ Binding ------------------ */

        let dao_consts = Box::new(|id: u8| match id {
            0 => DataType::String("neardao.near".into()),
            _ => unimplemented!(),
        });
        let activity_inputs = vec![
            vec![ArgType::Object(1)],
            vec![
                ArgType::Free,
                ArgType::Free,
                ArgType::Const(0),
                ArgType::VecObject(2),
                ArgType::Free,
                ArgType::Free,
                ArgType::Free,
            ],
            vec![ArgType::Const(0), ArgType::Free, ArgType::Free],
        ];

        //TODO implement and test expr
        let activity_expr: Vec<Expression> = vec![];

        bind_args(
            &dao_consts,
            binds.as_slice(),
            activity_inputs.as_slice(),
            &storage,
            &mut user_inputs,
            &mut user_collection_inputs,
            0,
            0,
        );

        let expected_bind_result = vec![
            vec![DataType::Null],
            vec![
                DataType::String("Neardao token auction".into()),
                DataType::String("www.neardao.com".into()),
                DataType::String("neardao.near".into()),
                DataType::Null,
                DataType::String("wrap.near".into()),
                DataType::U64(0.into()),
                DataType::U64(3600.into()),
            ],
        ];

        let expected_bind_collection_result = vec![vec![
            DataType::String("neardao.near".into()),
            DataType::U128(999.into()), // 1000 is limit
            DataType::Null,
        ]];

        assert_eq!(user_inputs, expected_bind_result);
        assert_eq!(user_collection_inputs, expected_bind_collection_result);

        /* ------------------ Parsing to JSON ------------------ */

        let out_tokens = SaleInputOutToken {
            token_account_id: ValidAccountId::try_from("neardao.near").unwrap(),
            balance: 999.into(),
            referral_bpt: None,
        };
        let sale_create_input = SaleInput {
            title: "Neardao token auction".into(),
            url: Some("www.neardao.com".into()),
            permissions_contract_id: Some(ValidAccountId::try_from("neardao.near").unwrap()),
            out_tokens: vec![out_tokens],
            in_token_account_id: ValidAccountId::try_from("wrap.near").unwrap(),
            start_time: 0.into(),
            duration: 3600.into(),
        };

        // Wanted arg object
        let args = SaleCreateArgOjb {
            sale: sale_create_input,
        };

        let result_json_string = args_to_json(
            user_inputs.as_slice(),
            user_collection_inputs.as_slice(),
            metadata.as_slice(),
            0,
        );
        let expected_json_string = serde_json::to_string(&args).unwrap();
        assert_eq!(result_json_string, expected_json_string);
        assert_eq!(
            serde_json::from_str::<SaleCreateArgOjb>(&result_json_string).unwrap(),
            args
        );
    }
}
