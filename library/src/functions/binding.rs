use crate::workflow::types::CollectionBindingStyle::{ForceSame, Overwrite};
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
        match &def.collection_data {
            None => {
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
            Some(data) => {
                // Version 1.x does not supports nested collections.
                let prefix = data
                    .prefixes
                    .get(0)
                    .expect("At least 0 prefix must be defined for a collection");
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
                match data.collection_binding_type {
                    Overwrite => {
                        while input.has_key(key.as_str()) {
                            input.set(key.as_str(), value.clone());
                            counter += 1;
                            key =
                                object_key(prefix, counter.to_string().as_str(), def.key.as_str());
                        }
                    }
                    ForceSame(number) => {
                        for _ in 0..number as usize {
                            input.set(key.as_str(), value.clone());
                            counter += 1;
                            key =
                                object_key(prefix, counter.to_string().as_str(), def.key.as_str());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
