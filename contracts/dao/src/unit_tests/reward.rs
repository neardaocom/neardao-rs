use data::workflow::basic::wf_add::WfAdd1;
use near_sdk::{testing_env, AccountId, ONE_NEAR};

use crate::{
    core::Contract,
    proposal::{Proposal, ProposalState},
    reward::{Reward, RewardType, RewardTypeIdent, RewardWage},
    treasury::{Asset, PartitionAsset, TreasuryPartition},
    unit_tests::{
        as_account_id, dummy_propose_settings, dummy_template_settings, get_context_builder,
        get_default_contract, get_role_id, ACC_1, ACC_2, FOUNDER_1, FOUNDER_2, FOUNDER_3,
    },
    wallet::{Wallet, WithdrawStats},
};

/// Convert timestamp seconds to miliseconds
/// Contract internally works with seconds.
fn tm(v: u64) -> u64 {
    v * 10u64.pow(9)
}

fn get_wallet(contract: &Contract, account_id: &AccountId) -> Wallet {
    let wallet: Wallet = contract
        .wallets
        .get(&account_id)
        .expect("wallet not found")
        .into();
    wallet
}

fn get_wallet_withdraw_stat<'a>(
    wallet: &'a Wallet,
    reward_id: u16,
    asset: &Asset,
) -> &'a WithdrawStats {
    let wallet_reward = wallet
        .wallet_reward(reward_id)
        .expect("wallet reward nout found");
    wallet_reward.withdraw_stat(asset)
}

#[test]
fn reward_wage_one_asset() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset = Asset::Near;
    let asset = reward_asset.to_string();
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        assets: vec![PartitionAsset::new(reward_asset.clone(), 1000, None, 0)],
    };
    let role_id = get_role_id(&contract, 1, "leader");
    let partition_id = contract.add_partition(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let reward = Reward::new(
        1,
        role_id,
        partition_id,
        RewardType::new_wage(2),
        vec![(reward_asset.clone(), 1)],
        0,
        1000,
    );
    let reward_id = contract.add_reward(reward);
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(2)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&1));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 1);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset);
    assert_eq!(wage_stats.timestamp_last_withdraw, 2);
    assert_eq!(wage_stats.amount, 1);
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&499));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 499);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert_eq!(wage_stats.timestamp_last_withdraw, 1000);
    assert_eq!(wage_stats.amount, 500);
    testing_env!(ctx.block_timestamp(tm(2000)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert_eq!(wage_stats.timestamp_last_withdraw, 1000);
    assert_eq!(wage_stats.amount, 500);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 500);
}

#[test]
fn reward_activity_one_asset() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset = Asset::Near;
    let asset = reward_asset.to_string();
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        assets: vec![PartitionAsset::new(reward_asset.clone(), 1000, None, 0)],
    };
    let role_id = get_role_id(&contract, 1, "leader");
    let partition_id = contract.add_partition(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let reward = Reward::new(
        1,
        role_id,
        partition_id,
        RewardType::new_user_activity(vec![0, 1]),
        vec![(reward_asset.clone(), 100)],
        0,
        4000,
    );
    let reward_id = contract.add_reward(reward);
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(2)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert!(activity_stats.asset_id == reward_asset);
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    testing_env!(ctx.block_timestamp(tm(2000)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));

    // Founder_1 adds new proposal and votes for it to get rewards for activities.
    testing_env!(ctx
        .predecessor_account_id(founder_1.clone())
        .attached_deposit(ONE_NEAR)
        .build());
    let proposal_id = contract.proposal_create(
        None,
        1,
        0,
        dummy_propose_settings(),
        Some(vec![dummy_template_settings()]),
        None,
    );
    testing_env!(ctx
        .predecessor_account_id(founder_1.clone())
        .attached_deposit(1)
        .build());
    contract.proposal_vote(proposal_id, 1);
    testing_env!(ctx
        .predecessor_account_id(founder_2.clone())
        .attached_deposit(1)
        .build());
    contract.proposal_vote(proposal_id, 1);
    testing_env!(ctx
        .predecessor_account_id(founder_3.clone())
        .attached_deposit(1)
        .build());
    contract.proposal_vote(proposal_id, 1);
    testing_env!(ctx
        .block_timestamp(tm(3000))
        .predecessor_account_id(founder_2.clone())
        .build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&100));
    contract.proposal_finish(proposal_id);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&200));
    let proposal: Proposal = contract.proposals.get(&proposal_id).unwrap().into();
    assert!(proposal.state == ProposalState::Accepted);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 2);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 200);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 3000);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 2);
    testing_env!(ctx.block_timestamp(tm(3500)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset.clone()), Some(&0));
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 3000);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 2);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 200);
}

