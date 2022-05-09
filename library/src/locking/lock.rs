use std::convert::From;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

// TODO: Integration tests.

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

/// TODO: Verify possible amounts limits.
/// Lock model implements unlocking function via interpolating intervals with linear unlocking.
/// Currently unlocks only integer amounts.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Lock {
    /// Total locked amount.
    pub amount_locked: u32,
    /// Available unlocked amount.
    pub amount_unlocked: u32,
    /// Amount of tokens unlocked during creation.
    pub amount_init_unlocked: u32,
    /// Timestamp in seconds.
    pub start_from: u64,
    /// Duration in seconds.
    pub duration: u64,
    /// Interpolated function into vec of periods. Max len is `u16::MAX`;
    pub periods: Vec<UnlockPeriod>,
    /// Current period.
    pub pos: u16,
    /// Total amount unlocked from current period.
    pub current_period_unlocked: u32,
}

impl Lock {
    pub fn check_duration_and_amount(
        duration: u64,
        amount: u32,
        init_distribution: u32,
        unlock_periods: &[UnlockPeriodInput],
    ) -> bool {
        let duration_sum: u64 = unlock_periods.iter().map(|el| el.duration).sum();
        let amount_sum: u32 = unlock_periods.iter().map(|el| el.amount).sum();

        amount_sum + init_distribution == amount
            && duration_sum == duration
            && unlock_periods.len() <= u16::MAX as usize
    }

    /// Calculates amount of tokens to be unlocked depending on current time.
    /// Updates owns stats about unlocking if necessary.
    /// Currently unlocks only integer amounts.
    pub fn unlock(&mut self, current_time: u64) -> u32 {
        if self.amount_locked == self.amount_unlocked {
            return 0;
        }

        let mut pos = self.pos as usize;
        let mut current_period = &self.periods[pos];
        let mut new_unlocked = 0;

        // Check if are still in the same period
        if current_period.end_at >= current_time {
            if current_period.amount > self.current_period_unlocked {
                let total_released = match current_period.kind {
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
        } else {
            // Unlock reminder amount of current period
            new_unlocked += current_period.amount - self.current_period_unlocked;

            // Find new current period
            for i in pos + 1..self.periods.len() {
                current_period = &self.periods[i];
                pos = i;
                if current_period.end_at >= current_time {
                    break;
                }
                // Sum all previous periods
                new_unlocked += current_period.amount;
            }

            self.current_period_unlocked = 0;

            if current_period.amount > self.current_period_unlocked {
                let total_released = match current_period.kind {
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

        // Save new stats
        self.pos = pos as u16;
        self.amount_unlocked += new_unlocked;

        new_unlocked
    }

    pub fn total_locked(&self) -> u32 {
        self.amount_locked
    }
    pub fn available(&self) -> u32 {
        self.amount_unlocked
    }

    // /// Distributes `amount` if possible (meaning unlocked >= amount) and updates stats.
    /*     pub fn distribute(&mut self, amount: u32) -> bool {
        if self.unlocked >= self.distributed + amount {
            self.distributed += amount;
            true
        } else {
            false
        }
    } */
}

/* impl TryFrom<GroupTokenLockInput> for TokenLock {
    type Error = &'static str;
    fn try_from(input: GroupTokenLockInput) -> Result<Self, Self::Error> {
        if !Self::check_duration_and_amount(
            input.duration,
            input.amount,
            input.init_distribution,
            input.periods.as_slice(),
        ) {
            return Err("Invalid duration or amount.");
        }

        let mut end_at = input.start_from;
        let mut periods = Vec::with_capacity(input.periods.len());
        for period_input in input.periods.into_iter() {
            end_at += period_input.duration;
            periods.push(UnlockPeriod {
                kind: period_input.kind,
                amount: period_input.amount,
                end_at,
            })
        }

        Ok(TokenLock {
            amount: input.amount,
            unlocked: input.init_distribution,
            distributed: input.init_distribution,
            init_distribution: input.init_distribution,
            start_from: input.start_from,
            duration: input.duration,
            periods,
            pos: 0,
            current_period_unlocked: 0,
            unlock_interval: input.unlock_interval,
        })
    }
} */

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
/// Defines `amount` FT to be release at `end_at` timestamp.
/// Kind defines type of releasing. "None" releases `amount` immediately. "Linear" lineary over the time period.
pub struct UnlockPeriod {
    pub kind: UnlockMethod,
    pub end_at: u64,
    pub amount: u32,
}

#[derive(Deserialize)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Clone, Debug, PartialEq, Serialize)
)]
#[serde(crate = "near_sdk::serde")]
/// Input version of `UnlockPeriod`.
pub struct UnlockPeriodInput {
    pub kind: UnlockMethod,
    pub duration: u64,
    pub amount: u32,
}
