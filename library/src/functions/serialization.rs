use core::panic;

use crate::{
    types::{
        activity_input::ActivityInput,
        datatype::{Datatype, Value},
    },
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

// TODO: Finish, benchmark and test.
pub fn serialize_to_json_new(
    user_input: Box<dyn ActivityInput>,
    metadata: &[FnCallMetadata],
) -> String {
    let mut json = String::with_capacity(256);
    let objects_count = metadata.len();
    let mut obj_done = 0;
    let mut current_obj = 0;
    let mut pos = 0;
    let mut len = metadata[0].arg_names.len();
    let mut stack: Vec<(usize, usize, usize, char)> = Vec::with_capacity(4);
    let mut prefix_stack: Vec<&str> = Vec::with_capacity(4);
    let mut is_vec_obj = false;
    let mut current_vec_obj = 0;

    json.push('{');
    while obj_done < objects_count {
        while pos < len {
            let (name, datatype) = (
                metadata[current_obj].arg_names[pos].as_str(),
                &metadata[current_obj].arg_types[pos],
            );

            let search_key = if is_vec_obj {
                format!("{}.{}.{}", prefix_stack.join("."), current_vec_obj, name)
            } else if !prefix_stack.is_empty() {
                format!("{}.{}", prefix_stack.join("."), name)
            } else {
                name.to_owned()
            };

            let value = user_input.get(search_key.as_str());

            if is_vec_obj && value.is_none() {
                if pos == 0 {
                    // We reached end of the collection
                    if let Some((obj_id, prev_pos, prev_len, bracket)) = stack.pop() {
                        current_obj = obj_id;
                        pos = prev_pos;
                        len = prev_len;
                        json.pop();
                        dbg!(bracket);
                        json.push(bracket);
                        json.push(',');
                        prefix_stack.pop();
                        is_vec_obj = false;
                        current_vec_obj = 0;
                        obj_done += 1;
                        continue;
                    } else {
                        panic!("Expected stack value");
                    }
                } else {
                    panic!(
                        "Missing attribute {} for collection object id: {}",
                        name, current_obj
                    );
                }
            } else if is_vec_obj && pos == 0 {
                json.push('{');
            }

            json.push('"');
            json.push_str(name);
            json.push('"');
            json.push(':');

            match datatype {
                Datatype::Bool(_) => todo!(),
                Datatype::U64(opt) => match opt {
                    true => {
                        let val = value
                            .expect("Expected integer value")
                            .try_into_u64()
                            .expect("Value is not integer");
                        json.push_str(&*val.to_string());
                    }
                    false => match value {
                        Some(s) => {
                            let val = s.try_into_u64().expect("Value is not integer");
                            json.push_str(&*val.to_string());
                        }
                        None => json.push_str("null"),
                    },
                },
                Datatype::U128(_) => todo!(),
                Datatype::String(opt) => match opt {
                    true => {
                        let str_val = value
                            .expect("Expected string value")
                            .try_into_str()
                            .expect("Value is not string");
                        json.push('"');
                        json.push_str(str_val);
                        json.push('"');
                    }
                    false => match value {
                        Some(s) => {
                            let str_val = s.try_into_str().expect("Value is not string");
                            json.push('"');
                            json.push_str(str_val);
                            json.push('"');
                        }
                        None => json.push_str("null"),
                    },
                },
                Datatype::VecU64 => todo!(),
                Datatype::VecU128 => todo!(),
                Datatype::VecString => todo!(),
                Datatype::Object(id) => {
                    json.push('{');

                    stack.push((current_obj, pos + 1, len, '}'));
                    prefix_stack.push(name);

                    // Jump to next obj
                    pos = 0;
                    len = metadata[*id as usize].arg_names.len();
                    current_obj = *id as usize;
                    continue;
                }
                Datatype::NullableObject(_) => todo!(),
                Datatype::VecObject(id) => {
                    json.push('[');

                    stack.push((current_obj, pos + 1, len, ']'));
                    prefix_stack.push(name);

                    // Jump to next obj
                    pos = 0;
                    len = metadata[*id as usize].arg_names.len();
                    current_obj = *id as usize;
                    is_vec_obj = true;
                    continue;
                }
            }
            json.push(',');
            pos += 1;

            if is_vec_obj && pos == len {
                json.pop();
                pos = 0;
                current_vec_obj += 1;
                json.push('}');
                json.push(',');
            }
        }
        json.pop();
        obj_done += 1;

        if let Some((obj_id, prev_pos, prev_len, bracket)) = stack.pop() {
            pos = prev_pos;
            len = prev_len;
            current_obj = obj_id;
            json.push(bracket);
            json.push(',');
            prefix_stack.pop();
        } else {
            break;
        }
    }

    if let Some(',') = json.chars().last() {
        json.pop();
    }
    json.push('}');

    json
}
