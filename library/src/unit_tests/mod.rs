#![cfg(test)]
#![allow(unused)]
use crate::workflow::DataType;

mod bounty;
mod skyward;

pub fn get_dao_consts() -> Box<dyn Fn(u8) -> DataType> {
    Box::new(|id: u8| match id {
        0 => DataType::String("neardao.testnet".into()),
        _ => unimplemented!(),
    })
}

pub const ONE_NEAR: u128 = 10u128.pow(24);
