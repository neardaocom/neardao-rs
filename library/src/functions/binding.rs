use crate::{
    types::{
        datatype::Value,
        error::{ProcessingError, SourceError},
    },
    workflow::{
        expression::Expression,
        types::{ArgSrc, ValueContainer},
    },
};

/// Binds values from template's sources/storage to replace those in `user_input`.
/// Schema is defined by `source_metadata` values.
/// Returns `Err(())` in case input/Wf structure is bad.
pub fn bind_from_sources<T: std::convert::AsRef<[Value]>>(
    source_metadata: &[Vec<ArgSrc>],
    sources: &ValueContainer<T>,
    expressions: &[Expression],
    user_input: &mut Vec<Vec<Value>>,
    metadata_pos: usize,
) -> Result<(), ProcessingError> {
    let mut result_args = Vec::with_capacity(
        source_metadata
            .get(metadata_pos)
            .ok_or(SourceError::InvalidArgId)?
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
                    Value::Null,
                ))
            }

            ArgSrc::Expression(expr_id) => result_args.push(
                expressions
                    .get(*expr_id as usize)
                    .ok_or(SourceError::InvalidArgId)?
                    .bind_and_eval(
                        sources,
                        user_input
                            .get(metadata_pos)
                            .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
                            .as_slice(),
                    )?,
            ),
            ArgSrc::Object(id) => {
                result_args.push(Value::Null);

                bind_from_sources(
                    source_metadata,
                    sources,
                    expressions,
                    user_input,
                    *id as usize,
                )?;
            }
            ArgSrc::VecObject(id) => {
                result_args.push(Value::Null);

                bind_vec_obj_args(
                    source_metadata,
                    sources,
                    expressions,
                    user_input,
                    *id as usize,
                )?;
            }
            _ => result_args.push(get_value_from_source(arg_type, sources)?),
        }
    }

    std::mem::swap(&mut result_args, &mut user_input[metadata_pos]);

    Ok(())
}

pub(crate) fn bind_vec_obj_args<T: std::convert::AsRef<[Value]>>(
    source_metadata: &[Vec<ArgSrc>],
    sources: &ValueContainer<T>,
    expressions: &[Expression],
    user_input: &mut Vec<Vec<Value>>,
    metadata_pos: usize,
) -> Result<(), ProcessingError> {
    let mut result_args = Vec::with_capacity(source_metadata.len());
    let mut obj_arg_pos = 0;
    let mut cycle_counter = 0;
    let obj_size = source_metadata
        .get(metadata_pos)
        .ok_or(SourceError::InvalidArgId)?
        .len();

    for _ in 0..user_input
        .get(metadata_pos)
        .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
        .len()
    {
        let arg_src = source_metadata
            .get(metadata_pos)
            .ok_or(SourceError::InvalidArgId)?
            .get(obj_arg_pos)
            .ok_or(SourceError::InvalidArgId)?;

        match arg_src {
            ArgSrc::User(arg_pos) => {
                let _ = user_input
                    .get(metadata_pos)
                    .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
                    .get(*arg_pos as usize + cycle_counter * obj_size)
                    .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?;

                result_args.push(std::mem::replace(
                    &mut user_input[metadata_pos][*arg_pos as usize + cycle_counter * obj_size],
                    Value::Null,
                ))
            }
            ArgSrc::Expression(expr_id) => result_args.push(
                expressions
                    .get(*expr_id as usize)
                    .ok_or(SourceError::InvalidArgId)?
                    .bind_and_eval(
                        sources,
                        user_input
                            .get(metadata_pos)
                            .ok_or_else(|| ProcessingError::UserInput(metadata_pos as u8))?
                            .as_slice(),
                    )?,
            ),
            // VecObject can have object only as another VecObject
            ArgSrc::Object(_) => return Err(ProcessingError::Unreachable),
            ArgSrc::VecObject(id) => {
                result_args.push(Value::Null);

                bind_vec_obj_args(
                    source_metadata,
                    sources,
                    expressions,
                    user_input,
                    *id as usize,
                )?;
            }
            _ => result_args.push(get_value_from_source(
                source_metadata
                    .get(metadata_pos)
                    .ok_or(SourceError::InvalidArgId)?
                    .get(obj_arg_pos)
                    .ok_or(SourceError::InvalidArgId)?,
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
pub fn get_value_from_source<T: std::convert::AsRef<[Value]>>(
    arg_src: &ArgSrc,
    container: &ValueContainer<T>,
) -> Result<Value, SourceError> {
    match arg_src {
        ArgSrc::ConstsTpl(id) => {
            let value = container
                .tpl_consts
                .as_ref()
                .get(*id as usize)
                .ok_or(SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::ConstsSettings(id) => {
            let value = container
                .settings_consts
                .as_ref()
                .get(*id as usize)
                .ok_or(SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::ConstAction(id) => {
            let value = container
                .action_proposal_consts
                .ok_or(SourceError::SourceMissing)?
                .as_ref()
                .get(*id as usize)
                .ok_or(SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::ConstActivityShared(id) => {
            let value = container
                .activity_shared_consts
                .ok_or(SourceError::SourceMissing)?
                .as_ref()
                .get(*id as usize)
                .ok_or(SourceError::InvalidArgId)?
                .clone();
            Ok(value)
        }
        ArgSrc::Storage(key) => {
            let value = container
                .storage
                .as_ref()
                .ok_or(SourceError::SourceMissing)?
                .get_data(key)
                .ok_or(SourceError::InvalidArgId)?;
            Ok(value)
        }
        ArgSrc::GlobalStorage(key) => {
            let value = container
                .global_storage
                .get_data(key)
                .ok_or(SourceError::InvalidArgId)?;
            Ok(value)
        }
        ArgSrc::Const(const_id) => {
            Ok((container.dao_consts)(*const_id).ok_or(SourceError::InvalidArgId)?)
        }
        _ => Err(SourceError::InvalidSourceVariant),
    }
}
