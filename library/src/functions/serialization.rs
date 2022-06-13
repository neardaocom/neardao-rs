//! Helper functions to serialize ObjectMetadata with ActivityInput into JSON string.
//! TODO: Currently invalid metadata still cause panic.

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
                    0,
                )?;
            }
            Datatype::OptionalObject(id) => {
                if input.get(&metadata[0].arg_names[i]).is_some() {
                    args.push_str(JSON_NULL);
                } else {
                    object_to_json(
                        &mut args,
                        input.as_mut(),
                        metadata,
                        *id as usize,
                        &metadata[0].arg_names[i],
                        0,
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
            Datatype::Enum(ids) => {
                enum_to_json(
                    &mut args,
                    input.as_mut(),
                    metadata,
                    ids,
                    &metadata[0].arg_names[i],
                    false,
                )?;
            }
            Datatype::OptionalEnum(ids) => {
                enum_to_json(
                    &mut args,
                    input.as_mut(),
                    metadata,
                    ids,
                    &metadata[0].arg_names[i],
                    true,
                )?;
            }
            Datatype::VecTuple(id) => {
                collection_tuple_to_json(
                    &mut args,
                    input.as_mut(),
                    metadata,
                    *id as usize,
                    &metadata[0].arg_names[i],
                )?;
            }
            Datatype::Tuple(id) => tuple_to_json(
                &mut args,
                input.as_mut(),
                metadata,
                *id as usize,
                &metadata[0].arg_names[i],
                false,
            )?,
            Datatype::OptionalTuple(id) => tuple_to_json(
                &mut args,
                input.as_mut(),
                metadata,
                *id as usize,
                &metadata[0].arg_names[i],
                true,
            )?,
            Datatype::VecEnum(ids) => {
                collection_enum_to_json(
                    &mut args,
                    input.as_mut(),
                    metadata,
                    ids,
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
    skip_arg_prefix: usize,
) -> Result<(), ProcessingError> {
    buf.push('{');
    for i in 0..metadata[meta_pos].arg_names.len() {
        buf.push('"');
        buf.push_str(&metadata[meta_pos].arg_names[i][skip_arg_prefix..]);
        buf.push('"');
        buf.push(':');
        let mut key =
            String::with_capacity(obj_prefix.len() + 1 + metadata[meta_pos].arg_names[i].len());
        key.push_str(obj_prefix);
        key.push('.');
        key.push_str(metadata[meta_pos].arg_names[i].as_str());
        match &metadata[meta_pos].arg_types[i] {
            Datatype::Object(id) => {
                object_to_json(
                    buf,
                    input,
                    metadata,
                    *id as usize,
                    key.as_str(),
                    skip_arg_prefix,
                )?;
            }
            Datatype::OptionalObject(id) => {
                if input.get(key.as_str()).is_some() {
                    buf.push_str(JSON_NULL);
                } else {
                    object_to_json(
                        buf,
                        input,
                        metadata,
                        *id as usize,
                        key.as_str(),
                        skip_arg_prefix,
                    )?;
                }
            }
            Datatype::VecObject(id) => {
                collection_to_json(buf, input, metadata, *id as usize, key.as_str())?;
            }
            Datatype::Enum(ids) => {
                enum_to_json(buf, input, metadata, ids, &metadata[0].arg_names[i], false)?;
            }
            Datatype::OptionalEnum(ids) => {
                enum_to_json(buf, input, metadata, ids, &metadata[0].arg_names[i], true)?;
            }
            Datatype::VecTuple(id) => {
                collection_tuple_to_json(
                    buf,
                    input,
                    metadata,
                    *id as usize,
                    &metadata[0].arg_names[i],
                )?;
            }
            Datatype::Tuple(id) => tuple_to_json(
                buf,
                input,
                metadata,
                *id as usize,
                &metadata[0].arg_names[i],
                false,
            )?,
            Datatype::OptionalTuple(id) => tuple_to_json(
                buf,
                input,
                metadata,
                *id as usize,
                &metadata[0].arg_names[i],
                true,
            )?,
            Datatype::VecEnum(ids) => {
                collection_enum_to_json(buf, input, metadata, ids, &metadata[0].arg_names[i])?;
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
        while let Some(_) = input.get(key.as_str()) {
            key.clear();
            key.push_str(obj_prefix);
            key.push('.');
            key.push_str(counter.to_string().as_str());
            object_to_json(buf, input, metadata, meta_pos, &key, 0)?;
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
fn collection_tuple_to_json(
    buf: &mut String,
    input: &mut dyn ActivityInput,
    metadata: &[ObjectMetadata],
    meta_pos: usize,
    obj_prefix: &str,
) -> Result<(), ProcessingError> {
    buf.push('[');
    if input.get(obj_prefix).is_none() {
        let mut counter: u8 = 0;
        let mut key = object_key(obj_prefix, counter.to_string().as_str(), "0");
        while let Some(_) = input.get(key.as_str()) {
            key.clear();
            key.push_str(obj_prefix);
            key.push('.');
            key.push_str(counter.to_string().as_str());
            tuple_to_json(buf, input, metadata, meta_pos, &key, false)?;
            counter += 1;
            buf.push(',');
            key.clear();
            key.push_str(obj_prefix);
            key.push('.');
            key.push_str(counter.to_string().as_str());
            key.push_str(".0");
        }
        if counter > 0 {
            buf.pop();
        }
    }
    buf.push(']');
    Ok(())
}
fn tuple_to_json(
    buf: &mut String,
    input: &mut dyn ActivityInput,
    metadata: &[ObjectMetadata],
    meta_pos: usize,
    obj_prefix: &str,
    optional: bool,
) -> Result<(), ProcessingError> {
    let mut key = String::with_capacity(obj_prefix.len() + 2);
    key.push_str(obj_prefix);
    key.push_str(".0");
    if !optional && input.get(&key).is_none() {
        return Err(ProcessingError::MissingInputKey("missing tuple key".into()));
    } else if optional && input.get(&key).is_none() {
        buf.push_str(JSON_NULL);
        return Ok(());
    }
    buf.push('[');
    for idx in 0..metadata[meta_pos].arg_types.len() {
        key.clear();
        key.push_str(obj_prefix);
        key.push('.');
        key.push_str(idx.to_string().as_str());
        let datatype = &metadata[meta_pos].arg_types[idx];
        primitive_to_json(buf, input, datatype, &key)?;
        buf.push(',');
    }
    buf.pop();
    buf.push(']');
    Ok(())
}
fn enum_to_json(
    buf: &mut String,
    input: &mut dyn ActivityInput,
    metadata: &[ObjectMetadata],
    meta_pos: &[u8],
    obj_prefix: &str,
    optional: bool,
) -> Result<(), ProcessingError> {
    let mut found = false;
    let mut string_enum = false;
    buf.push('{');
    for id in meta_pos {
        let enum_first_key = metadata[*id as usize].arg_names[0].as_str();
        let key = object_key(obj_prefix, enum_first_key, "");
        if input.get(&key).is_some() {
            let dot_pos = enum_first_key
                .find(|c| c == '.')
                .unwrap_or_else(|| enum_first_key.len());
            if dot_pos == enum_first_key.len() {
                buf.pop();
                buf.push('"');
                buf.push_str(enum_first_key);
                buf.push('"');
                string_enum = true;
            } else {
                let enum_variant = &enum_first_key[0..dot_pos];
                buf.push('"');
                buf.push_str(enum_variant);
                buf.push('"');
                buf.push(':');
                object_to_json(
                    buf,
                    input,
                    metadata,
                    *id as usize,
                    &obj_prefix,
                    enum_variant.len() + 1,
                )?;
            }
            found = true;
            break;
        }
    }
    if !found && !optional {
        return Err(ProcessingError::MissingInputKey(
            "missing enum object".into(),
        ));
    } else if !found {
        buf.pop();
        buf.push_str(JSON_NULL);
    } else if !string_enum {
        buf.push('}');
    }
    Ok(())
}
fn collection_enum_to_json(
    buf: &mut String,
    input: &mut dyn ActivityInput,
    metadata: &[ObjectMetadata],
    meta_pos: &[u8],
    obj_prefix: &str,
) -> Result<(), ProcessingError> {
    buf.push('[');
    if input.get(obj_prefix).is_none() {
        let mut counter: u8 = 0;
        let possible_enum_variants = get_enum_first_keys(metadata, meta_pos)?;
        let mut key = object_key(obj_prefix, counter.to_string().as_str(), "");
        let mut tmp_key = String::with_capacity(obj_prefix.len() + 16);
        while exist_next_variant(input, &key, possible_enum_variants.as_slice(), &mut tmp_key) {
            key.clear();
            key.push_str(obj_prefix);
            key.push('.');
            key.push_str(counter.to_string().as_str());
            enum_to_json(buf, input, metadata, meta_pos, &key, false)?;
            counter += 1;
            buf.push(',');
            key.clear();
            key.push_str(obj_prefix);
            key.push('.');
            key.push_str(counter.to_string().as_str());
        }
        if counter > 0 {
            buf.pop();
        }
    }
    buf.push(']');
    Ok(())
}
fn get_enum_first_keys<'a>(
    metadata: &'a [ObjectMetadata],
    meta_pos: &[u8],
) -> Result<Vec<&'a str>, ProcessingError> {
    let mut variants = Vec::with_capacity(meta_pos.len());
    for id in meta_pos {
        let enum_first_key = metadata[*id as usize].arg_names[0].as_str();
        variants.push(enum_first_key);
    }
    Ok(variants)
}
fn exist_next_variant(
    input: &dyn ActivityInput,
    prefix: &str,
    variants: &[&str],
    tmp_key: &mut String,
) -> bool {
    for first_key in variants {
        tmp_key.clear();
        tmp_key.push_str(prefix);
        tmp_key.push('.');
        tmp_key.push_str(first_key);
        if input.get(&tmp_key).is_some() {
            return true;
        }
    }
    false
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
