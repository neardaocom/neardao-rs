use std::convert::TryFrom;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env::{self},
    log,
    serde::{Deserialize, Serialize},
    AccountId, Balance, Promise, json_types::U128,
};

use crate::{
    action::{
        Action, ActionGroupInput, ActionGroupRight, ActionTx, RightTarget, TokenGroup, TxInput,
        TxValidationErr,
    },
    config::{Config, ConfigInput},
    constants::{
        ACC_REF_FINANCE, ACC_SKYWARD_FINANCE, ACC_WNEAR, DEFAULT_DOC_CAT, DEPOSIT_STANDARD_STORAGE,
    },
    core::{DaoContract},
    file::{FileType, VFileMetadata},
    release::{ReleaseDb, ReleaseModel, ReleaseModelInput, VReleaseDb, VReleaseModel},
    vote_policy::{VVoteConfig, VoteConfig, VoteConfigInput}, callbacks::ext_self,
};

impl DaoContract {
    pub fn setup_voting_policy(&mut self, configs: Vec<VoteConfigInput>) {
        for p in configs.into_iter() {
            assert!(
                self.vote_policy_config
                    .insert(
                        &p.proposal_kind.clone(),
                        &VVoteConfig::Curr(VoteConfig::try_from(p).unwrap())
                    )
                    .is_none(),
               "{}",
                "Duplicate voting policy"
            );
        }
    }

    pub fn setup_release_models(
        &mut self,
        release_config: Vec<(TokenGroup, ReleaseModelInput)>,
        founders_distribution: u32,
    ) {
        let config: Config = self.config.get().unwrap().into();

        for (group, model) in release_config.into_iter() {
            let release_model =
                ReleaseModel::from_input(model, (env::block_timestamp() / 10u64.pow(9)) as u32);

            let release_db;
            match group {
                TokenGroup::Council => {
                    release_db = if release_model == ReleaseModel::None {
                        let total = (config.council_share as u64 * self.ft_total_supply as u64
                            / 100) as u32;
                        ReleaseDb::new(total, total, founders_distribution)
                    } else {
                        ReleaseDb::new(
                            (config.council_share as u64 * self.ft_total_supply as u64 / 100)
                                as u32,
                            founders_distribution,
                            founders_distribution,
                        )
                    };
                }
                _ => env::panic(b"Cannot set Release model for Public"),
            }

            self.release_db
                .insert(&group, &VReleaseDb::Curr(release_db));
            self.release_config
                .insert(&group, &VReleaseModel::Curr(release_model));
        }

        // We set dao release
        let ft_amount = ((100
            - config.council_share as u64
            - config.foundation_share.unwrap_or_default() as u64
            - config.community_share.unwrap_or_default() as u64)
            * self.ft_total_supply as u64
            / 100) as u32;

        // dao itself has all tokens unlocked from the beginning
        self.release_db.insert(
            &TokenGroup::Public,
            &VReleaseDb::Curr(ReleaseDb::new(ft_amount, ft_amount, 0)),
        );
        self.release_config.insert(
            &TokenGroup::Public,
            &VReleaseModel::Curr(ReleaseModel::None),
        );
    }

    pub fn init_mappers(&mut self) {
        self.mappers.insert(
            &MapperKind::Doc,
            &Mapper::Doc {
                tags: [].into(),
                categories: [DEFAULT_DOC_CAT.into()].into(),
            },
        );
    }

    // Assumed user cannot unregister with non-zero amount of FT
    pub fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        self.council.remove(&account_id);

