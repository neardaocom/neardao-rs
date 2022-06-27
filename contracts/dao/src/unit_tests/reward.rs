use data::workflow::basic::basic_package::WfBasicPkg1;
use library::{
    locking::{LockInput, UnlockMethod, UnlockPeriodInput, UnlockingDB, UnlockingInput},
    workflow::types::ActivityRight,
};
use near_sdk::{test_utils::accounts, testing_env, AccountId, ONE_NEAR};

use crate::{
    constants::LATEST_REWARD_ACTIVITY_ID,
    contract::Contract,
    proposal::{Proposal, ProposalState},
    reward::{Reward, RewardType, RewardTypeIdent, RewardWage},
    treasury::{Asset, AssetRegistrar, PartitionAsset, PartitionAssetInput, TreasuryPartition},
    unit_tests::{
        as_account_id, assert_cache_reward_activity, claimable_rewards_sum, dummy_propose_settings,
        dummy_template_settings, get_context_builder, get_default_contract, get_role_id,
        get_wallet, get_wallet_withdraw_stat, tm, ACC_1, ACC_2, FOUNDER_1, FOUNDER_2, FOUNDER_3,
        GROUP_1_ROLE_1, TOKEN_TOTAL_SUPPLY, VOTE_TOKEN_ACC,
    },
    wallet::{ClaimableReward, Wallet, WithdrawStats},
};

#[test]
fn reward_wage_one_asset() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset = Asset::Near;
    let reward_asset_id = 0;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![PartitionAsset::try_from(
            PartitionAssetInput {
                asset_id: reward_asset.clone(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 1000,
                    lock: None,
                },
            },
            &mut contract as &mut dyn AssetRegistrar,
        )
        .unwrap()],
    };
    let role_id = get_role_id(&contract, 1, GROUP_1_ROLE_1);
    let partition_id = contract.partition_add(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let reward = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_wage(2),
        vec![(reward_asset_id, 1)],
        0,
        1000,
    );
    let reward_id = contract.reward_add(reward).unwrap();
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(2)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        1
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 1);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset_id);
    assert_eq!(wage_stats.timestamp_last_withdraw, 2);
    assert_eq!(wage_stats.amount, 1);
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        499
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 499);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert_eq!(wage_stats.timestamp_last_withdraw, 1000);
    assert_eq!(wage_stats.amount, 500);
    testing_env!(ctx.block_timestamp(tm(2000)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert_eq!(wage_stats.timestamp_last_withdraw, 1000);
    assert_eq!(wage_stats.amount, 500);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 500);
    assert!(contract.partition_add_asset_amount(partition_id, reward_asset_id, 100));
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(
        partition_asset.available_amount(),
        1000 * ONE_NEAR - 500 + 100
    );
    assert!(!contract.partition_add_asset_amount(2, reward_asset_id, 100));
}

#[test]
fn reward_activity_one_asset() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset = Asset::Near;
    let reward_asset_id = 0;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![PartitionAsset::try_from(
            PartitionAssetInput {
                asset_id: reward_asset.clone(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 1000,
                    lock: None,
                },
            },
            &mut contract as &mut dyn AssetRegistrar,
        )
        .unwrap()],
    };
    let role_id = get_role_id(&contract, 1, GROUP_1_ROLE_1);
    let partition_id = contract.partition_add(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let reward = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_user_activity(vec![0, 1]),
        vec![(reward_asset_id, 100)],
        0,
        4000,
    );
    let reward_id = contract.reward_add(reward).unwrap();
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(2)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert!(activity_stats.asset_id == reward_asset_id);
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    testing_env!(ctx.block_timestamp(tm(2000)).build());
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
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
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
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
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        100
    );
    contract.proposal_finish(proposal_id);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        200
    );
    let proposal: Proposal = contract.proposals.get(&proposal_id).unwrap().into();
    assert!(proposal.state == ProposalState::Accepted);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 2);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 200);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 3000);
    assert_eq!(activity_stats.executed_count, 0);
    assert_eq!(activity_stats.total_withdrawn_count, 2);
    testing_env!(ctx.block_timestamp(tm(3500)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
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
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 200);
}

