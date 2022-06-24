use std::convert::From;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

#[derive(Deserialize, Serialize, BorshDeserialize, BorshSerialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Copy, Clone))]
#[serde(crate = "near_sdk::serde")]
#[serde(rename_all = "snake_case")]
pub enum UnlockMethod {
    /// All is immediately unlocked in the time period.
    None = 0,
    /// Linear unlocking over the time period.
    Linear,
}

impl From<&str> for UnlockMethod {
    fn from(value: &str) -> Self {
        match value {
            "linear" => Self::Linear,
            _ => Self::None,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Debug, PartialEq, Clone, Deserialize)
)]
#[serde(crate = "near_sdk::serde")]
pub struct UnlockingDB {
    /// Available unlocked amount.
    amount_available_unlocked: u32,
    /// Amount of tokens unlocked during creation.
    amount_init_unlocked: u32,
    lock: Option<Lock>,
}

impl UnlockingDB {
    /// Withdraw max possible amount up to `amount`.
    /// Update internal stats.
    pub fn withdraw(&mut self, amount: u32) -> u32 {
        let amount_possible = std::cmp::min(self.amount_available_unlocked, amount);
        self.amount_available_unlocked -= amount_possible;
        amount_possible
    }
    /// Return actually available amount.
    pub fn available(&self) -> u32 {
        self.amount_available_unlocked
    }
    /// Unlock possible amount depending on the `current_time`.
    pub fn unlock(&mut self, current_time: u64) -> u32 {
        if let Some(lock) = self.lock.as_mut() {
            let unlocked = lock.unlock(current_time);
            self.amount_available_unlocked += unlocked;
            unlocked
        } else {
            0
        }
    }
    /// Return total locked amount in inner lock.
    pub fn total_locked(&self) -> u32 {
        if let Some(ref lock) = self.lock {
            lock.amount_total_locked
        } else {
            0
        }
    }
    pub fn init_unlocked(&self) -> u32 {
        self.amount_init_unlocked
    }
}

impl TryFrom<UnlockingInput> for UnlockingDB {
    type Error = &'static str;

    fn try_from(input: UnlockingInput) -> Result<Self, Self::Error> {
        let lock = if let Some(lock_input) = input.lock {
            Some(Lock::try_from(lock_input)?)
        } else {
            None
        };
        Ok(Self {
            amount_init_unlocked: input.amount_init_unlock,
            amount_available_unlocked: input.amount_init_unlock,
            lock,
        })
    }
}

impl TryFrom<LockInput> for Lock {
    type Error = &'static str;

    fn try_from(value: LockInput) -> Result<Self, Self::Error> {
        if !check_duration_and_amount(
            value.duration,
            value.amount_total_lock,
            value.periods.as_slice(),
        ) {
            return Err("Invalid duration or amount.");
        }

        let mut end_at = value.start_from;
        let mut periods = Vec::with_capacity(value.periods.len());
        for period_input in value.periods.into_iter() {
            end_at += period_input.duration;
            periods.push(UnlockPeriod {
                r#type: period_input.r#type,
                amount: period_input.amount,
                end_at,
            })
        }
        let lock = Lock {
            amount_total_locked: value.amount_total_lock,
            amount_total_unlocked: 0,
            start_from: value.start_from,
            duration: value.duration,
            periods,
            pos: 0,
            current_period_unlocked: 0,
        };
        Ok(lock)
    }
}

/// Lock model implements unlocking function via interpolating intervals with linear unlocking.
/// Currently unlocks only integer amounts.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Debug, PartialEq, Clone, Deserialize)
)]
#[serde(crate = "near_sdk::serde")]
pub struct Lock {
    /// Total locked amount.
    amount_total_locked: u32,
    /// Total unlocked  amount.
    amount_total_unlocked: u32,
    /// Unlocking start timestamp in seconds.
    start_from: u64,
    /// Unlocking total duration in seconds.
    duration: u64,
    /// Interpolated function into vec of periods. Max len is `u16::MAX`;
    periods: Vec<UnlockPeriod>,
    /// Current period.
    pos: u16,
    /// Total amount unlocked from current period.
    current_period_unlocked: u32,
}

