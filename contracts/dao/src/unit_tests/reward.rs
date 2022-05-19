use data::workflow::basic::wf_add::WfAdd1;
use library::locking::{LockInput, UnlockMethod, UnlockPeriodInput, UnlockingDB, UnlockingInput};
use near_sdk::{testing_env, AccountId, ONE_NEAR};

use crate::{
    core::Contract,
    proposal::{Proposal, ProposalState},
    reward::{Reward, RewardType, RewardTypeIdent, RewardWage},
    treasury::{Asset, PartitionAsset, PartitionAssetInput, TreasuryPartition},
    unit_tests::{
        as_account_id, dummy_propose_settings, dummy_template_settings, get_context_builder,
        get_default_contract, get_role_id, ACC_1, ACC_2, FOUNDER_1, FOUNDER_2, FOUNDER_3,
    },
    wallet::{ClaimbleReward, Wallet, WithdrawStats},
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

fn claimable_rewards_sum(claimable_rewards: &[ClaimbleReward], asset: &Asset) -> u128 {
    let mut sum = 0;
    for reward in claimable_rewards.into_iter() {
        if reward.asset == *asset {
            sum += reward.amount.0
        }
    }
    sum
}

#[test]
fn reward_wage_one_asset() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset = Asset::Near;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![PartitionAsset::try_from(PartitionAssetInput {
            asset_id: reward_asset.clone(),
            unlocking: UnlockingInput {
                amount_init_unlock: 1000,
                lock: None,
            },
        })
        .unwrap()],
    };
    let role_id = get_role_id(&contract, 1, "leader");
    let partition_id = contract.partition_add(partition);
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
    let reward_id = contract.reward_add(reward);
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(2)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        1
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 1);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset);
    assert_eq!(wage_stats.timestamp_last_withdraw, 2);
    assert_eq!(wage_stats.amount, 1);
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        499
    );
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
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![PartitionAsset::try_from(PartitionAssetInput {
            asset_id: reward_asset.clone(),
            unlocking: UnlockingInput {
                amount_init_unlock: 1000,
                lock: None,
            },
        })
        .unwrap()],
    };
    let role_id = get_role_id(&contract, 1, "leader");
    let partition_id = contract.partition_add(partition);
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
    let reward_id = contract.reward_add(reward);
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
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
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );

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
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        100
    );
    contract.proposal_finish(proposal_id);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        200
    );
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
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
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
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset),
        0
    );
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
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![
            PartitionAsset::try_from(PartitionAssetInput {
                asset_id: reward_asset_1.clone(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 1000,
                    lock: None,
                },
            })
            .unwrap(),
            PartitionAsset::try_from(PartitionAssetInput {
                asset_id: reward_asset_2.clone(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 2000,
                    lock: None,
                },
            })
            .unwrap(),
            PartitionAsset::try_from(PartitionAssetInput {
                asset_id: reward_asset_3.clone(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 3000,
                    lock: None,
                },
            })
            .unwrap(),
        ],
    };
    let role_id = get_role_id(&contract, 1, "leader");
    let partition_id = contract.partition_add(partition);
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
    let reward_id_only_near = contract.reward_add(reward_only_near);
    let reward_id_only_fts = contract.reward_add(reward_only_fts);
    let reward_id_all_tokens = contract.reward_add(reward_all_tokens);
    assert!(reward_id_only_near == 1 && reward_id_only_fts == 2 && reward_id_all_tokens == 3);
    testing_env!(ctx.block_timestamp(tm(9)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        0
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        0
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        0
    );
    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        200
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        200
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        200
    );
    testing_env!(ctx.block_timestamp(tm(20)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        400
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        400
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        400
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 200);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        200
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        400
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        400
    );
    testing_env!(ctx.block_timestamp(tm(200)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        3_800
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        4_000
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        4_000
    );
    let withdraw_amount =
        contract.internal_withdraw_reward(&founder_1, vec![2, 3], &reward_asset_2);
    assert_eq!(withdraw_amount, 4_000);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        3_800
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        0
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        4_000
    );
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        19_800
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        16_000
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        20_000
    );
    testing_env!(ctx.block_timestamp(tm(2000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        19_800
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        16_000
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        20_000
    );
    let withdraw_amount =
        contract.internal_withdraw_reward(&founder_1, vec![2, 3], &reward_asset_3);
    assert_eq!(withdraw_amount, 20_000);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        19_800
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_2),
        16_000
    );
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_3),
        0
    );
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

/// Test case description:
/// 1. Dao creates new Partition with 1 NEAR unlocked and 5 NEAR locked with linear unlock.
/// 2. Dao creates new Reward(wage) referencing the created partition and reward 1 NEAR per second (valid 10 seconds).
/// 3. User withdraws rewards 1 NEAR.
/// 4. User waits for unlocking partition and then withdraw 5 unlocked NEAR.
/// 5. User waits for refill partition and withdraw rest remaing 4 NEAR.
#[test]
fn reward_one_asset_scenario() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset_1 = Asset::Near;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let lock_input = UnlockingInput {
        amount_init_unlock: 1,
        lock: Some(LockInput {
            amount_total_lock: 5,
            start_from: 5,
            duration: 5,
            periods: vec![UnlockPeriodInput {
                r#type: UnlockMethod::Linear,
                duration: 5,
                amount: 5,
            }],
        }),
    };
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![PartitionAsset::try_from(PartitionAssetInput {
            asset_id: reward_asset_1.clone(),
            unlocking: lock_input,
        })
        .unwrap()],
    };
    let partition_id = contract.partition_add(partition);
    let mut partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1 * ONE_NEAR);
    partition.unlock_all(0);
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1 * ONE_NEAR);
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
    let reward_id = contract.reward_add(reward);
    testing_env!(ctx.block_timestamp(tm(0)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        1 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 1 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        0
    );
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 0);
    testing_env!(ctx.block_timestamp(tm(5)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        4 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        9 * ONE_NEAR
    );

    // Unlock all
    let mut partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 0);
    testing_env!(ctx.block_timestamp(tm(20)).build());
    partition.unlock_all(11);
    contract
        .treasury_partition
        .insert(&partition_id, &partition.into());
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(&reward_asset_1)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 5 * ONE_NEAR);
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 5 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        4 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(420)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        4 * ONE_NEAR
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

    // Refill 100 NEARs
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
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        4 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 4 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        0
    );
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
        100 * ONE_NEAR - 4 * ONE_NEAR
    );
    testing_env!(ctx.block_timestamp(tm(9999)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(claimable_rewards.as_slice(), &reward_asset_1),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset_1);
    assert_eq!(withdraw_amount, 0);
}
