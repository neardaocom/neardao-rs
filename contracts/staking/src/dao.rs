use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
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
    pub users: LookupMap<AccountId, VersionedUser>,
    /// Total token amount deposited.
    pub total_amount: Balance,
}

// TODO: Unstake settings.
impl Dao {
    fn save_user(&mut self, account_id: &AccountId, user: User) {
        self.users.insert(account_id, &VersionedUser::Default(user));
    }

    /// Delegate give amount of votes to given account.
    /// If enough tokens and storage, forwards this to owner account.
    pub fn delegate_owned(
        &mut self,
        sender_id: AccountId,
        delegate_id: AccountId,
        amount: u128,
    ) -> bool {
        let mut sender = self.get_user(&sender_id);
        let mut delegate = self.get_user(&delegate_id);
        let new_added = sender.delegate_owned(delegate_id.clone(), amount);
        if new_added {
            delegate.add_delegator(sender_id.clone(), amount);
        }
        self.save_user(&sender_id, sender);
        self.save_user(&delegate_id, delegate);
        new_added
    }

    /// Remove given amount of delegation.
    /// Returns true if delegate was removed.
    pub fn undelegate(
        &mut self,
        sender_id: AccountId,
        delegate_id: AccountId,
        amount: u128,
    ) -> bool {
        let mut sender = self.get_user(&sender_id);
        let mut delegate = self.get_user(&delegate_id);
        let remainaing_amount = sender.undelegate(&delegate_id, amount);
        delegate.remove_delegated(&sender_id, amount, remainaing_amount);
        self.save_user(&sender_id, sender);
        self.save_user(&delegate_id, delegate);
        remainaing_amount == 0
    }

    /// Delegate all delegated tokens aka transitive delegation.
    pub fn delegate(&mut self, sender_id: &AccountId, delegate_id: AccountId) -> u128 {
        let mut sender = self.get_user(sender_id);
        let (amount, changed_users) = sender.forward_delegated();
        self.update_user_delegations(&sender_id, &delegate_id, &changed_users);
        self.update_delegate(&delegate_id, amount, changed_users);
        self.save_user(sender_id, sender);
        amount
    }

    /// Transfers delegated tokens to new delegate
    fn update_delegate(&mut self, delegate_id: &AccountId, amount: u128, users: Vec<AccountId>) {
        let mut delegate = self.get_user(&delegate_id);
        for user in users {
            delegate.add_delegator(user, amount);
        }
        self.save_user(&delegate_id, delegate);
    }

    /// Updates delegate of users
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
    pub fn user_deposit(&mut self, sender_id: AccountId, amount: u128) {
        let mut sender = self.get_user(&sender_id);
        sender.deposit(amount);
        self.save_user(&sender_id, sender);
        self.total_amount += amount;
    }

    /// Withdraw non delegated tokens back to the user's account.
    /// If user's account is not registered, will keep funds here.
    pub fn user_withdraw(&mut self, sender_id: &AccountId, amount: u128) {
        let mut sender = self.get_user(&sender_id);
        sender.withdraw(amount);
        self.save_user(&sender_id, sender);
        require!(self.total_amount >= amount, "ERR_INTERNAL");
        self.total_amount -= amount;
    }

    /// Registers user in DAO:
    pub fn register_user(&mut self, sender_id: &AccountId) {
        require!(!self.users.contains_key(sender_id));
        let user = User::new();
        self.save_user(sender_id, user);
    }

    /// Removes user from DAO.
    /// Operation fails of user's amount of vote tokens is non-zero.
    pub fn unregister_user(&mut self, sender_id: &AccountId) {
        let user = self.get_user(sender_id);
        require!(user.vote_amount == 0, "Non zero amount of vote tokens");
        self.users.remove(sender_id);
    }

    /// Total number of tokens staked in this contract.
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
                VersionedUser::Default(user) => user,
            })
            .expect("Account not registered in dao")
    }
}