#[test]
fn reward_activity_one_asset_for_anyone() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let mut tpl_settings = contract.workflow_template.get(&1).unwrap();
    tpl_settings.1[0].allowed_voters = ActivityRight::Anyone;
    contract.workflow_template.insert(&1, &tpl_settings);
    let reward_asset = Asset::Near;
    let reward_asset_id = 0;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let random_account = accounts(5);
    let reward = Reward::new(
        "test".into(),
        0,
        0,
        1,
        RewardType::new_user_activity(vec![0, 1]),
        vec![(reward_asset_id, 100)],
        0,
        4000,
    );
    let reward_id = contract.reward_add(reward).unwrap();
    assert!(contract.wallets.get(&founder_1).is_none());
    assert!(contract.wallets.get(&founder_2).is_none());
    assert!(contract.wallets.get(&founder_3).is_none());
    assert!(contract.wallets.get(&random_account).is_none());

    // Founder_1 adds new proposal and votes for it to get rewards for activities.
    testing_env!(ctx
        .predecessor_account_id(founder_1.clone())
        .attached_deposit(ONE_NEAR)
        .build());
    let mut tpl_settings = dummy_template_settings();
    tpl_settings.allowed_voters = ActivityRight::Anyone;
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
    assert!(contract.wallets.get(&founder_1).is_some());
    assert!(contract.wallets.get(&founder_2).is_none());
    assert!(contract.wallets.get(&founder_3).is_none());
    assert!(contract.wallets.get(&random_account).is_none());
    testing_env!(ctx
        .predecessor_account_id(random_account.clone())
        .attached_deposit(1)
        .build());
    contract.proposal_vote(proposal_id, 1);
    assert!(contract.wallets.get(&founder_1).is_some());
    assert!(contract.wallets.get(&founder_2).is_none());
    assert!(contract.wallets.get(&founder_3).is_none());
    assert!(contract.wallets.get(&random_account).is_some());
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    contract.proposal_finish(proposal_id);
    let proposal: Proposal = contract.proposals.get(&proposal_id).unwrap().into();
    assert!(proposal.state == ProposalState::Accepted);
    assert!(contract.wallets.get(&founder_2).is_none());
    assert!(contract.wallets.get(&founder_3).is_none());
    let wallet_1 = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet_1, reward_id, reward_asset_id);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 2);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        200
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 200);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let wallet_2 = get_wallet(&contract, &random_account);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet_2, reward_id, reward_asset_id);
    let activity_stats = withdraw_stats
        .activity_as_ref()
        .expect("reward is not activity");
    assert_eq!(activity_stats.timestamp_last_withdraw, 0);
    assert_eq!(activity_stats.executed_count, 1);
    assert_eq!(activity_stats.total_withdrawn_count, 0);
    let claimable_rewards = contract.claimable_rewards(random_account.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        100
    );
    let withdraw_amount =
        contract.internal_withdraw_reward(&random_account, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 100);
    let claimable_rewards = contract.claimable_rewards(random_account.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
}

#[test]
fn reward_wage_one_asset_reward_updated() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset = Asset::Near;
    let reward_asset_id = 0;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![PartitionAsset::try_from(
            PartitionAssetInput {
                asset_id: reward_asset.clone(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 1000,
                    lock: None,
                },
            },
            &mut contract as &mut dyn AssetRegistrar,
        )
        .unwrap()],
    };
    let role_id = get_role_id(&contract, 1, GROUP_1_ROLE_1);
    let partition_id = contract.partition_add(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let reward = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_wage(1),
        vec![(reward_asset_id, 1)],
        0,
        100,
    );
    let reward_id = contract.reward_add(reward).unwrap();
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);

    testing_env!(ctx.block_timestamp(tm(1)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        1
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 1);

    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        9
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 9);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset_id);
    assert_eq!(wage_stats.timestamp_last_withdraw, 10);
    assert_eq!(wage_stats.amount, 10);

    testing_env!(ctx.block_timestamp(tm(20)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        10
    );
    contract.reward_update(1, 15);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        5
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 5);

    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset_id);
    assert_eq!(wage_stats.timestamp_last_withdraw, 20);
    assert_eq!(wage_stats.amount, 15);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 15);
}

