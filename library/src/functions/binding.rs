use crate::workflow::types::SrcOrExprOrValue::{Expr, Src, Value};
use crate::{
    interpreter::expression::EExpr,
    types::{activity_input::ActivityInput, error::ProcessingError, source::Source},
    workflow::types::{ArgSrc, BindDefinition},
};

use super::utils::{get_value_from_source, object_key};

// TODO: Replace panic
pub fn bind_input(
    sources: &dyn Source,
    bind_definitions: &[BindDefinition],
    expressions: &[EExpr],
    input: &mut dyn ActivityInput,
) -> Result<(), ProcessingError>
where
{
    for def in bind_definitions.iter() {
        match def.is_collection {
            false => {
                let value = match &def.key_src {
                    Src(arg_src) => match arg_src {
                        ArgSrc::User(_) => continue,
                        _ => get_value_from_source(sources, arg_src)?,
                    },
                    Expr(expr) => expr.bind_and_eval(sources, Some(input), expressions)?,
                    Value(v) => v.clone(),
                };
                input.set(def.key.as_str(), value);
            }
            true => {
                // At this version we support only one collection in the whole object.
                // Nested collection are not supported yet.
                let prefix = def.prefixes.get(0).expect("Prefix 0 not found").as_str();

                let value = match &def.key_src {
                    Src(arg_src) => match arg_src {
                        ArgSrc::User(_) => continue,
                        _ => get_value_from_source(sources, arg_src)?,
                    },
                    Expr(expr) => expr.bind_and_eval(sources, Some(input), expressions)?,
                    Value(v) => v.clone(),
                };
                let mut counter: u32 = 0;
                let mut key = object_key(prefix, counter.to_string().as_str(), def.key.as_str());
                while input.has_key(key.as_str()) {
                    input.set(key.as_str(), value.clone());
                    counter += 1;
                    key = object_key(prefix, counter.to_string().as_str(), def.key.as_str());
                }
            }
        }
    }

    Ok(())
}