impl Lock {
    /// TODO: Refactoring.
    /// Calculates amount of tokens to be unlocked depending on current time.
    /// Updates internal stats.
    /// Currently unlocks only integer amounts.
    /// Return new unlocked amount.
    pub fn unlock(&mut self, current_time: u64) -> u32 {
        if self.amount_total_locked == self.amount_total_unlocked || self.start_from > current_time
        {
            return 0;
        }

        let mut pos = self.pos as usize;
        let mut current_period = &self.periods[pos];
        let mut new_unlocked = 0;

        // Check if are still in the same period.
        if current_period.end_at >= current_time {
            if current_period.amount > self.current_period_unlocked {
                let total_released = match current_period.r#type {
                    UnlockMethod::None => current_period.amount,
                    UnlockMethod::Linear => {
                        let start_time = if pos > 0 {
                            self.periods[pos - 1].end_at
                        } else {
                            self.start_from
                        };

                        let duration = current_period.end_at - start_time;
                        let current_duration = current_time - start_time;
                        ((current_duration * 100 / duration) * current_period.amount as u64 / 100)
                            as u32
                    }
                };
                new_unlocked += total_released as u32 - self.current_period_unlocked;
                self.current_period_unlocked += new_unlocked;
            }

            if current_period.end_at == current_time {
                pos += 1;
                self.current_period_unlocked = 0;
            }
        } else if current_time >= self.start_from + self.duration {
            pos = self.periods.len() - 1;
            new_unlocked = self.amount_total_locked - self.amount_total_unlocked
        } else {
            // Unlock reminder amount of current period.
            new_unlocked += current_period.amount - self.current_period_unlocked;

            // Find new current period.
            for i in pos + 1..self.periods.len() {
                current_period = &self.periods[i];
                pos = i;
                if current_period.end_at >= current_time {
                    break;
                }
                // Sum all previous periods.
                new_unlocked += current_period.amount;
            }

            self.current_period_unlocked = 0;

            if current_period.amount > self.current_period_unlocked {
                let total_released = match current_period.r#type {
                    UnlockMethod::None => current_period.amount,
                    UnlockMethod::Linear => {
                        let start_time = if pos != 0 {
                            self.periods[pos - 1].end_at
                        } else {
                            self.start_from
                        };

                        let duration = current_period.end_at - start_time;
                        let current_duration = current_time - start_time;
                        ((current_duration * 100 / duration) * current_period.amount as u64 / 100)
                            as u32
                    }
                };
                new_unlocked += total_released as u32 - self.current_period_unlocked;
                self.current_period_unlocked += new_unlocked;
            }

