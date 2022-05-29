//! Helper functions to serialize ObjectMetadata with ActivityInput into JSON string.

use crate::{
    types::{activity_input::ActivityInput, datatype::Datatype, error::ProcessingError},
    workflow::types::ObjectMetadata,
};

use super::utils::object_key;

const JSON_NULL: &str = "null";

/// Serialize JSON string by metadata schema.
pub fn serialize_to_json(
    mut input: Box<dyn ActivityInput>,
    metadata: &[ObjectMetadata],
) -> Result<String, ProcessingError> {
    let mut args = String::with_capacity(256);
    args.push('{');
    for i in 0..metadata[0].arg_names.len() {
        args.push('"');
        args.push_str(metadata[0].arg_names[i].as_str());
        args.push('"');
        args.push(':');
        match &metadata[0].arg_types[i] {
            Datatype::Object(id) => {
                object_to_json(
                    &mut args,
                    input.as_mut(),
                    metadata,
                    *id as usize,
                    &metadata[0].arg_names[i],
                )?;
            }
            Datatype::NullableObject(id) => {
                if input.get(&metadata[0].arg_names[i]).is_some() {
                    args.push_str(JSON_NULL);
                } else {
                    object_to_json(
                        &mut args,
                        input.as_mut(),
                        metadata,
                        *id as usize,
                        &metadata[0].arg_names[i],
                    )?;
                }
            }
            Datatype::VecObject(id) => {
                collection_to_json(
                    &mut args,
                    input.as_mut(),
                    metadata,
                    *id as usize,
                    &metadata[0].arg_names[i],
                )?;
            }
            _ => primitive_to_json(
                &mut args,
                input.as_mut(),
                &metadata[0].arg_types[i],
                &metadata[0].arg_names[i],
            )?,
        }
        args.push(',');
    }
    args.pop();
    args.push('}');
    Ok(args)
}

fn object_to_json(
    buf: &mut String,
    input: &mut dyn ActivityInput,
    metadata: &[ObjectMetadata],
    meta_pos: usize,
    obj_prefix: &str,
) -> Result<(), ProcessingError> {
    buf.push('{');
    for i in 0..metadata[meta_pos].arg_names.len() {
        buf.push('"');
        buf.push_str(metadata[meta_pos].arg_names[i].as_str());
        buf.push('"');
        buf.push(':');

        let mut key =
            String::with_capacity(obj_prefix.len() + 1 + metadata[meta_pos].arg_names[i].len());
        key.push_str(obj_prefix);
        key.push('.');
        key.push_str(metadata[meta_pos].arg_names[i].as_str());
        match &metadata[meta_pos].arg_types[i] {
            Datatype::Object(id) => {
                object_to_json(buf, input, metadata, *id as usize, key.as_str())?;
            }
            Datatype::NullableObject(id) => {
                if input.get(key.as_str()).is_some() {
                    buf.push_str(JSON_NULL);
                } else {
                    object_to_json(buf, input, metadata, *id as usize, key.as_str())?;
                }
            }
            Datatype::VecObject(id) => {
                collection_to_json(buf, input, metadata, *id as usize, key.as_str())?;
            }
            _ => {
                primitive_to_json(buf, input, &metadata[meta_pos].arg_types[i], key.as_str())?;
            }
        }
        buf.push(',');
    }
    buf.pop();
    buf.push('}');
    Ok(())
}
fn collection_to_json(
    buf: &mut String,
    input: &mut dyn ActivityInput,
    metadata: &[ObjectMetadata],
    meta_pos: usize,
    obj_prefix: &str,
) -> Result<(), ProcessingError> {
    buf.push('[');
    if input.get(obj_prefix).is_none() {
        let mut counter: u8 = 0;
        let mut key = object_key(
            obj_prefix,
            counter.to_string().as_str(),
            metadata[meta_pos].arg_names[0].as_str(),
        );
        while input.has_key(key.as_str()) {
            key.clear();
            key.push_str(obj_prefix);
            key.push('.');
            key.push_str(counter.to_string().as_str());
            object_to_json(buf, input, metadata, meta_pos, &key)?;
            counter += 1;
            buf.push(',');
            key.clear();
            key.push_str(obj_prefix);
            key.push('.');
            key.push_str(counter.to_string().as_str());
            key.push('.');
            key.push_str(metadata[meta_pos].arg_names[0].as_str());
        }
        if counter > 0 {
            buf.pop();
        }
    }
    buf.push(']');
    Ok(())
}
fn primitive_to_json(
    buf: &mut String,
    input: &mut dyn ActivityInput,
    datatype_def: &Datatype,
    key: &str,
) -> Result<(), ProcessingError> {
    let value = if let Some(value) = input.take(key) {
        value
    } else if datatype_def.is_optional() {
        buf.push_str(JSON_NULL);
        return Ok(());
    } else {
        return Err(ProcessingError::MissingInputKey(key.into()));
    };

    match datatype_def {
        Datatype::U64(opt) => match (opt, value.is_null()) {
            (true, true) => buf.push_str(JSON_NULL),
            (false, true) => return Err(ProcessingError::MissingInputKey(key.into())),
            _ => buf.push_str(value.try_into_u64()?.to_string().as_str()),
        },
        Datatype::U128(opt) => match (opt, value.is_null()) {
            (true, true) => buf.push_str(JSON_NULL),
            (false, true) => return Err(ProcessingError::MissingInputKey(key.into())),
            _ => {
                buf.push('"');
                buf.push_str(value.try_into_u128()?.to_string().as_str());
                buf.push('"');
            }
        },
        Datatype::String(opt) => match (opt, value.is_null()) {
            (true, true) => buf.push_str(JSON_NULL),
            (false, true) => return Err(ProcessingError::MissingInputKey(key.into())),
            _ => {
                let x = value.try_into_str()?;
                buf.push('"');
                buf.push_str(x);
                buf.push('"');
            }
        },
        Datatype::VecU64 => {
            let v = value.try_into_vec_u64()?;
            buf.push('[');
            for e in v.into_iter() {
                buf.push_str(e.to_string().as_str());
                buf.push(',');
            }
            buf.pop();
            buf.push(']');
        }
        Datatype::Bool(opt) => match (opt, value.is_null()) {
            (true, true) => buf.push_str(JSON_NULL),
            (false, true) => return Err(ProcessingError::MissingInputKey(key.into())),
            _ => match value.try_into_bool()? {
                true => buf.push_str("true"),
                false => buf.push_str("false"),
            },
        },
        Datatype::VecU128 => {
            let v = value.try_into_vec_u64()?;
            buf.push('[');
            for e in v.into_iter() {
                buf.push('"');
                buf.push_str(e.to_string().as_str());
                buf.push('"');
                buf.push(',');
            }
            buf.pop();
            buf.push(']');
        }
        Datatype::VecString => {
            let v = value.try_into_vec_string()?;
            buf.push('[');
            for e in v.iter() {
                buf.push('"');
                buf.push_str(e.to_string().as_str());
                buf.push('"');
                buf.push(',');
            }
            buf.pop();
            buf.push(']');
        }
        _ => (),
    }
    Ok(())
}
