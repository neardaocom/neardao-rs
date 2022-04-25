use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

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
    /// Invariant: Sum of all delegations <= `self.vote_amount`.
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

    //fn assert_storage(&self) {
    //    assert!(
    //        (self.storage_used as Balance) * env::storage_byte_cost() <= self.near_amount.0,
    //        "ERR_NOT_ENOUGH_STORAGE"
    //    );
    //}

    pub(crate) fn delegated_amount(&self) -> Balance {
        self.delegated_amounts
            .iter()
            .fold(0, |total, (_, amount)| total + amount)
    }

    /// Record delegation owned tokens from this account to another account.
    /// Fails if not enough available balance to delegate.
    pub fn delegate_owned(&mut self, delegate_id: AccountId, amount: Balance) {
        assert!(
            self.delegated_amount() + amount <= self.vote_amount,
            "ERR_NOT_ENOUGH_AMOUNT"
        );
        /* assert!(
            env::block_timestamp() >= self.next_action_timestamp,
            "ERR_NOT_ENOUGH_TIME_PASSED"
        ); */
        //self.storage_used += delegate_id.as_bytes().len() as StorageUsage + U128_LEN;
        self.delegated_amounts.push((delegate_id, amount));
        //self.assert_storage();
    }

    /// Removes all delegations of delegators to this user.
    pub fn forward_delegated(&mut self) -> (u128, Vec<AccountId>) {
        assert_eq!(self.delegated_vote_amount, 0, "Zero delegated tokens");
        let amount = self.delegated_vote_amount;
        let delegators = std::mem::take(&mut self.delegators);
        self.delegated_vote_amount = 0;
        (amount, delegators)
    }

    /// Add new delegated vote amount
    /// Adds new delegator if he is new one.
    pub fn add_delegated(&mut self, delegator_id: AccountId, amount: Balance) {
        if !self.is_delegator(&delegator_id) {
            self.delegators.push(delegator_id);
        }
        self.delegated_vote_amount += amount;
    }

    /// Remove delegated vote amount
    pub fn remove_delegated(
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
                .expect("Delegator not found");
            self.delegators.swap_remove(delegator_pos);
        }
        self.delegated_vote_amount -= amount;
    }

    /// Update delegate when the delegate forwards user's tokens to another user.
    pub fn update_delegation(&mut self, old_delegate_id: &AccountId, new_delegate_id: &AccountId) {
        let delegate_pos = self
            .delegated_amounts
            .iter()
            .position(|(e, _)| e == old_delegate_id)
            .expect("Delegate not found");
        self.delegated_amounts.get_mut(delegate_pos).unwrap().0 = new_delegate_id.clone();
    }

    /// Remove given amount from delegates.
    /// Fails if delegate not found or not enough amount delegated.
    /// Returns remaining amount.
    pub fn undelegate(
        &mut self,
        delegate_id: &AccountId,
        amount: Balance,
        //undelegation_period: Duration,
    ) -> u128 {
        let f = self
            .delegated_amounts
            .iter()
            .enumerate()
            .find(|(_, (account_id, _))| account_id == delegate_id)
            .expect("ERR_NO_DELEGATE");
        let element = (f.0, ((f.1).1));
        assert!(element.1 >= amount, "ERR_NOT_ENOUGH_AMOUNT");
        if element.1 == amount {
            self.delegated_amounts.remove(element.0);
            0
            //self.storage_used -= delegate_id.as_bytes().len() as StorageUsage + U128_LEN;
        } else {
            (self.delegated_amounts[element.0].1) -= amount;
            self.delegated_amounts[element.0].1
        }
        //self.next_action_timestamp = (env::block_timestamp() + undelegation_period).into();
    }

    /// Withdraw the amount.
    /// Fails if there is not enough available balance.
    pub fn withdraw(&mut self, amount: Balance) {
        assert!(
            self.delegated_amount() + amount <= self.vote_amount,
            "ERR_NOT_ENOUGH_AVAILABLE_AMOUNT"
        );
        //assert!(
        //    env::block_timestamp() >= self.next_action_timestamp,
        //    "ERR_NOT_ENOUGH_TIME_PASSED"
        //);
        self.vote_amount -= amount;
    }

    /// Deposit given amount of vote tokens.
    pub fn deposit(&mut self, amount: Balance) {
        self.vote_amount += amount;
    }

    fn is_delegator(&self, account_id: &AccountId) -> bool {
        self.delegators.contains(account_id)
    }

    // /// Returns amount in NEAR that is available for storage.
    // pub fn storage_available(&self) -> Balance {
    //     self.near_amount.0 - self.storage_used as Balance * env::storage_byte_cost()
    // }
}