/// User withdraws amount at time x and reward is then updated to be valid only to time x - y; y > 0.
#[test]
fn reward_wage_one_asset_reward_updated_2() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let reward_asset = Asset::Near;
    let reward_asset_id = 0;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![PartitionAsset::try_from(
            PartitionAssetInput {
                asset_id: reward_asset.clone(),
                unlocking: UnlockingInput {
                    amount_init_unlock: 1000,
                    lock: None,
                },
            },
            &mut contract as &mut dyn AssetRegistrar,
        )
        .unwrap()],
    };
    let role_id = get_role_id(&contract, 1, GROUP_1_ROLE_1);
    let partition_id = contract.partition_add(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR);
    let reward = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_wage(1),
        vec![(reward_asset_id, 1)],
        0,
        100,
    );
    let reward_id = contract.reward_add(reward).unwrap();
    let wallet = get_wallet(&contract, &founder_1);
    let wallet_rewards = wallet.rewards();
    assert!(!wallet_rewards.is_empty(), "founder_1 has no rewards");
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);

    testing_env!(ctx.block_timestamp(tm(1)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        1
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 1);

    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        9
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 9);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset_id);
    assert_eq!(wage_stats.timestamp_last_withdraw, 10);
    assert_eq!(wage_stats.amount, 10);

    testing_env!(ctx.block_timestamp(tm(20)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        10
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 10);
    contract.reward_update(1, 15);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);

    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_id);
    assert_eq!(withdraw_amount, 0);
    let wallet = get_wallet(&contract, &founder_1);
    let withdraw_stats = get_wallet_withdraw_stat(&wallet, reward_id, reward_asset_id);
    let wage_stats = withdraw_stats.wage_as_ref().expect("reward is not wage");
    assert!(wage_stats.asset_id == reward_asset_id);
    assert_eq!(wage_stats.timestamp_last_withdraw, 20);
    assert_eq!(wage_stats.amount, 20);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 20);
}

