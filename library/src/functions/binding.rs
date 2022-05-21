use crate::workflow::types::CollectionBindingStyle::{ForceSame, Overwrite};
use crate::{
    interpreter::expression::EExpr,
    types::{activity_input::ActivityInput, error::ProcessingError, source::Source},
    workflow::types::BindDefinition,
};

use super::evaluation::eval;
use super::utils::object_key;

// TODO: Error handling.
pub fn bind_input(
    sources: &dyn Source,
    bind_definitions: &[BindDefinition],
    expressions: &[EExpr],
    input: &mut dyn ActivityInput,
) -> Result<(), ProcessingError> {
    for def in bind_definitions.iter() {
        match &def.collection_data {
            None => {
                let value =
                    eval(&def.key_src, sources, expressions, Some(input)).expect("value not found");
                input.set(def.key.as_str(), value);
            }
            Some(data) => {
                // Version 1.0 does not support nested collections.
                let prefix = data
                    .prefixes
                    .get(0)
                    .expect("At least 0 prefix must be defined for a collection");
                let value =
                    eval(&def.key_src, sources, expressions, Some(input)).expect("value not found");
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
