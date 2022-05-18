use near_sdk::{testing_env, ONE_NEAR};

use crate::{
    treasury::{Asset, TreasuryPartition},
    unit_tests::{
        as_account_id, get_context_builder, get_default_contract, TOKEN_TOTAL_SUPPLY,
        VOTE_TOKEN_ACC,
    },
};

#[test]
fn treasury_default_dao() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let asset_near = Asset::new_near();
    let asset_ft = Asset::new_ft(as_account_id(VOTE_TOKEN_ACC), 24);
    let mut partition_near: TreasuryPartition = contract.treasury_partition.get(&1).unwrap().into();
    let mut partition_vote_token: TreasuryPartition =
        contract.treasury_partition.get(&2).unwrap().into();
    let asset_1 = partition_near.asset(&asset_near).unwrap();
    let asset_2 = partition_vote_token.asset(&asset_ft).unwrap();
    assert_eq!(asset_1.asset_id(), &asset_near);
    assert_eq!(asset_1.available_amount(), 100 * ONE_NEAR);
    assert_eq!(asset_2.asset_id(), &asset_ft);
    assert_eq!(asset_2.available_amount(), 0);
    partition_near.unlock_all(100);
    partition_vote_token.unlock_all(100);
    let asset_1 = partition_near.asset(&asset_near).unwrap();
    let asset_2 = partition_vote_token.asset(&asset_ft).unwrap();
    assert_eq!(asset_1.available_amount(), 100 * ONE_NEAR);
    assert_eq!(
        asset_2.available_amount(),
        TOKEN_TOTAL_SUPPLY as u128 * ONE_NEAR / 10
    );
    partition_near.unlock_all(1000);
    partition_vote_token.unlock_all(1000);
    let asset_1 = partition_near.asset(&asset_near).unwrap();
    let asset_2 = partition_vote_token.asset(&asset_ft).unwrap();
    assert_eq!(asset_1.available_amount(), 100 * ONE_NEAR);
    assert_eq!(
        asset_2.available_amount(),
        TOKEN_TOTAL_SUPPLY as u128 * ONE_NEAR
    );
    partition_near.unlock_all(2000);
    partition_vote_token.unlock_all(2000);
    let asset_1 = partition_near.asset(&asset_near).unwrap();
    let asset_2 = partition_vote_token.asset(&asset_ft).unwrap();
    assert_eq!(asset_1.available_amount(), 100 * ONE_NEAR);
    assert_eq!(
        asset_2.available_amount(),
        TOKEN_TOTAL_SUPPLY as u128 * ONE_NEAR
    );
}

#[test]
fn treasury_remove_more_than_available() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let asset_near = Asset::new_near();
    let mut partition_near: TreasuryPartition = contract.treasury_partition.get(&1).unwrap().into();
    let asset_1 = partition_near.asset(&asset_near).unwrap();
    assert_eq!(asset_1.available_amount(), 100 * ONE_NEAR);
    let amount_removed = partition_near.remove_amount(&asset_near, 0, 1_000_000 * ONE_NEAR);
    assert_eq!(amount_removed, 100 * ONE_NEAR);
    let asset_1 = partition_near.asset(&asset_near).unwrap();
    assert_eq!(asset_1.available_amount(), 0);
}