#[test]
fn reward_multiple_wage_rewards() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let ft_account_id_1 = as_account_id(ACC_1);
    let ft_account_id_2 = as_account_id(ACC_2);
    let reward_asset_1 = Asset::Near;
    let reward_asset_2 = Asset::new_ft(ft_account_id_1.clone(), 24);
    let reward_asset_3 = Asset::new_ft(ft_account_id_2.clone(), 24);
    let asset_1 = reward_asset_1.to_string();
    let asset_2 = reward_asset_2.to_string();
    let asset_3 = reward_asset_3.to_string();
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        assets: vec![
            PartitionAsset::new(reward_asset_1.clone(), 1000, None, 0),
            PartitionAsset::new(reward_asset_2.clone(), 2000, None, 0),
            PartitionAsset::new(reward_asset_3.clone(), 3000, None, 0),
        ],
    };
    let role_id = get_role_id(&contract, 1, "leader");
    let partition_id = contract.add_partition(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset_1 = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    let partition_asset_2 = partition
        .asset(&reward_asset_2)
        .expect("partition asset not found");
    let partition_asset_3 = partition
        .asset(&reward_asset_3)
        .expect("partition asset not found");
    assert_eq!(partition_asset_1.available_amount(), 1000 * ONE_NEAR);
    assert_eq!(partition_asset_2.available_amount(), 2000 * ONE_NEAR);
    assert_eq!(partition_asset_3.available_amount(), 3000 * ONE_NEAR);
    let reward_only_near = Reward::new(
        1,
        role_id,
        partition_id,
        RewardType::new_wage(10),
        vec![(reward_asset_1.clone(), 100)],
        0,
        1000,
    );
    let reward_only_fts = Reward::new(
        1,
        role_id,
        partition_id,
        RewardType::new_wage(10),
        vec![(reward_asset_2.clone(), 100), (reward_asset_3.clone(), 100)],
        0,
        1000,
    );
    let reward_all_tokens = Reward::new(
        1,
        role_id,
        partition_id,
        RewardType::new_wage(10),
        vec![
            (reward_asset_1.clone(), 100),
            (reward_asset_2.clone(), 100),
            (reward_asset_3.clone(), 100),
        ],
        0,
        1000,
    );
    let reward_id_only_near = contract.add_reward(reward_only_near);
    let reward_id_only_fts = contract.add_reward(reward_only_fts);
    let reward_id_all_tokens = contract.add_reward(reward_all_tokens);
    assert!(reward_id_only_near == 1 && reward_id_only_fts == 2 && reward_id_all_tokens == 3);
    testing_env!(ctx.block_timestamp(tm(9)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1), Some(&0));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&0));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&0));
    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&200));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&200));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&200));
    testing_env!(ctx.block_timestamp(tm(20)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&400));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&400));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&400));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 200);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&200));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&400));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&400));
    testing_env!(ctx.block_timestamp(tm(200)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&3_800));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&4_000));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&4_000));
    let withdraw_amount =
        contract.internal_withdraw_reward(&founder_1, vec![2, 3], &reward_asset_2);
    assert_eq!(withdraw_amount, 4_000);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&3_800));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&0));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&4_000));
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&19_800));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&16_000));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&20_000));
    testing_env!(ctx.block_timestamp(tm(2000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&19_800));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&16_000));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&20_000));
    let withdraw_amount =
        contract.internal_withdraw_reward(&founder_1, vec![2, 3], &reward_asset_3);
    assert_eq!(withdraw_amount, 20_000);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&19_800));
    assert_eq!(claimable_rewards.get(&asset_2.clone()), Some(&16_000));
    assert_eq!(claimable_rewards.get(&asset_3.clone()), Some(&0));
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 200);
    let partition_asset = partition
        .asset(&reward_asset_2)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 2000 * ONE_NEAR - 4_000);
    let partition_asset = partition
        .asset(&reward_asset_3)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 3000 * ONE_NEAR - 20_000);
}

#[test]
fn reward_treasury_is_missing_asset_and_later_refill() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset_1 = Asset::Near;
    let asset_1 = reward_asset_1.to_string();
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        assets: vec![PartitionAsset::new(reward_asset_1.clone(), 1, None, 0)],
    };
    let partition_id = contract.add_partition(partition);
    let role_id = get_role_id(&contract, 1, "leader");
    let reward = Reward::new(
        1,
        role_id,
        partition_id,
        RewardType::new_wage(1),
        vec![(reward_asset_1.clone(), 1 * ONE_NEAR)],
        0,
        10,
    );
    let reward_id = contract.add_reward(reward);

    testing_env!(ctx.block_timestamp(tm(0)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&0));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards.get(&asset_1.clone()),
        Some(&(1 * ONE_NEAR))
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 1 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&0));
    testing_env!(ctx.block_timestamp(tm(5)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards.get(&asset_1.clone()),
        Some(&(4 * ONE_NEAR))
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards.get(&asset_1.clone()),
        Some(&(9 * ONE_NEAR))
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(420)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards.get(&asset_1.clone()),
        Some(&(9 * ONE_NEAR))
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(500)).build());
    let mut partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 0);
    partition.add_amount(&reward_asset_1, 100 * ONE_NEAR);
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 100 * ONE_NEAR);
    contract
        .treasury_partition
        .insert(&partition_id, &partition.into());

    testing_env!(ctx.block_timestamp(tm(500)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards.get(&asset_1.clone()),
        Some(&(9 * ONE_NEAR))
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 9 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&0));
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(
        partition_asset.available_amount(),
        100 * ONE_NEAR - 9 * ONE_NEAR
    );

    testing_env!(ctx.block_timestamp(tm(9999)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(claimable_rewards.get(&asset_1.clone()), Some(&0));
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
}
