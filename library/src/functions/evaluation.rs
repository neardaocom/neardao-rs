use crate::types::datatype::Value;
use crate::types::error::ProcessingError;
use crate::workflow::types::{Src, ValueSrc};
use crate::{
    interpreter::expression::EExpr,
    types::{activity_input::ActivityInput, source::Source},
};

use super::utils::get_value_from_source;

/// Evaluate to Value according to `src` definition.
pub fn eval(
    src: &ValueSrc,
    sources: &dyn Source,
    expressions: &[EExpr],
    input: Option<&dyn ActivityInput>,
) -> Result<Value, ProcessingError> {
    match src {
        ValueSrc::Src(arg_src) => match arg_src {
            Src::Input(key) => {
                if let Some(input) = input {
                    Ok(input
                        .get(key)
                        .ok_or(ProcessingError::MissingInputKey(key.into()))?
                        .clone())
                } else {
                    return Err(ProcessingError::InputNotProvided);
                }
            }
            _ => Ok(get_value_from_source(sources, arg_src)?),
        },
        ValueSrc::Expr(expr) => Ok(expr.bind_and_eval(sources, input, expressions)?),
        ValueSrc::Value(v) => Ok(v.clone()),
    }
}
