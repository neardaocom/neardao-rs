use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{
    functions::{evaluation::eval, utils::object_key},
    interpreter::expression::EExpr,
    types::{
        activity_input::ActivityInput,
        error::{ProcessingError, ValidationError},
        source::Source,
    },
};

use super::types::{Src, ValueSrc};

// TODO: Remove all Debug in production!

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum Validator {
    Object(ObjectValidator),
    Collection(CollectionValidator),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ObjectValidator {
    /// Id of expression "doing" the validation.
    pub expression_id: u8,
    /// Sources for keys being validated.
    pub value: Vec<ValueSrc>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct CollectionValidator {
    /// Id of expression "doing" the validation.
    pub expression_id: u8,
    /// Sources for keys being validated.
    pub value: Vec<ValueSrc>,
    /// Prefixes for nested collection objects.
    /// Defined as Vec<String> for forward-compatible changes.
    pub prefixes: Vec<String>,
}

impl Validator {
    pub fn validate(
        &self,
        sources: &dyn Source,
        expressions: &[EExpr],
        input: &dyn ActivityInput,
    ) -> Result<bool, ProcessingError> {
        let expr = expressions
            .get(self.get_expression_id() as usize)
            .ok_or(ProcessingError::MissingExpression(self.get_expression_id()))?;
        let mut binded_args = Vec::with_capacity(8);
        let result = match self {
            Validator::Object(o) => {
                for src in o.value.iter() {
                    let value = eval(src, sources, expressions, Some(input))?;
                    binded_args.push(value);
                }
                expr.eval(binded_args.as_slice())?.try_into_bool()?
            }
            Validator::Collection(o) => {
                let mut counter: u32 = 0;
                let mut started_new = false;
                loop {
                    for src in o.value.iter() {
                        let mapped_src = if let ValueSrc::Src(src) = src {
                            if let Src::User(key_suffix) = src {
                                let key = object_key(
                                    o.prefixes
                                        .get(0)
                                        .ok_or(ValidationError::MissingKeyPrefix(0))?,
                                    counter.to_string().as_str(),
                                    key_suffix.as_str(),
                                );
                                Some(ValueSrc::Src(src.with_new_user_key(key).unwrap()))
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        let value = eval(
                            mapped_src.as_ref().unwrap_or(src),
                            sources,
                            expressions,
                            Some(input),
                        )?;
                        binded_args.push(value);
                    }
                    // Return true only if all object attributes have been validated.
                    if !started_new {
                        break true;
                    } else if binded_args.len() < o.value.len() {
                        return Err(ProcessingError::Validation(
                            ValidationError::InvalidDefinition,
                        ));
                    // TODO: add other variant
                    } else {
                        counter += 1;
                        if !expr.eval(binded_args.as_slice())?.try_into_bool()? {
                            break false;
                        }
                        binded_args.clear();
                        started_new = false;
                    }
                }
            }
        };
        Ok(result)
    }

    fn get_expression_id(&self) -> u8 {
        match self {
            Validator::Object(o) => o.expression_id,
            Validator::Collection(o) => o.expression_id,
        }
    }
}
