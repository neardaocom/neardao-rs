use crate::{
    interpreter::expression::EExpr,
    types::{activity_input::ActivityInput, error::ProcessingError, source::Source},
    workflow::validator::Validator,
};

/// Validates inputs by validator_exprs.
pub fn validate<S, A>(
    sources: &S,
    validators: &[Validator],
    expressions: &[EExpr],
    user_input: &A,
) -> Result<bool, ProcessingError>
where
    S: Source + ?Sized,
    A: ActivityInput + ?Sized,
{
    for validator in validators.iter() {
        if !validator.validate(sources, user_input, expressions)? {
            return Ok(false);
        }
    }

    Ok(true)
}
