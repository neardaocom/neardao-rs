use crate::{
    types::{
        activity_input::ActivityInput,
        datatype::Value,
        error::{ProcessingError, SourceError},
        source::Source,
    },
    workflow::{
        expression::{Expression, ExpressionNew},
        types::{FnCallMetadata, ValidatorRef, ValidatorType, ValueContainer},
    },
};

/// Validates inputs by validator_exprs.
/// Returns `Err(())` in case input/Wf structure is bad.
pub fn validate<T: std::convert::AsRef<[Value]>>(
    sources: &ValueContainer<T>,
    validator_refs: &[ValidatorRef],
    validator_exprs: &[Expression],
    metadata: &[FnCallMetadata],
    user_input: &[Vec<Value>],
) -> Result<bool, ProcessingError> {
    for v_ref in validator_refs.iter() {
        let validator = validator_exprs
            .get(v_ref.val_id as usize)
            .ok_or(SourceError::InvalidArgId)?;
        let inputs: &[Value] = &user_input[v_ref.obj_id as usize];

        match v_ref.v_type {
            ValidatorType::Simple => {
                let inputs: &[Value] = user_input
                    .get(v_ref.obj_id as usize)
                    .ok_or(ProcessingError::UserInput(v_ref.obj_id))?;
                if !validator.bind_and_eval(sources, inputs)?.try_into_bool()? {
                    return Ok(false);
                }
            }
            //TODO validate by pos??
            ValidatorType::Collection => {
                let obj_len = metadata
                    .get(v_ref.obj_id as usize)
                    .ok_or(SourceError::InvalidArgId)?
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

pub fn validate_new<S, A>(
    source: &S,
    validator_refs: &[ValidatorRef],
    validator_exprs: &[ExpressionNew],
    //metadata: &[FnCallMetadata],
    user_input: &A,
) -> Result<bool, ProcessingError>
where
    S: Source + ?Sized,
    A: ActivityInput + ?Sized,
{
    for v_ref in validator_refs.iter() {
        let validator = validator_exprs
            .get(v_ref.val_id as usize)
            .ok_or(SourceError::InvalidArgId)?;

        match v_ref.v_type {
            ValidatorType::Simple => {
                if !validator
                    .bind_and_eval(source, user_input)?
                    .try_into_bool()?
                {
                    return Ok(false);
                }
            }
            ValidatorType::Collection => todo!(),
        }
    }

    Ok(true)
}
