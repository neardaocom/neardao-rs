use crate::types::datatype::Value;
use crate::workflow::types::ValueSrc;
use crate::{
    interpreter::expression::EExpr,
    types::{activity_input::ActivityInput, source::Source},
    workflow::types::ArgSrc,
};

use super::utils::get_value_from_source;

// TODO: Error handling.
pub fn eval(
    src: &ValueSrc,
    sources: &dyn Source,
    expressions: &[EExpr],
    input: Option<&dyn ActivityInput>,
) -> Option<Value> {
    match src {
        ValueSrc::Src(arg_src) => match arg_src {
            ArgSrc::User(key) => {
                if let Some(input) = input {
                    Some(input.get(key).expect("Failed to get value").clone())
                } else {
                    panic!("eval user input not provided")
                }
            }
            _ => Some(get_value_from_source(sources, arg_src).expect("eval source error")),
        },
        ValueSrc::Expr(expr) => Some(
            expr.bind_and_eval(sources, input, expressions)
                .expect("eval expr error"),
        ),
        ValueSrc::Value(v) => Some(v.clone()),
    }
}