        log!(
            "Closed @{} and all it's FT: {} were transfered back to the contract",
            account_id,
            balance
        );
    }

    //TODO: Tests
    pub fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        self.ft.internal_deposit(&env::current_account_id(), amount);

        self.council.remove(&account_id);

        log!(
            "Account @{} deleted and all it's FT: {} were transfered back to the contract",
            account_id,
            amount
        );
    }

    /// Validates all actions and tries to execute transaction
    pub fn execute_tx(&mut self, tx: &ActionTx, ctx: Context) -> Result<(), Vec<TxValidationErr>> {
        let mut errors: Vec<TxValidationErr> = Vec::new();

        // Checks if all actions might be successfully executed
        self.validate_tx_before_execution(
            tx,
            ctx.current_balance,
            ctx.attached_deposit,
            ctx.current_block_timestamp,
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(errors);
        }

        // All actions should be executed now without any error
        for action in tx.actions.iter() {
            self.inner_execute_action(action, &ctx);
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn validate_tx_before_execution(
        &self,
        tx: &ActionTx,
        current_balance: u128,
        attached_deposit: u128,
        current_block_timestamp: u64,
        errors: &mut Vec<TxValidationErr>,
    ) {
        for action in tx.actions.iter() {
            match action {
                Action::SendNear {
                    account_id,
                    amount_near,
                } => {
                    if current_balance < *amount_near {
                        errors.push(TxValidationErr::NotEnoughNears);
                    }
                }
                Action::AddMember { account_id, group } => {}
                Action::RemoveMember { account_id, group } => {}
                Action::GeneralProposal { title } => {}
                Action::AddFile {
                    cid,
                    ftype,
                    metadata,
                    new_category,
                    new_tags,
                } => match ftype {
                    FileType::Doc => {
                        if self.doc_metadata.get(cid).is_some() {
                            errors.push(TxValidationErr::CIDExists);
                        }
                    }
                    _ => unimplemented!(),
                },
                Action::InvalidateFile { cid } => {}
                Action::DistributeFT {
                    amount,
                    from_group,
                    accounts,
                } => {
                    let db: ReleaseDb = self.release_db.get(&from_group).unwrap().into();

                    if db.unlocked - db.distributed < *amount {
                        errors.push(TxValidationErr::NotEnoughFT);
                    }
                }
                Action::AddRightsForActionGroup {
                    to,
                    rights,
                    time_from,
                    time_to,
                } => {}
                _ => unimplemented!(),
            }
        }
    }

    #[allow(unused)]
    pub fn inner_execute_action(&mut self, action: &Action, ctx: &Context) {
        match action {
            Action::SendNear {
                account_id,
                amount_near,
            } => {
                Promise::new(account_id.into()).transfer(*amount_near);
            }
            Action::AddMember { account_id, group } => {
                if !self.ft.accounts.contains_key(account_id) {
                    self.ft.internal_register_account(account_id);
                }

                match group {
                    TokenGroup::Council => {
                        self.council.insert(account_id);
                    }
                    TokenGroup::Public => (),
                }
            }
            Action::RemoveMember { account_id, group } => match group {
                TokenGroup::Council => {
                    self.council.remove(account_id);
                }
                TokenGroup::Public => unreachable!(),
            },
            Action::GeneralProposal { title } => {}
            Action::AddFile {
                cid,
                ftype,
                metadata,
                new_category,
                new_tags,
            } => {
                match ftype {
                    FileType::Doc => {
                        match self.mappers.get(&MapperKind::Doc).unwrap() {
                            Mapper::Doc {
                                mut tags,
                                mut categories,
                            } => {
                                let mut new_metadata = match metadata {
                                    VFileMetadata::Curr(fm) => fm.clone(),
                                    _ => unreachable!(),
                                };
                                if new_category.is_some() {
                                    if let Some(idx) =
                                        categories.iter().enumerate().find_map(|(i, s)| {
                                            s.eq(new_category.as_ref().unwrap()).then(|| i)
                                        })
                                    {
                                        new_metadata.category = idx as u8;
                                    } else {
                                        categories.push(new_category.clone().unwrap());
                                        new_metadata.category = categories.len() as u8 - 1;
                                    }
                                }

                                if new_tags.len() > 0 {
                                    // Check any of the new tags exist
                                    for nt in new_tags {
                                        if tags
                                            .iter()
                                            .enumerate()
                                            .find_map(|(i, s)| s.eq(nt).then(|| i))
                                            .is_none()
                                        {
                                            tags.push(nt.clone());
                                            new_metadata.tags.push(tags.len() as u8 - 1);
                                        }
                                    }
                                }

                                self.doc_metadata
                                    .insert(cid, &VFileMetadata::Curr(new_metadata));
                                self.mappers
                                    .insert(&MapperKind::Doc, &Mapper::Doc { tags, categories });
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            Action::InvalidateFile { cid } => {
                let mut metadata = match self.doc_metadata.get(&cid.clone()).unwrap() {
                    VFileMetadata::Curr(fm) => fm,
                    _ => unreachable!(),
                };

                if metadata.valid == true {
                    metadata.valid = false;
                    self.doc_metadata
                        .insert(&cid.clone(), &VFileMetadata::Curr(metadata));
                }
            }
            Action::DistributeFT {
                amount,
                from_group,
                accounts,
            } => {
                let mut db: ReleaseDb = self.release_db.get(&from_group).unwrap().into();
                let amount_per_account = *amount / accounts.len() as u32;

                for acc in accounts.iter() {
                    if !self.ft.accounts.contains_key(acc) {
                        self.ft.internal_register_account(acc);
                    }

                    self.ft.internal_transfer(
                        &env::current_account_id(),
                        &acc,
                        amount_per_account as u128 * self.decimal_const,
                        None,
                    );
                }

                self.ft_total_distributed += amount_per_account * accounts.len() as u32;
                db.distributed += amount_per_account * accounts.len() as u32;
                self.release_db.insert(from_group, &VReleaseDb::Curr(db));
            }
            Action::AddRightsForActionGroup {
                to,
                rights,
                time_from,
                time_to,
            } => match to {
                RightTarget::Group { value } => {
                    let mut group_rights = self
                        .group_rights
                        .get(value)
                        .unwrap_or(Vec::with_capacity(2));
                    merge_rights(rights, &mut group_rights, *time_from, *time_to);

                    self.group_rights.insert(value, &group_rights);
                }
                RightTarget::Users { values } => {
                    for u in values {
                        let mut user_rights =
                            self.user_rights.get(u).unwrap_or(Vec::with_capacity(2));
                        merge_rights(rights, &mut user_rights, *time_from, *time_to);

                        self.user_rights.insert(u, &user_rights);
                    }
                }
            },
            _ => unimplemented!(),
        }
    }

    pub fn create_tx(
        &self,
        tx_input: TxInput,
        _caller: &AccountId,
        _current_block_timestamp: u64,
    ) -> Result<ActionTx, Vec<TxValidationErr>> {
        let mut actions = Vec::with_capacity(2);
        let mut errors = Vec::with_capacity(2);
        //let _config = Config::from(self.config.get().unwrap());

        match tx_input {
            TxInput::Pay {
                account_id,
                amount_near,
            } => {
                actions.push(Action::SendNear {
                    account_id,
                    amount_near: amount_near.0,
                });
            }
            TxInput::AddMember { account_id, group } => {
                match group {
                    TokenGroup::Council => {
                        if self.council.contains(&account_id) {
                            errors.push(TxValidationErr::UserAlreadyInGroup);
                        }
                    }
                    TokenGroup::Public => {
                        if self.ft.accounts.contains_key(&account_id) {
                            errors.push(TxValidationErr::UserAlreadyInGroup);
                        }
                    }
                }

                if errors.is_empty() {
                    actions.push(Action::AddMember {
                        account_id,
                        group: group,
                    });
                }
            }
            TxInput::RemoveMember { account_id, group } => {
                match group {
                    TokenGroup::Council => {
                        if !self.council.contains(&account_id) {
                            errors.push(TxValidationErr::UserNotInGroup);
                        }
                    }
                    TokenGroup::Public => {
                        errors.push(TxValidationErr::GroupForbidden);
                    }
                }

                if errors.is_empty() {
                    actions.push(Action::RemoveMember {
                        account_id,
                        group: group,
                    });
                }
            }
            TxInput::GeneralProposal { title } => {
                //TODO limit title length ??
                actions.push(Action::GeneralProposal { title });
            }
            TxInput::AddDocFile {
                cid,
                metadata,
                new_category,
                new_tags,
            } => {
                //TODO check precise length, not range
                if cid.len() > crate::constants::CID_MAX_LENGTH.into() {
                    errors.push(TxValidationErr::Custom("Invalid CID length".into()));
                } else if self.doc_metadata.get(&cid).is_some() {
                    errors.push(TxValidationErr::Custom("Metadata already exists".into()));
                } else if new_category.is_some()
                    && new_category.as_ref().map(|s| s.len()).unwrap() == 0
                {
                    errors.push(TxValidationErr::Custom(
                        "Category cannot be empty string".into(),
                    ));
                } else {
                    //TODO tags check ??
                    actions.push(Action::AddFile {
                        cid,
                        metadata,
                        ftype: FileType::Doc,
                        new_category,
                        new_tags,
                    });
                }
            }
            TxInput::InvalidateFile { cid } => {
                if self.doc_metadata.get(&cid).is_none() {
                    errors.push(TxValidationErr::Custom("Metadata does not exist".into()));
                } else {
                    actions.push(Action::InvalidateFile { cid });
                }
            }
            TxInput::DistributeFT {
                total_amount,
                from_group,
                accounts,
            } => {
                let db: ReleaseDb = self.release_db.get(&from_group).unwrap().into();

                if db.unlocked - db.distributed < total_amount {
                    errors.push(TxValidationErr::NotEnoughFT);
                } else {
                    actions.push(Action::DistributeFT {
                        amount: total_amount,
                        from_group,
                        accounts,
                    });
                }
            }
            TxInput::RightForActionCall {
                to,
                rights,
                time_from,
                time_to,
            } => {
                let time_from = time_from.map(|t| t.0).unwrap_or(0u64);
                let time_to = time_to.map(|t| t.0).unwrap_or(u64::MAX);

                if time_from >= time_to {
                    errors.push(TxValidationErr::InvalidTimeInputs);
                } else {
                    actions.push(Action::AddRightsForActionGroup {
                        to,
                        rights,
                        time_from,
                        time_to,
                    });
                }
            }
            _ => unimplemented!(),
        }

        if errors.is_empty() {
            return Ok(ActionTx { actions });
        } else {
            return Err(errors);
        }
    }

    pub fn execute_privileged_action_group_call(&mut self, action_group: ActionGroupInput) -> Promise {
        match action_group {
            ActionGroupInput::RefRegisterTokens => {
                let mut promise = Promise::new(ACC_REF_FINANCE.into());

                if !self.storage_deposit.contains(&ACC_REF_FINANCE.into()) {
                    promise = promise.function_call(
                        b"storage_deposit".to_vec(),
                        b"{}".to_vec(),
                        100_000_000_000_000_000_000_000,
                        10_000_000_000_000,
                    );

                    self.storage_deposit.insert(&ACC_REF_FINANCE.into());
                }

                promise.function_call(
                    b"register_tokens".to_vec(),
                    format!(
                        "{{\"token_ids\":[\"{}\",\"{}\"]}}",
                        env::current_account_id(),
                        ACC_WNEAR,
                    )
                    .into_bytes(),
                    1,
                    15_000_000_000_000,
                )
            }
            ActionGroupInput::RefAddPool { fee} => {
                Promise::new(ACC_REF_FINANCE.into()).function_call(
                    b"add_simple_pool".to_vec(),
                    format!(
                        "{{\"tokens\":[\"{}\",\"{}\"],\"fee\": {}}}",
                        env::current_account_id(),
                        ACC_WNEAR,
                        fee.unwrap_or(25),
                    )
                    .into_bytes(),
                    100_000_000_000_000_000_000_000, // 0.1 N
                    10_000_000_000_000,
                ).then(ext_self::callback_insert_ref_pool(&env::current_account_id(), 0, 10_000_000_000_000))
            }
            ActionGroupInput::RefAddLiquidity {
                pool_id,
                amount_near,
                amount_ft,
            } => {
                let current_account_id = env::current_account_id();

                if !self.ft.accounts.contains_key(&ACC_REF_FINANCE.into()) {
                    self.ft
                        .accounts
                        .insert(&ACC_REF_FINANCE.into(), &amount_ft.0);
                } else {
                    self.ft.internal_transfer(
                        &current_account_id,
                        &ACC_REF_FINANCE.into(),
                        amount_ft.0,
                        None,
                    );
                }

                self.distribute_ft_checked(amount_ft.0, &TokenGroup::Public);

                let mut promise_wrap = Promise::new(ACC_WNEAR.into());

                if !self.storage_deposit.contains(&ACC_WNEAR.into()) {
                    promise_wrap = promise_wrap.function_call(
                        b"storage_deposit".to_vec(),
                        b"{}".to_vec(),
                        DEPOSIT_STANDARD_STORAGE,
                        10_000_000_000_000,
                    );

                    self.storage_deposit.insert(&ACC_WNEAR.into());
                }

                promise_wrap = promise_wrap
                    .function_call(
                        b"near_deposit".to_vec(),
                        b"{}".to_vec(),
                        amount_near.into(),
                        10_000_000_000_000,
                    )
                    .function_call(
                        b"ft_transfer_call".to_vec(),
                        format!(
                            "{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"msg\":\"\"}}",
                            ACC_REF_FINANCE, amount_near.0
                        )
                        .into_bytes(),
                        1,
                        50_000_000_000_000,
                    );

                let promise_ref = Promise::new(ACC_REF_FINANCE.into())
                    .function_call(
                        b"ft_on_transfer".to_vec(),
                        format!(
                            "{{\"sender_id\":\"{}\",\"amount\":\"{}\",\"msg\": \"\"}}",
                            &current_account_id, amount_ft.0
                        )
                        .into_bytes(),
                        0,
                        20_000_000_000_000,
                    )
                    .function_call(
                        b"add_liquidity".to_vec(),
                        format!(
                            "{{\"pool_id\":{},\"amounts\":[\"{}\",\"{}\"]}}",
                            pool_id, amount_ft.0, amount_near.0
                        )
                        .into_bytes(),
                        100_000_000_000_000_000_000_000, // 0.1 N
                        20_000_000_000_000,
                    );

                promise_wrap.then(promise_ref)
            }
            ActionGroupInput::RefWithdrawLiquidity { pool_id, shares, min_ft, min_near } => {
                Promise::new(ACC_REF_FINANCE.into()).function_call(
                    b"remove_liquidity".to_vec(),
                    format!(
                        "{{\"pool_id\":{},\"shares\": \"{}\", \"min_amounts\": [\"{}\",\"{}\"]}}",
                        pool_id, shares.0,
                        min_ft.unwrap_or(U128::from(1)).0,
                        min_near.unwrap_or(U128::from(1)).0,
                    )
                    .into_bytes(),
                    1,
                    50_000_000_000_000,
                )
            }
            ActionGroupInput::RefWithdrawDeposit { token_id, amount } => {
                Promise::new(ACC_REF_FINANCE.into()).function_call(
                    b"withdraw".to_vec(),
                    format!(
                        "{{\"token_id\":\"{}\",\"amount\":\"{}\", \"unregister\": false}}",
                        token_id, amount.0
                    )
                    .into_bytes(),
                    1,
                    100_000_000_000_000,
                )
            }
            ActionGroupInput::SkyCreateSale {
                title,
                url,
                amount_ft,
                out_token_id,
                time_from,
                duration,
            } => {
                let current_account_id = env::current_account_id();

                if !self.ft.accounts.contains_key(&ACC_SKYWARD_FINANCE.into()) {
                    self.ft
                        .accounts
                        .insert(&ACC_SKYWARD_FINANCE.into(), &amount_ft.0);
                } else {
                    self.ft.internal_transfer(
                        &current_account_id,
                        &ACC_SKYWARD_FINANCE.into(),
                        amount_ft.0,
                        None,
                    );
                }

                self.distribute_ft_checked(amount_ft.0, &TokenGroup::Public);

                Promise::new(ACC_SKYWARD_FINANCE.into())
                    .function_call(
                        b"register_tokens".to_vec(),
                        format!(
                            "{{\"token_account_ids\":[\"{}\",\"{}\"]}}",
                            current_account_id,
                            ACC_WNEAR,
                        )
                        .into_bytes(),
                        20_000_000_000_000_000_000_000, // 0.02 N
                        15_000_000_000_000,
                    )
                    .function_call(
                        b"ft_on_transfer".to_vec(),
                        format!(
                            "{{\"sender_id\":\"{}\",\"amount\":\"{}\",\"msg\": \"\\\"AccountDeposit\\\"\"}}",
                            current_account_id,
                            amount_ft.0
                        )
                        .into_bytes(),
                        0,
                        20_000_000_000_000,
                    )
                    .function_call(b"sale_create".to_vec(),
                    format!(
                        "{{ \"sale\": {{ \"title\": \"{}\", \"url\": \"{}\", \"permissions_contract_id\": \"{}\", \"out_tokens\": [{{ \"token_account_id\": \"{}\", \"balance\":\"{}\", \"referral_bpt\": null}}],\"in_token_account_id\":\"{}\",\"start_time\":\"{}\", \"duration\":\"{}\"}} }}",
                        title,
                        url,
                        current_account_id,
                        current_account_id,
                        amount_ft.0,
                        out_token_id,
                        time_from.0,
                        duration.0,
                    )
                    .into_bytes(),
                    2_000_000_000_000_000_000_000_000, // 2 N,
                    50_000_000_000_000
                    )
                    .then(ext_self::callback_insert_skyward_auction(&env::current_account_id(), 0, 10_000_000_000_000))
            }
        }
    }

    pub fn get_users_group(&self, account_id: &AccountId) -> Option<TokenGroup> {
        if self.council.contains(account_id) {
            Some(TokenGroup::Council)
        } else {
            None
        }
    }

    pub fn distribute_ft_checked(&mut self, amount: u128, from_group: &TokenGroup) {
        let mut group_stats: ReleaseDb = self.release_db.get(&from_group).unwrap().into();
        group_stats.distributed += (amount / self.decimal_const) as u32;

        assert!(
            group_stats.distributed <= group_stats.unlocked,
            "Not enough unlocked tokens for group"
        );

        self.release_db
            .insert(&from_group, &VReleaseDb::Curr(group_stats));
    }
}

#[inline]
pub fn assert_valid_init_config(config: &ConfigInput) {
    assert!(
        config.council_share.unwrap()
            + config.community_share.unwrap_or_default()
            + config.foundation_share.unwrap_or_default()
            <= 100
    );
    assert!(config.vote_spam_threshold.unwrap_or_default() <= 100);
    assert!(config.description.as_ref().unwrap().len() > 0);
}

#[inline]
pub fn assert_valid_founders(founders: &mut Vec<AccountId>) {
    let founders_len_before_dedup = founders.len();
    founders.sort();
    founders.dedup();
    assert_eq!(founders_len_before_dedup, founders.len());
}

pub fn merge_rights(
    input_rights: &Vec<ActionGroupRight>,
    current_rights: &mut Vec<(ActionGroupRight, TimeInterval)>,
    time_from: u64,
    time_to: u64,
) {
    for r in input_rights.iter() {
        let mut found = false;

        // try to find the right and change it with new duration
        for (gr, t) in current_rights.iter_mut() {
            if gr == r {
                *t = TimeInterval::new(time_from, time_to);
                found = true;
                break;
            }
        }

        // otherwise push it
        if !found {
            current_rights.push((r.clone(), TimeInterval::new(time_from, time_to)));
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum MapperKind {
    Doc,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub enum Mapper {
    Doc {
        tags: Vec<String>,
        categories: Vec<String>,
    },
}

pub struct Context {
    pub proposal_id: u32,
    pub attached_deposit: u128,
    pub current_balance: u128,
    pub current_block_timestamp: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct TimeInterval {
    pub from: u64,
    pub to: u64,
}

impl TimeInterval {
    #[inline]
    pub fn new(from: u64, to: u64) -> Self {
        assert!(from < to);
        TimeInterval { from, to }
    }
}
