use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

use crate::{
    functions::utils::object_key,
    interpreter::expression::EExpr,
    types::{activity_input::ActivityInput, error::ProcessingError, source::Source},
};

use super::types::ArgSrc;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Validator {
    Object(ObjectValidator),
    Collection(CollectionValidator),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct ObjectValidator {
    /// Id of expression "doing" the validation.
    pub expression_id: u8,
    /// Sources for keys being validated.
    pub key_src: Vec<ArgSrc>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct CollectionValidator {
    /// Id of expression "doing" the validation.
    pub expression_id: u8,
    /// Sources for keys being validated.
    pub key_src: Vec<ArgSrc>,
    /// Prefixes for nested collection objects.
    /// Defined as Vec<String> for forward-compatible changes.
    pub prefixes: Vec<String>,
}

impl Validator {
    pub fn validate<S, A>(
        &self,
        sources: &S,
        args: &A,
        expressions: &[EExpr],
    ) -> Result<bool, ProcessingError>
    where
        S: Source + ?Sized,
        A: ActivityInput + ?Sized,
    {
        let expr = expressions
            .get(self.get_expression_id() as usize)
            .expect("Validator expression missing");
        let mut binded_args = Vec::with_capacity(8);
        let result = match self {
            Validator::Object(o) => {
                for src in o.key_src.iter() {
                    match src {
                        ArgSrc::User(key) => {
                            binded_args.push(
                                args.get(key.as_str())
                                    .expect("Failed to get user value")
                                    .clone(),
                            );
                        }
                        ArgSrc::ConstsTpl(key) => {
                            binded_args.push(
                                sources
                                    .tpl(key.as_str())
                                    .expect("Failed to get tpl value")
                                    .clone(),
                            );
                        }
                        ArgSrc::ConstsSettings(_) => todo!(),
                        ArgSrc::ConstActivityShared(_) => todo!(),
                        ArgSrc::ConstAction(_) => todo!(),
                        ArgSrc::Storage(_) => todo!(),
                        ArgSrc::GlobalStorage(_) => todo!(),
                        //ArgSrcNew::Expression(_) => todo!(),
                        ArgSrc::Const(_) => todo!(),
                    }
                }
                expr.eval(binded_args.as_slice())?.try_into_bool()?
            }
            Validator::Collection(o) => {
                let mut counter: u32 = 0;
                let mut started_new = false;
                loop {
                    for src in o.key_src.iter() {
                        match src {
                            ArgSrc::User(key_suffix) => {
                                let key = object_key(
                                    o.prefixes.get(0).expect("No prefix for collection"),
                                    counter.to_string().as_str(),
                                    key_suffix.as_str(),
                                );
                                if let Some(v) = args.get(key.as_str()) {
                                    binded_args.push(v.clone());
                                    started_new = true;
                                } else {
                                    break;
                                };
                            }
                            ArgSrc::ConstsTpl(key) => {
                                if let Some(v) = sources.tpl(key.as_str()) {
                                    binded_args.push(v.clone());
                                } else {
                                    return Err(ProcessingError::InvalidValidatorDefinition);
                                    // TODO: add other variant
                                }
                            }
                            ArgSrc::ConstsSettings(_) => todo!(),
                            ArgSrc::ConstActivityShared(_) => todo!(),
                            ArgSrc::ConstAction(_) => todo!(),
                            ArgSrc::Storage(_) => todo!(),
                            ArgSrc::GlobalStorage(_) => todo!(),
                            //ArgSrcNew::Expression(_) => todo!(),
                            ArgSrc::Const(_) => todo!(),
                        }
                    }
                    // Return true only if all object attributes have been validated.
                    if !started_new {
                        break true;
                    } else if binded_args.len() < o.key_src.len() {
                        return Err(ProcessingError::InvalidValidatorDefinition);
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