#[test]
fn reward_multiple_wage_rewards() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    let ft_account_id_1 = as_account_id(ACC_1);
    let ft_account_id_2 = as_account_id(ACC_2);
    let reward_asset_1 = Asset::Near;
    let reward_asset_1_id = 0;
    let reward_asset_2 = Asset::new_ft(ft_account_id_1.clone(), 24);
    let reward_asset_2_id = 2;
    let reward_asset_3 = Asset::new_ft(ft_account_id_2.clone(), 24);
    let reward_asset_3_id = 3;
    let (founder_1, founder_2, founder_3) = (
        as_account_id(FOUNDER_1),
        as_account_id(FOUNDER_2),
        as_account_id(FOUNDER_3),
    );
    let partition = TreasuryPartition {
        name: "test".into(),
        assets: vec![
            PartitionAsset::try_from(
                PartitionAssetInput {
                    asset_id: reward_asset_1.clone(),
                    unlocking: UnlockingInput {
                        amount_init_unlock: 1000,
                        lock: None,
                    },
                },
                &mut contract as &mut dyn AssetRegistrar,
            )
            .unwrap(),
            PartitionAsset::try_from(
                PartitionAssetInput {
                    asset_id: reward_asset_2.clone(),
                    unlocking: UnlockingInput {
                        amount_init_unlock: 2000,
                        lock: None,
                    },
                },
                &mut contract as &mut dyn AssetRegistrar,
            )
            .unwrap(),
            PartitionAsset::try_from(
                PartitionAssetInput {
                    asset_id: reward_asset_3.clone(),
                    unlocking: UnlockingInput {
                        amount_init_unlock: 3000,
                        lock: None,
                    },
                },
                &mut contract as &mut dyn AssetRegistrar,
            )
            .unwrap(),
        ],
    };
    assert_eq!(
        contract.registered_assets(),
        vec![
            (0, Asset::Near),
            (1, Asset::new_ft(as_account_id(VOTE_TOKEN_ACC), 24)),
            (2, reward_asset_2.clone()),
            (3, reward_asset_3.clone())
        ]
    );
    let role_id = get_role_id(&contract, 1, GROUP_1_ROLE_1);
    let partition_id = contract.partition_add(partition);
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset_1 = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    let partition_asset_2 = partition
        .asset(reward_asset_2_id)
        .expect("partition asset not found");
    let partition_asset_3 = partition
        .asset(reward_asset_3_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset_1.available_amount(), 1000 * ONE_NEAR);
    assert_eq!(partition_asset_2.available_amount(), 2000 * ONE_NEAR);
    assert_eq!(partition_asset_3.available_amount(), 3000 * ONE_NEAR);
    let reward_only_near = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_wage(10),
        vec![(reward_asset_1_id, 100)],
        0,
        1000,
    );
    let reward_only_fts = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_wage(10),
        vec![(reward_asset_2_id, 100), (reward_asset_3_id, 100)],
        0,
        1000,
    );
    let reward_all_tokens = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_wage(10),
        vec![
            (reward_asset_1_id, 100),
            (reward_asset_2_id, 100),
            (reward_asset_3_id, 100),
        ],
        0,
        1000,
    );
    let reward_id_only_near = contract.reward_add(reward_only_near).unwrap();
    let reward_id_only_fts = contract.reward_add(reward_only_fts).unwrap();
    let reward_id_all_tokens = contract.reward_add(reward_all_tokens).unwrap();
    assert!(reward_id_only_near == 1 && reward_id_only_fts == 2 && reward_id_all_tokens == 3);
    testing_env!(ctx.block_timestamp(tm(9)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        0
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        0
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        0
    );
    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        200
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        200
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        200
    );
    testing_env!(ctx.block_timestamp(tm(20)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        400
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        400
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        400
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 200);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        200
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        400
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        400
    );
    testing_env!(ctx.block_timestamp(tm(200)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        3_800
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        4_000
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        4_000
    );
    let withdraw_amount =
        contract.internal_withdraw_reward(&founder_1, vec![2, 3], reward_asset_2_id);
    assert_eq!(withdraw_amount, 4_000);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        3_800
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        0
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        4_000
    );
    testing_env!(ctx.block_timestamp(tm(1000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        19_800
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        16_000
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        20_000
    );
    testing_env!(ctx.block_timestamp(tm(2000)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        19_800
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        16_000
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        20_000
    );
    let withdraw_amount =
        contract.internal_withdraw_reward(&founder_1, vec![2, 3], reward_asset_3_id);
    assert_eq!(withdraw_amount, 20_000);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        19_800
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_2
        ),
        16_000
    );
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_3
        ),
        0
    );
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1000 * ONE_NEAR - 200);
    let partition_asset = partition
        .asset(reward_asset_2_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 2000 * ONE_NEAR - 4_000);
    let partition_asset = partition
        .asset(reward_asset_3_id)
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
    let reward_asset_1_id = 0;
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
        assets: vec![PartitionAsset::try_from(
            PartitionAssetInput {
                asset_id: reward_asset_1.clone(),
                unlocking: lock_input,
            },
            &mut contract as &mut dyn AssetRegistrar,
        )
        .unwrap()],
    };
    let partition_id = contract.partition_add(partition);
    let mut partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1 * ONE_NEAR);
    partition.unlock_all(0);
    let partition_asset = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 1 * ONE_NEAR);
    let role_id = get_role_id(&contract, 1, GROUP_1_ROLE_1);
    let reward = Reward::new(
        "test".into(),
        1,
        role_id,
        partition_id,
        RewardType::new_wage(1),
        vec![(reward_asset_1_id, 1 * ONE_NEAR)],
        0,
        10,
    );
    let reward_id = contract.reward_add(reward).unwrap();
    testing_env!(ctx.block_timestamp(tm(0)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(1)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        1 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 1 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        0
    );
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 0);
    testing_env!(ctx.block_timestamp(tm(5)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        4 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(10)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        9 * ONE_NEAR
    );

    // Unlock all
    let mut partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_1_id)
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
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 5 * ONE_NEAR);
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 5 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        4 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(420)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        4 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 0);
    testing_env!(ctx.block_timestamp(tm(500)).build());
    let mut partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 0);

    // Refill 100 NEARs
    partition.add_amount(reward_asset_1_id, 100 * ONE_NEAR);
    let partition_asset = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(partition_asset.available_amount(), 100 * ONE_NEAR);
    contract
        .treasury_partition
        .insert(&partition_id, &partition.into());

    testing_env!(ctx.block_timestamp(tm(500)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        4 * ONE_NEAR
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 4 * ONE_NEAR);
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        0
    );
    let partition: TreasuryPartition = contract
        .treasury_partition
        .get(&partition_id)
        .expect("partition not found")
        .into();
    let partition_asset = partition
        .asset(reward_asset_1_id)
        .expect("partition asset not found");
    assert_eq!(
        partition_asset.available_amount(),
        100 * ONE_NEAR - 4 * ONE_NEAR
    );
    testing_env!(ctx.block_timestamp(tm(9999)).build());
    let claimable_rewards = contract.claimable_rewards(founder_1.clone());
    assert_eq!(
        claimable_rewards_sum(
            claimable_rewards.claimable_rewards.as_slice(),
            &reward_asset_1
        ),
        0
    );
    let withdraw_amount = contract.internal_withdraw_reward(&founder_1, vec![1], reward_asset_1_id);
    assert_eq!(withdraw_amount, 0);
}

