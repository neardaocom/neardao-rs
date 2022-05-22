use serde::{Deserialize, Serialize};

/// Staking user structure
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// Amount of staked vote token.
    pub vote_amount: u128,
    /// List of delegations to other accounts.
    /// Invariant: Sum of all delegations <= `self.vote_amount`.
    pub delegated_amounts: Vec<(workspaces::AccountId, u128)>,
    /// Total delegated amount to this user by others.
    pub delegated_vote_amount: u128,
    /// List of users whom delegated their tokens to this user.
    pub delegators: Vec<workspaces::AccountId>,
}
