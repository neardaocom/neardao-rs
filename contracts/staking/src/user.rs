use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{require, AccountId, Balance};

/// User data.
/// Recording deposited voting tokens, storage used and delegations and received delegations for voting.
/// Once delegated - the tokens are used in the votes. It records for each delegate when was the last vote.
/// It's possible to transfer all received delegations to another user.
/// Once transfered - cannot be taken back. At least for now.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// Amount of staked vote token.
    pub vote_amount: u128,
    /// List of delegations to other accounts.
    /// Invariant 1: Sum of all delegations <= `self.vote_amount`.
    /// Invariant 2: Each account_id must be unique.
    pub delegated_amounts: Vec<(AccountId, u128)>,
    /// Total delegated amount to this user by others.
    pub delegated_vote_amount: u128,
    /// List of users whom delegated their tokens to this user.
    pub delegators: Vec<AccountId>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedUser {
    Default(User),
}

impl User {
    pub fn new() -> Self {
        Self {
            vote_amount: 0,
            delegated_amounts: vec![],
            delegated_vote_amount: 0,
            delegators: vec![],
        }
    }

    pub(crate) fn delegated_amount(&self) -> Balance {
        self.delegated_amounts
            .iter()
            .fold(0, |total, (_, amount)| total + amount)
    }

    /// Record delegation owned tokens from this account to another account.
    /// Fails if not enough available balance to delegate.
    /// Return true if new delegate was added.
    pub fn delegate_owned(&mut self, delegate_id: AccountId, amount: Balance) -> bool {
        require!(
            self.delegated_amount() + amount <= self.vote_amount,
            "not enough vote tokens"
        );

        if let Some(delegate_pos) = self
            .delegated_amounts
            .iter()
            .position(|(e, _)| e == &delegate_id)
        {
            self.delegated_amounts.get_mut(delegate_pos).unwrap().1 += amount;
            false
        } else {
            self.delegated_amounts.push((delegate_id, amount));
            true
        }
    }

    /// Removes all delegators and their delegations adn returns them.
    pub fn forward_delegated(&mut self) -> (u128, Vec<AccountId>) {
        require!(self.delegated_vote_amount > 0, "no delegated tokens");
        let amount = self.delegated_vote_amount;
        let delegators = std::mem::take(&mut self.delegators);
        self.delegated_vote_amount = 0;
        (amount, delegators)
    }

    /// Add new delegated vote amount.
    /// Adds new delegator if he .
    pub fn add_delegator(&mut self, delegator_id: AccountId, amount: Balance) {
        if !self.is_delegator(&delegator_id) {
            self.delegators.push(delegator_id);
        }
        self.delegated_vote_amount += amount;
    }

    /// Remove delegated vote amount.
    /// Removes delegator from delegators if amount is zero.
    pub fn remove_delegated_amount(
        &mut self,
        delegator_id: &AccountId,
        amount: Balance,
        delegator_remaining_amount: u128,
    ) {
        if delegator_remaining_amount == 0 {
            let delegator_pos = self
                .delegators
                .iter()
                .position(|e| e == delegator_id)
                .expect("internal delegator not found");
            self.delegators.swap_remove(delegator_pos);
        }
        self.delegated_vote_amount -= amount;
    }

    /// Update delegate when the delegate forwards user's tokens to another user.
    pub fn update_delegation(&mut self, prev_delegate_id: &AccountId, new_delegate_id: &AccountId) {
        let delegate_old_pos = self
            .delegated_amounts
            .iter()
            .position(|(e, _)| e == prev_delegate_id)
            .expect("internal delegate not found");

        let delegate_new_pos = self
            .delegated_amounts
            .iter()
            .position(|(e, _)| e == new_delegate_id);

        // If the new delegate is already in user's delegates, then just add amount of the old one.
        // Else just update account id.
        if let Some(new_pos) = delegate_new_pos {
            let prev_amount = self.delegated_amounts[delegate_old_pos].1;
            self.delegated_amounts.get_mut(new_pos).unwrap().1 += prev_amount;
            self.delegated_amounts.swap_remove(delegate_old_pos);
        } else {
            self.delegated_amounts.get_mut(delegate_old_pos).unwrap().0 = new_delegate_id.clone();
        }
    }

    /// Remove given `amount` from `delegate_id`.
    /// Fails if delegate not found or not enough amount delegated.
    /// Returns remaining amount.
    pub fn undelegate(&mut self, delegate_id: &AccountId, amount: Balance) -> u128 {
        let f = self
            .delegated_amounts
            .iter()
            .enumerate()
            .find(|(_, (account_id, _))| account_id == delegate_id)
            .expect("delegate not found");
        let element = (f.0, ((f.1).1));
        require!(element.1 >= amount, "amount greater than delegated amount");
        if element.1 == amount {
            self.delegated_amounts.swap_remove(element.0);
            0
        } else {
            (self.delegated_amounts[element.0].1) -= amount;
            self.delegated_amounts[element.0].1
        }
    }

    /// Withdraw the amount.
    /// Fails if there is not enough available balance.
    pub fn withdraw(&mut self, amount: Balance) {
        require!(
            self.delegated_amount() + amount <= self.vote_amount,
            "not enough free vote amount"
        );

        self.vote_amount -= amount;
    }

    /// Deposit given amount of vote tokens.
    pub fn deposit(&mut self, amount: Balance) {
        self.vote_amount += amount;
    }

    fn is_delegator(&self, account_id: &AccountId) -> bool {
        self.delegators.contains(account_id)
    }
}