#[test]
fn cache_reward_activity_management() {
    let mut ctx = get_context_builder();
    testing_env!(ctx.build());
    let mut contract = get_default_contract();
    assert!(contract.valid_reward_list_for_activity(0).is_empty());
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![]),
            (1, vec![]),
            (2, vec![]),
            (3, vec![]),
            (4, vec![]),
        ],
    );
    let reward_1 = Reward::new(
        "test".into(),
        1,
        0,
        1,
        RewardType::new_user_activity(vec![0, 1]),
        vec![(0, 100)],
        0,
        10,
    );
    let reward_id_1 = contract.reward_add(reward_1.clone()).unwrap();
    assert_eq!(
        contract.valid_reward_list_for_activity(0),
        vec![(reward_id_1, reward_1.clone())]
    );
    assert_eq!(
        contract.valid_reward_list_for_activity(1),
        vec![(reward_id_1, reward_1.clone())]
    );
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![reward_id_1]),
            (1, vec![reward_id_1]),
            (2, vec![]),
            (3, vec![]),
            (4, vec![]),
        ],
    );

    let reward_2 = Reward::new(
        "test".into(),
        1,
        0,
        1,
        RewardType::new_user_activity(vec![0, 2]),
        vec![(0, 100)],
        0,
        100,
    );
    let reward_id_2 = contract.reward_add(reward_2.clone()).unwrap();
    assert_eq!(
        contract.valid_reward_list_for_activity(0),
        vec![
            (reward_id_1, reward_1.clone()),
            (reward_id_2, reward_2.clone())
        ]
    );
    assert_eq!(
        contract.valid_reward_list_for_activity(1),
        vec![(reward_id_1, reward_1.clone())]
    );
    assert_eq!(
        contract.valid_reward_list_for_activity(2),
        vec![(reward_id_2, reward_2.clone())]
    );
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![reward_id_1, reward_id_2]),
            (1, vec![reward_id_1]),
            (2, vec![reward_id_2]),
            (3, vec![]),
            (4, vec![]),
        ],
    );
    testing_env!(ctx.block_timestamp(tm(10)).build());
    assert_eq!(
        contract.valid_reward_list_for_activity(0),
        vec![
            (reward_id_1, reward_1.clone()),
            (reward_id_2, reward_2.clone())
        ]
    );
    assert_eq!(
        contract.valid_reward_list_for_activity(1),
        vec![(reward_id_1, reward_1.clone())]
    );
    assert_eq!(
        contract.valid_reward_list_for_activity(2),
        vec![(reward_id_2, reward_2.clone())]
    );
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![reward_id_1, reward_id_2]),
            (1, vec![reward_id_1]),
            (2, vec![reward_id_2]),
            (3, vec![]),
            (4, vec![]),
        ],
    );
    testing_env!(ctx.block_timestamp(tm(11)).build());
    assert_eq!(
        contract.valid_reward_list_for_activity(0),
        vec![(reward_id_2, reward_2.clone())]
    );
    assert!(contract.valid_reward_list_for_activity(1).is_empty());
    assert_eq!(
        contract.valid_reward_list_for_activity(2),
        vec![(reward_id_2, reward_2.clone())]
    );
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![reward_id_2]),
            (1, vec![]),
            (2, vec![reward_id_2]),
            (3, vec![]),
            (4, vec![]),
        ],
    );
    testing_env!(ctx.block_timestamp(tm(4001)).build());
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![reward_id_2]),
            (1, vec![]),
            (2, vec![reward_id_2]),
            (3, vec![]),
            (4, vec![]),
        ],
    );
    assert!(contract.valid_reward_list_for_activity(0).is_empty());
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![]),
            (1, vec![]),
            (2, vec![reward_id_2]),
            (3, vec![]),
            (4, vec![]),
        ],
    );
    assert!(contract.valid_reward_list_for_activity(2).is_empty());
    assert_cache_reward_activity(
        &contract,
        vec![
            (0, vec![]),
            (1, vec![]),
            (2, vec![]),
            (3, vec![]),
            (4, vec![]),
        ],
    );
}
