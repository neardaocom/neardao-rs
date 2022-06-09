use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    json_types::U128,
    require, AccountId, Balance,
};

use crate::{User, VersionedUser};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Dao {
    pub account_id: AccountId,
    /// Vote token account.
    pub vote_token_id: AccountId,
    /// Recording user deposits.
    pub users: UnorderedMap<AccountId, VersionedUser>,
    /// Total token amount deposited.
    pub total_amount: Balance,
}

impl Dao {
    fn save_user(&mut self, account_id: &AccountId, user: User) {
        self.users.insert(account_id, &VersionedUser::V1(user));
    }

    /// Delegate `amount` of votes from `sender_id` to `delegate_id` account.
    pub fn delegate_owned(&mut self, sender_id: AccountId, delegate_id: AccountId, amount: u128) {
        let mut sender = self.get_user(&sender_id);
        sender.delegate_owned(delegate_id.clone(), amount);
        if sender_id != delegate_id {
            let mut delegate = self.get_user(&delegate_id);
            delegate.add_delegator(sender_id.clone(), amount);
            self.save_user(&delegate_id, delegate);
        } else {
            sender.add_delegator(sender_id.clone(), amount);
        }
        self.save_user(&sender_id, sender);
    }

    /// Undelegates `amount` from `delegate_id` account back to `sender`id account.
    pub fn undelegate(&mut self, sender_id: AccountId, delegate_id: AccountId, amount: u128) {
        let mut sender = self.get_user(&sender_id);
        let remaining_amount = sender.undelegate(&delegate_id, amount);
        if sender_id != delegate_id {
            let mut delegate = self.get_user(&delegate_id);
            delegate.remove_delegated_amount(&sender_id, amount, remaining_amount);
            self.save_user(&delegate_id, delegate);
        } else {
            sender.remove_delegated_amount(&sender_id, amount, remaining_amount);
        }
        self.save_user(&sender_id, sender);
    }

    /// Delegate all delegated tokens aka transitive delegation.
    /// Once delegated - cannot be undeleged.
    pub fn delegate(&mut self, sender_id: &AccountId, delegate_id: AccountId) -> u128 {
        let mut sender = self.get_user(sender_id);
        let (amount, delegators) = sender.forward_delegated();
        self.update_user_delegations(&sender_id, &delegate_id, &delegators);
        self.update_delegate(&delegate_id, amount, delegators);
        self.save_user(sender_id, sender);
        amount
    }

    /// Update delegate with delegators.
    fn update_delegate(
        &mut self,
        delegate_id: &AccountId,
        amount: u128,
        delegators: Vec<AccountId>,
    ) {
        let mut delegate = self.get_user(&delegate_id);
        for user in delegators {
            delegate.add_delegator(user, amount);
        }
        self.save_user(&delegate_id, delegate);
    }

    /// Updates delegate of `users`.
    fn update_user_delegations(
        &mut self,
        prev_delegate_id: &AccountId,
        new_delegate_id: &AccountId,
        users: &[AccountId],
    ) {
        for acc in users {
            let mut user = self.get_user(&acc);
            user.update_delegation(prev_delegate_id, new_delegate_id);
            self.save_user(&acc, user);
        }
    }

    /// Deposit voting token.
    pub fn user_deposit(&mut self, sender_id: &AccountId, amount: u128) {
        let mut sender = self.get_user(&sender_id);
        sender.deposit(amount);
        self.save_user(&sender_id, sender);
        self.total_amount += amount;
    }

    /// Withdraw owned tokens.
    pub fn user_withdraw(&mut self, sender_id: &AccountId, amount: u128) {
        let mut sender = self.get_user(&sender_id);
        sender.withdraw(amount);
        self.save_user(&sender_id, sender);
        require!(self.total_amount >= amount, "internal user withdraw");
        self.total_amount -= amount;
    }

    /// Register user in dao.
    pub fn register_user(&mut self, sender_id: &AccountId) {
        require!(self.users.get(sender_id).is_none(), "already registered");
        let user = User::new();
        self.save_user(sender_id, user);
    }

    /// Remove user from DAO.
    /// Fails if `sender_id` account has non-zero owned/delegated tokens.
    pub fn unregister_user(&mut self, sender_id: &AccountId) {
        let user = self.get_user(sender_id);
        require!(user.vote_amount == 0, "non-zero amount of vote tokens");
        require!(
            user.delegated_vote_amount == 0,
            "non-zero amount of delegated vote tokens"
        );
        self.users.remove(sender_id);
    }

    /// Total number of tokens staked in this dao.
    pub fn ft_total_supply(&self) -> U128 {
        U128(self.total_amount)
    }

    /// Total number of tokens staked by given user.
    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        let user = self.get_user(&account_id);
        user.vote_amount.into()
    }

    /// Returns user information.
    pub fn get_user(&self, account_id: &AccountId) -> User {
        self.users
            .get(account_id)
            .map(|versioned_user| match versioned_user {
                VersionedUser::V1(user) => user,
            })
            .expect("account not registered in dao")
    }
}