            if current_period.end_at == current_time {
                pos += 1;
                self.current_period_unlocked = 0;
            }
        }
        // Save new stats.
        self.pos = pos as u16;
        self.amount_total_unlocked += new_unlocked;
        new_unlocked
    }
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct UnlockingInput {
    /// Amount of tokens unlocked during creation.
    pub amount_init_unlock: u32,
    pub lock: Option<LockInput>,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct LockInput {
    /// Total locked amount.
    pub amount_total_lock: u32,
    /// Timestamp in seconds.
    pub start_from: u64,
    /// Duration in seconds.
    pub duration: u64,
    /// Interpolated function into vec of periods. Max len is `u16::MAX`;
    pub periods: Vec<UnlockPeriodInput>,
}

/// Validation that sum of all `UnlockPeriodInput` matches duration and total amount locked.
pub fn check_duration_and_amount(
    duration: u64,
    amount_total_lock: u32,
    unlock_periods: &[UnlockPeriodInput],
) -> bool {
    let duration_sum: u64 = unlock_periods.iter().map(|el| el.duration).sum();
    let amount_sum: u32 = unlock_periods.iter().map(|el| el.amount).sum();

    amount_sum == amount_total_lock
        && duration_sum == duration
        && unlock_periods.len() <= u16::MAX as usize
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
/// Defines `amount` FT to be release at `end_at` timestamp.
/// Kind defines type of releasing. "None" releases `amount` immediately. "Linear" lineary over the time period.
pub struct UnlockPeriod {
    pub r#type: UnlockMethod,
    pub end_at: u64,
    pub amount: u32,
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
/// Input version of `UnlockPeriod`.
pub struct UnlockPeriodInput {
    pub r#type: UnlockMethod,
    pub duration: u64,
    pub amount: u32,
}

#[cfg(test)]
mod test {
    use super::{LockInput, UnlockMethod, UnlockPeriodInput, UnlockingDB, UnlockingInput};
    const TOTAL_AMOUNT: u32 = 1_000_000_000;
    /// TOTAL_AMOUNT / 8
    const INIT_AMOUNT: u32 = TOTAL_AMOUNT / 8;
    const LOCK_AMOUNT: u32 = TOTAL_AMOUNT - INIT_AMOUNT;

    fn lock_one_linear_period() -> UnlockingDB {
        let input = get_unlocking_input(
            10,
            0,
            10,
            10,
            vec![UnlockPeriodInput {
                r#type: UnlockMethod::Linear,
                duration: 10,
                amount: 10,
            }],
        );
        UnlockingDB::try_from(input).expect("failed to convert LockInput to Lock")
    }

    fn default_lock() -> UnlockingDB {
        let input = get_unlocking_input(
            LOCK_AMOUNT,
            INIT_AMOUNT,
            0,
            1000,
            vec![
                UnlockPeriodInput {
                    r#type: UnlockMethod::Linear,
                    duration: 100,
                    amount: LOCK_AMOUNT / 8,
                },
                UnlockPeriodInput {
                    r#type: UnlockMethod::Linear,
                    duration: 300,
                    amount: 0,
                },
                UnlockPeriodInput {
                    r#type: UnlockMethod::None,
                    duration: 300,
                    amount: LOCK_AMOUNT / 2,
                },
                UnlockPeriodInput {
                    r#type: UnlockMethod::Linear,
                    duration: 100,
                    amount: LOCK_AMOUNT / 8,
                },
                UnlockPeriodInput {
                    r#type: UnlockMethod::Linear,
                    duration: 100,
                    amount: LOCK_AMOUNT / 8,
                },
                UnlockPeriodInput {
                    r#type: UnlockMethod::Linear,
                    duration: 100,
                    amount: LOCK_AMOUNT / 8,
                },
            ],
        );
        UnlockingDB::try_from(input).expect("failed to convert LockInput to Lock")
    }

    fn get_unlocking_input(
        amount_locked: u32,
        init_amount: u32,
        start_from: u64,
        duration: u64,
        periods: Vec<UnlockPeriodInput>,
    ) -> UnlockingInput {
        let lock_input = Some(LockInput {
            amount_total_lock: amount_locked,
            start_from,
            duration,
            periods,
        });
        UnlockingInput {
            amount_init_unlock: init_amount,
            lock: lock_input,
        }
    }

    #[test]
    fn lock_unlock_scenario() {
        let mut tl = default_lock();

        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(tl.lock.as_ref().unwrap().amount_total_unlocked, 0);
        assert_eq!(tl.available(), INIT_AMOUNT);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);

        assert_eq!(tl.withdraw(INIT_AMOUNT / 2), INIT_AMOUNT / 2);
        assert_eq!(tl.unlock(0), 0);

        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(tl.lock.as_ref().unwrap().amount_total_unlocked, 0);
        assert_eq!(tl.available(), INIT_AMOUNT / 2);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);

        // Unlock multiple times in one period.
        assert_eq!(tl.unlock(25), LOCK_AMOUNT / 32);
        assert_eq!(tl.unlock(75), LOCK_AMOUNT / 16);
        assert_eq!(tl.unlock(100), LOCK_AMOUNT / 32);
        assert_eq!(tl.withdraw(INIT_AMOUNT / 2), INIT_AMOUNT / 2);

        assert_eq!(tl.lock.as_ref().unwrap().amount_total_locked, LOCK_AMOUNT);
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_unlocked,
            LOCK_AMOUNT / 8
        );
        assert_eq!(tl.available(), LOCK_AMOUNT / 8);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);

        // Unlock 0 over some period.
        assert_eq!(tl.unlock(250), 0);
        assert_eq!(tl.unlock(399), 0);
        assert_eq!(tl.unlock(400), 0);
        assert_eq!(tl.lock.as_ref().unwrap().amount_total_locked, LOCK_AMOUNT);
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_unlocked,
            LOCK_AMOUNT / 8
        );
        assert_eq!(tl.available(), LOCK_AMOUNT / 8);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);

        // Unlock with all tokens distributed immediately.
        assert_eq!(tl.unlock(400), LOCK_AMOUNT / 2);
        assert_eq!(tl.unlock(550), 0);
        assert_eq!(tl.unlock(600), 0);
        assert_eq!(tl.unlock(700), 0);

        // Unlock after some periods already passed.
        assert_eq!(tl.unlock(900), LOCK_AMOUNT / 4);
        assert_eq!(tl.unlock(950), LOCK_AMOUNT / 16);
        assert_eq!(tl.unlock(1000), LOCK_AMOUNT / 16);

        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_unlocked,
            TOTAL_AMOUNT / 8 + TOTAL_AMOUNT / 4 + TOTAL_AMOUNT / 2
        );
        assert_eq!(tl.available(), TOTAL_AMOUNT - TOTAL_AMOUNT / 8);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);

        // Withdraw max possible amount.
        assert_eq!(tl.withdraw(TOTAL_AMOUNT), TOTAL_AMOUNT - TOTAL_AMOUNT / 8);
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_unlocked,
            TOTAL_AMOUNT / 8 + TOTAL_AMOUNT / 4 + TOTAL_AMOUNT / 2
        );
        assert_eq!(tl.available(), 0);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);

        // Try to unlock when all FT has already been unlocked.
        assert_eq!(tl.unlock(2000), 0);

        assert_eq!(tl.withdraw(TOTAL_AMOUNT), 0);
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_unlocked,
            TOTAL_AMOUNT / 8 + TOTAL_AMOUNT / 4 + TOTAL_AMOUNT / 2
        );
        assert_eq!(tl.available(), 0);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);
    }

    #[test]
    fn lock_unlock_all_at_once() {
        let mut tl = default_lock();
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(tl.lock.as_ref().unwrap().amount_total_unlocked, 0);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);
        assert_eq!(tl.amount_available_unlocked, INIT_AMOUNT);

        // Unlock all - lock duration has passed.
        assert_eq!(tl.unlock(1500), TOTAL_AMOUNT - INIT_AMOUNT);
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_unlocked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);
        assert_eq!(tl.amount_available_unlocked, TOTAL_AMOUNT);

        // Try to unlock when all FT has already been unlocked.
        assert_eq!(tl.unlock(2000), 0);
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_unlocked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);
        assert_eq!(tl.amount_available_unlocked, TOTAL_AMOUNT);
    }

    #[test]
    fn lock_init() {
        let tl = default_lock();
        assert_eq!(tl.lock.as_ref().unwrap().periods.len(), 6);
        assert_eq!(
            tl.lock.as_ref().unwrap().amount_total_locked,
            TOTAL_AMOUNT - INIT_AMOUNT
        );
        assert_eq!(tl.lock.as_ref().unwrap().amount_total_unlocked, 0);
        assert_eq!(tl.amount_init_unlocked, INIT_AMOUNT);
        assert_eq!(tl.amount_available_unlocked, INIT_AMOUNT);
        assert_eq!(tl.lock.as_ref().unwrap().duration, 1000);
        assert_eq!(tl.lock.as_ref().unwrap().start_from, 0);
        assert_eq!(tl.lock.as_ref().unwrap().periods[0].end_at, 100);
        assert_eq!(tl.lock.as_ref().unwrap().periods[0].amount, LOCK_AMOUNT / 8);
        assert_eq!(tl.lock.as_ref().unwrap().periods[1].end_at, 400);
        assert_eq!(tl.lock.as_ref().unwrap().periods[1].amount, 0);
        assert_eq!(tl.lock.as_ref().unwrap().periods[2].end_at, 700);
        assert_eq!(tl.lock.as_ref().unwrap().periods[2].amount, LOCK_AMOUNT / 2);
        assert_eq!(tl.lock.as_ref().unwrap().periods[3].end_at, 800);
        assert_eq!(tl.lock.as_ref().unwrap().periods[3].amount, LOCK_AMOUNT / 8);
        assert_eq!(tl.lock.as_ref().unwrap().periods[4].end_at, 900);
        assert_eq!(tl.lock.as_ref().unwrap().periods[4].amount, LOCK_AMOUNT / 8);
        assert_eq!(tl.lock.as_ref().unwrap().periods[5].end_at, 1000);
        assert_eq!(tl.lock.as_ref().unwrap().periods[5].amount, LOCK_AMOUNT / 8);
    }

    #[test]
    fn lock_later_unlock() {
        let mut tl = lock_one_linear_period();
        assert_eq!(tl.unlock(0), 0);
        assert_eq!(tl.unlock(5), 0);
        assert_eq!(tl.unlock(9), 0);
        assert_eq!(tl.unlock(10), 0);
        assert_eq!(tl.unlock(11), 1);
        assert_eq!(tl.unlock(12), 1);
        assert_eq!(tl.unlock(20), 8);
        assert_eq!(tl.unlock(21), 0);
        assert_eq!(tl.unlock(200), 0);
    }

    #[test]
    fn lock_later_mass_unlock() {
        let mut tl = lock_one_linear_period();
        assert_eq!(tl.unlock(0), 0);
        assert_eq!(tl.unlock(5), 0);
        assert_eq!(tl.unlock(9), 0);
        assert_eq!(tl.unlock(21), 10);
    }
}
