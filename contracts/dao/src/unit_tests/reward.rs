use data::workflow::basic::wf_add::WfAdd1;
use near_sdk::{testing_env, AccountId, ONE_NEAR};

use crate::{
    core::Contract,
    proposal::{Proposal, ProposalState},
    reward::{Reward, RewardType, RewardTypeIdent, RewardWage},
    treasury::{Asset, PartitionAsset, TreasuryPartition},
    unit_tests::{
        as_account_id, dummy_propose_settings, dummy_template_settings, get_context_builder,
        get_default_contract, FOUNDER_1, FOUNDER_2, FOUNDER_3,
    },
    wallet::{Wallet, WithdrawStats},
};

/// Convert timestamp seconds to miliseconds
/// Contract internally works with seconds.
fn tm(v: u64) -> u64 {
    v * 10u64.pow(9)
}

fn get_role_id(contract: &Contract, group_id: u16, role_name: &str) -> u16 {
    let group_roles = contract
        .group_roles
        .get(&group_id)
        .expect("group not found");
    let role_id = group_roles
        .iter()
        .find(|(key, name)| name.as_str() == role_name)
        .expect("role not found");
    *role_id.0
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
    let partition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found");
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
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(2)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 1);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset);
    assert_eq!(wage_stats.timestamp_last_withdraw, 2);
    assert_eq!(wage_stats.amount, 1);
    testing_env!(ctx.block_timestamp(tm(1000)).build());
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
    let partition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found");
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
        assets: vec![PartitionAsset::new(reward_asset.clone(), 1000, None, 0)],
    };
    let role_id = get_role_id(&contract, 1, "leader");
    let partition_id = contract.add_partition(partition);
    let partition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found");
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
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
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
    let partition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found");
    let partition_asset = partition
        .asset(&reward_asset)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);

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
    contract.proposal_finish(proposal_id);
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
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 3000);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 2);
    testing_env!(ctx.block_timestamp(tm(3500)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], &reward_asset);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, &reward_asset);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 3000);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 2);
    let partition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found");
    let partition_asset = partition
        .asset(&reward_asset)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 200);
}
