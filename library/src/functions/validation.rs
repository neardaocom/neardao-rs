use crate::{
    interpreter::expression::EExpr,
    types::{activity_input::ActivityInput, error::ProcessingError, source::Source},
    workflow::validator::Validator,
};

/// Validate `user_input` according to `validators` definition.
pub fn validate(
    sources: &dyn Source,
    validators: &[Validator],
    expressions: &[EExpr],
    user_input: &dyn ActivityInput,
) -> Result<bool, ProcessingError> {
    for validator in validators.iter() {
        if !validator.validate(sources, expressions, user_input)? {
            return Ok(false);
        }
    }
    Ok(true)
}
