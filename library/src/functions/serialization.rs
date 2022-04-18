use crate::{
    types::datatype::{Datatype, Value},
    workflow::types::FnCallMetadata,
};

/// Serializes JSON string by metadata schema.
pub fn serialize_to_json(
    user_input: &[Vec<Value>],
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
            Datatype::Object(id) => {
                args.push_str(serialize_to_json(user_input, metadata, *id as usize).as_str());
            }
            Datatype::NullableObject(id) => {
                // check first elem in the obj array
                // if theres optional attribute with null on the 0th position, then this wont work
                if user_input[*id as usize][0] == Value::Null {
                    args.push_str("null");
                } else {
                    args.push_str(serialize_to_json(user_input, metadata, *id as usize).as_str());
                }
            }
            Datatype::VecObject(id) => {
                colection_to_json(&mut args, user_input, metadata, *id as usize);
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
    user_input: &[Vec<Value>],
    metadata: &[FnCallMetadata],
    metadata_id: usize,
) {
    let obj_size = metadata[metadata_id].arg_types.len();
    buf.push('[');
    if !user_input[metadata_id].is_empty() {
        for (i, _) in user_input[metadata_id].iter().enumerate().step_by(obj_size) {
            buf.push('{');
            for j in 0..obj_size {
                buf.push('"');
                buf.push_str(&metadata[metadata_id].arg_names[j]);
                buf.push('"');
                buf.push(':');
                match &metadata[metadata_id].arg_types[j] {
                    // Cannot use defaul obj serialization coz structure is different. Each Object in VecObj is technically VecObj.
                    Datatype::VecObject(id) => {
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
            buf.push('}');
            buf.push(',');
        }
        buf.pop();
    }
    buf.push(']');
}

/// Serializes collection of object to JSON.
/// Because of schema this has to be separate method.
#[allow(clippy::explicit_counter_loop)]
pub(crate) fn collection_obj_to_json(
    buf: &mut String,
    user_input: &[Vec<Value>],
    metadata: &[FnCallMetadata],
    metadata_id: usize,
    pos: usize,
) {
    buf.push('{');
    let mut counter = 0;
    let obj_size = metadata[metadata_id].arg_names.len();
    for i in pos * obj_size..pos * obj_size + obj_size {
        buf.push('"');
        buf.push_str(metadata[metadata_id].arg_names[counter].as_str());
        buf.push('"');
        buf.push(':');
        match &metadata[metadata_id].arg_types[counter] {
            Datatype::VecObject(id) => {
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
pub(crate) fn primitive_arg_to_json(buf: &mut String, datatype_defs: &Datatype, value: &Value) {
    match datatype_defs {
        Datatype::String(opt) => match (opt, value) {
            (true, Value::String(v)) | (false, Value::String(v)) => {
                buf.push('"');
                buf.push_str(v.as_str());
                buf.push('"');
            }
            (true, Value::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during serializing json"),
        },

        Datatype::Bool(opt) => match (opt, value) {
            (true, Value::Bool(v)) | (false, Value::Bool(v)) => match v {
                true => buf.push_str("true"),
                false => buf.push_str("false"),
            },
            (true, Value::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during parsing"),
        },
        Datatype::U64(opt) => match (opt, value) {
            (true, Value::U64(v)) | (false, Value::U64(v)) => {
                buf.push_str(&*v.to_string());
            }
            (true, Value::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during parsing"),
        },
        Datatype::U128(opt) => match (opt, value) {
            (true, Value::U128(v)) | (false, Value::U128(v)) => {
                buf.push('"');
                buf.push_str(&*v.0.to_string());
                buf.push('"');
            }
            (true, Value::Null) => buf.push_str("null"),
            _ => panic!("Invalid type during parsing"),
        },

        // Assuming no API expects something like Option<Vec<_>>, but instead Vec<_> is just empty
        Datatype::VecString | Datatype::VecU128 => {
            buf.push('[');

            match value {
                Value::VecString(v) => {
                    for e in v.iter() {
                        buf.push('"');
                        buf.push_str(e);
                        buf.push('"');
                        buf.push(',');
                    }
                    buf.pop();
                }
                Value::VecU128(v) => {
                    for e in v.iter() {
                        buf.push('"');
                        buf.push_str(&*e.0.to_string());
                        buf.push('"');
                        buf.push(',');
                    }
                    buf.pop();
                }
                Value::VecU64(v) => {
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
