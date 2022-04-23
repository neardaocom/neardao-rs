use crate::{
    interpreter::expression::EExpr,
    types::{
        activity_input::ActivityInput,
        datatype::Value,
        error::{ProcessingError, SourceError},
        source::Source,
    },
    workflow::types::{
        ArgSrc, BindDefinition,
        SrcOrExpr::{Expr, Src},
    },
};

use super::utils::object_key;

pub fn bind_from_sources<S, A>(
    sources: &S,
    bind_definitions: &[BindDefinition],
    expressions: &[EExpr],
    inputs: &mut A,
) -> Result<(), ProcessingError>
where
    S: Source + ?Sized,
    A: ActivityInput + ?Sized,
{
    for def in bind_definitions.iter() {
        match def.is_collection {
            false => {
                let value = match &def.key_src {
                    Src(arg_src) => match arg_src {
                        ArgSrc::ConstsTpl(ref key) => sources
                            .tpl(key.as_str())
                            .expect("Failed to get tpl value for binding")
                            .clone(),
                        ArgSrc::User(_) => continue,
                        _ => todo!(),
                    },
                    Expr(expr) => expr.bind_and_eval(sources, inputs, expressions)?,
                };
                inputs.set(def.key.as_str(), value);
            }
            true => {
                let prefix = def.prefixes.get(0).expect("Prefix 0 not found").as_str();

                let value = match &def.key_src {
                    Src(arg_src) => match arg_src {
                        ArgSrc::ConstsTpl(ref key) => sources
                            .tpl(key.as_str())
                            .expect("Failed to get tpl value for binding")
                            .clone(),
                        ArgSrc::User(_) => continue,
                        _ => todo!(),
                    },
                    Expr(expr) => expr.bind_and_eval(sources, inputs, expressions)?,
                };
                let mut counter: u32 = 0;
                let mut key = object_key(prefix, counter.to_string().as_str(), def.key.as_str());
                while inputs.has_key(key.as_str()) {
                    inputs.set(key.as_str(), value.clone());
                    counter += 1;
                    key = object_key(prefix, counter.to_string().as_str(), def.key.as_str());
                }
            }
        }
    }

    Ok(())
}

/// Helper function to fetch value ref from Source.
pub fn get_value_from_source<'a, S>(sources: &'a S, src: &ArgSrc) -> Result<&'a Value, SourceError>
where
    S: Source + ?Sized,
{
    match src {
        ArgSrc::ConstsTpl(key) => {
            let value = sources.tpl(key).ok_or(SourceError::SourceMissing)?;
            Ok(value)
        }
        ArgSrc::ConstsSettings(key) => {
            let value = sources
                .tpl_settings(key)
                .ok_or(SourceError::SourceMissing)?;
            Ok(value)
        }
        ArgSrc::ConstAction(_key) => {
            unimplemented!();
        }
        ArgSrc::ConstActivityShared(_key) => {
            unimplemented!();
        }
        ArgSrc::Storage(key) => {
            let value = sources.storage(key).ok_or(SourceError::SourceMissing)?;
            Ok(value)
        }
        ArgSrc::GlobalStorage(key) => {
            let value = sources
                .global_storage(key)
                .ok_or(SourceError::SourceMissing)?;
            Ok(value)
        }
        ArgSrc::Const(key) => {
            let value = sources.dao_const(*key).ok_or(SourceError::SourceMissing)?;
            Ok(value)
        }
        _ => Err(SourceError::InvalidSourceVariant),
    }
}
