use std::convert::TryFrom;

use crate::{
    constants::MAX_FT_TOTAL_SUPPLY,
    create_val_to_percent_closure,
    group::GroupTokenLockInput,
    token_lock::{UnlockMethod, TokenLock, UnlockPeriod, UnlockPeriodInput},
    unit_tests::DURATION_1Y_S,
};

const DURATION: u64 = 1_000;
const TOTAL_AMOUNT: u32 = MAX_FT_TOTAL_SUPPLY;

fn get_default_release() -> TokenLock {
    get_test_release(
        TOTAL_AMOUNT,
        0,
        1000,
        vec![
            UnlockPeriodInput {
                kind: UnlockMethod::Linear,
                duration: 100,
                amount: TOTAL_AMOUNT / 8,
            },
            UnlockPeriodInput {
                kind: UnlockMethod::Linear,
                duration: 300,
                amount: 0,
            },
            UnlockPeriodInput {
                kind: UnlockMethod::None,
                duration: 300,
                amount: TOTAL_AMOUNT / 2,
            },
            UnlockPeriodInput {
                kind: UnlockMethod::Linear,
                duration: 100,
                amount: TOTAL_AMOUNT / 16,
            },
            UnlockPeriodInput {
                kind: UnlockMethod::Linear,
                duration: 100,
                amount: TOTAL_AMOUNT / 16,
            },
            UnlockPeriodInput {
                kind: UnlockMethod::Linear,
                duration: 100,
                amount: TOTAL_AMOUNT / 8,
            },
        ],
    )
}

fn get_test_release(
    amount: u32,
    start_from: u64,
    duration: u64,
    periods: Vec<UnlockPeriodInput>,
) -> TokenLock {
    let group_token_lock_input = GroupTokenLockInput {
        amount,
        start_from,
        init_distribution: TOTAL_AMOUNT / 8,
        duration,
        unlock_interval: 0,
        periods,
    };

    TokenLock::try_from(group_token_lock_input)
        .expect("Failed to convert TokenLock from GroupTokenLockInput.")
}

#[test]
fn release_complex() {
    let mut tl = get_default_release();

    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT / 8);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);

    assert_eq!(tl.unlock(0), 0);

    // Unlock multiple times in one period.
    assert_eq!(tl.unlock(25), TOTAL_AMOUNT / 32);
    assert_eq!(tl.unlock(75), TOTAL_AMOUNT / 16);
    assert_eq!(tl.unlock(100), TOTAL_AMOUNT / 32);

    // Unlock 0 over some period.
    assert_eq!(tl.unlock(250), 0);
    assert_eq!(tl.unlock(399), 0);
    assert_eq!(tl.unlock(400), 0);

    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT / 4);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);

    // Unlock with all tokens distributed immediately.
    assert_eq!(tl.unlock(400), TOTAL_AMOUNT / 2);
    assert_eq!(tl.unlock(550), 0);
    assert_eq!(tl.unlock(600), 0);
    assert_eq!(tl.unlock(700), 0);

    // Unlock after some periods already passed.
    assert_eq!(tl.unlock(900), TOTAL_AMOUNT / 8);
    assert_eq!(tl.unlock(950), TOTAL_AMOUNT / 16);
    assert_eq!(tl.unlock(1000), TOTAL_AMOUNT / 16);

    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);
    
    // Try to unlock when all FT has already been unlocked.
    assert_eq!(tl.unlock(2000), 0);

    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);
}

#[test]
fn release_all_once() {
    let mut tl = get_default_release();

    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT / 8);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);

    assert_eq!(tl.unlock(1000), TOTAL_AMOUNT - TOTAL_AMOUNT / 8);

    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);

    // Try to unlock when all FT has already been unlocked.
    assert_eq!(tl.unlock(2000), 0);

    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);
}

#[test]
fn init() {
    let tl = get_default_release();

    assert_eq!(tl.periods.len(), 6);
    assert_eq!(tl.distributed, TOTAL_AMOUNT / 8);
    assert_eq!(tl.init_distribution, TOTAL_AMOUNT / 8);
    assert_eq!(tl.unlocked, TOTAL_AMOUNT / 8);
    assert_eq!(tl.amount, TOTAL_AMOUNT);
    assert_eq!(tl.duration, 1000);
    assert_eq!(tl.start_from, 0);
    assert_eq!(tl.periods[0].end_at, 100);
    assert_eq!(tl.periods[0].amount, TOTAL_AMOUNT / 8);
    assert_eq!(tl.periods[1].end_at, 400);
    assert_eq!(tl.periods[1].amount, 0);
    assert_eq!(tl.periods[2].end_at, 700);
    assert_eq!(tl.periods[2].amount, TOTAL_AMOUNT / 2);
    assert_eq!(tl.periods[3].end_at, 800);
    assert_eq!(tl.periods[3].amount, TOTAL_AMOUNT / 16);
    assert_eq!(tl.periods[4].end_at, 900);
    assert_eq!(tl.periods[4].amount, TOTAL_AMOUNT / 16);
    assert_eq!(tl.periods[5].end_at, 1000);
    assert_eq!(tl.periods[5].amount, TOTAL_AMOUNT / 8);
}
