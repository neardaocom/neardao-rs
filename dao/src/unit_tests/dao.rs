#[cfg(test)]
mod test {
    use std::convert::TryFrom;
    use std::time::Duration;
    use std::u128;

    use near_contract_standards::fungible_token::metadata::{
        FungibleTokenMetadata, FT_METADATA_SPEC,
    };
    use near_contract_standards::storage_management::StorageManagement;
    use near_sdk::env::{self, block_timestamp};
    use near_sdk::json_types::{ValidAccountId, U128};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, AccountId, MockedBlockchain};
    use near_sdk_sim::to_yocto;

    use crate::action::{TokenGroup, TxInput};
    use crate::config::{Config, ConfigInput};
    use crate::core::{
        DaoContract, DEPOSIT_ADD_PROPOSAL, GAS_ADD_PROPOSAL, GAS_FINISH_PROPOSAL, GAS_VOTE,
    };
    use crate::proposal::{Proposal, ProposalInput, ProposalKindIdent, ProposalStatus, VoteResult};
    use crate::release::{ReleaseDb, ReleaseModel, ReleaseModelInput};
    use crate::unit_tests::{DURATION_2Y_S, DURATION_3Y_S, DURATION_ONE_WEEK};
    use crate::view::StatsFT;
    use crate::vote_policy::VoteConfigInput;

    const ISSUER_ACC: &str = "dao_factory";
    const OWNER_ACC: &str = "dao_instance";
    const OWNER_ACC_FULLNAME: &str = "dao_instance.dao_factory";

    const DAO_NAME: &str = "dao";
    const DAO_DESC: &str = "dao description";

    const TOKEN_TOTAL_SUPPLY: u32 = 1_000_000_000;
    const INIT_DISTRIBUTION: u32 = 200_000_000;
    const METADATA_DECIMALS: u8 = 24;

    const DURATION_WAITING: u64 = 10_000_000_000;

    //distribution percent of free tokens
    const COUNCIL_SHARE: u8 = 25;
    const FOUNDATION_SHARE: u8 = 15;
    const COMMUNITY_SHARE: u8 = 10;

    const VOTE_SPAM_TH: u8 = 80;

    const FOUNDER_1: &str = "founder_1";
    const FOUNDER_2: &str = "founder_2";
    const FOUNDER_3: &str = "founder_3";
    const FOUNDER_4: &str = "founder_4";
    const FOUNDER_5: &str = "founder_5";

    /*************************************************
                UTIL/HELPER FUNCTIONS
    *************************************************/

    fn decimal_const() -> u128 {
        10u128.pow(METADATA_DECIMALS as u32)
    }

    fn get_default_metadata() -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Example NEAR fungible token".to_string(),
            symbol: "EXAMPLE".to_string(),
            icon: Some("some_icon".to_string()),
            reference: None,
            reference_hash: None,
            decimals: METADATA_DECIMALS,
        }
    }

    fn get_default_dao_config(council_share: Option<u8>, foundation_share: Option<u8>, community_share: Option<u8>) -> ConfigInput {
        ConfigInput {
            name: "dao".into(),
            lang: "cs".into(),
            slogan: "best dao in EU".into(),
            council_share,
            foundation_share,
            community_share,
            description: Some(DAO_DESC.into()),
            vote_spam_threshold: Some(VOTE_SPAM_TH),
        }
    }
    fn get_default_release_config() -> Vec<(TokenGroup, ReleaseModelInput)> {
        let mut vec = Vec::with_capacity(3);
        vec.push((
            TokenGroup::Council,
            ReleaseModelInput::Linear {
                from: Some(0),
                duration: DURATION_2Y_S,
            },
        ));
        vec.push((
            TokenGroup::Foundation,
            ReleaseModelInput::Linear {
                from: Some(0),
                duration: DURATION_3Y_S,
            },
        ));
        vec.push((
            TokenGroup::Community,
            ReleaseModelInput::Linear {
                from: Some(0),
                duration: DURATION_3Y_S,
            },
        ));

        vec
    }

    fn get_default_voting_policy() -> Vec<VoteConfigInput> {
        let mut vec: Vec<VoteConfigInput> = Vec::with_capacity(8);

        vec.push(VoteConfigInput {
            proposal_kind: ProposalKindIdent::AddMember,
            duration: DURATION_ONE_WEEK,
            waiting_open_duration: DURATION_WAITING,
            quorum: 50,
            approve_threshold: 51,
            vote_only_once: true,
        });

        vec.push(VoteConfigInput {
            proposal_kind: ProposalKindIdent::RemoveMember,
            duration: DURATION_ONE_WEEK,
            waiting_open_duration: DURATION_WAITING,
            quorum: 50,
            approve_threshold: 51,
            vote_only_once: true,
        });

        vec.push(VoteConfigInput {
            proposal_kind: ProposalKindIdent::Pay,
            duration: DURATION_ONE_WEEK,
            waiting_open_duration: DURATION_WAITING,
            quorum: 50,
            approve_threshold: 51,
            vote_only_once: true,
        });

        vec.push(VoteConfigInput {
            proposal_kind: ProposalKindIdent::RegularPayment,
            duration: DURATION_ONE_WEEK,
            waiting_open_duration: DURATION_WAITING,
            quorum: 50,
            approve_threshold: 51,
            vote_only_once: true,
        });

        vec.push(VoteConfigInput {
            proposal_kind: ProposalKindIdent::GeneralProposal,
            duration: DURATION_ONE_WEEK,
            waiting_open_duration: DURATION_WAITING,
            quorum: 50,
            approve_threshold: 51,
            vote_only_once: true,
        });

        vec.push(VoteConfigInput {
            proposal_kind: ProposalKindIdent::AddDocFile,
            duration: DURATION_ONE_WEEK,
            waiting_open_duration: DURATION_WAITING,
            quorum: 50,
            approve_threshold: 51,
            vote_only_once: true,
        });

        vec.push(VoteConfigInput {
            proposal_kind: ProposalKindIdent::InvalidateFile,
            duration: DURATION_ONE_WEEK,
            waiting_open_duration: DURATION_WAITING,
            quorum: 50,
            approve_threshold: 51,
            vote_only_once: true,
        });

        vec
    }

    fn get_default_founders_5() -> Vec<AccountId> {
        let mut founders = Vec::with_capacity(5);

        founders.push(FOUNDER_1.into());
        founders.push(FOUNDER_2.into());
        founders.push(FOUNDER_3.into());
        founders.push(FOUNDER_4.into());
        founders.push(FOUNDER_5.into());

        founders
    }

    /// Contract constructor
    fn get_contract(
        total_supply: u32,
        founders_init_distribution: u32,
        metadata: FungibleTokenMetadata,
        config: ConfigInput,
        release_config: Vec<(TokenGroup, ReleaseModelInput)>,
        vote_policy_config: Vec<VoteConfigInput>,
        founders: Vec<AccountId>,
    ) -> DaoContract {
        DaoContract::new(
            total_supply,
            founders_init_distribution,
            metadata,
            config,
            release_config,
            vote_policy_config,
            founders,
        )
    }

    fn get_default_contract() -> DaoContract {
        get_contract(
            TOKEN_TOTAL_SUPPLY,
            INIT_DISTRIBUTION,
            get_default_metadata(),
            get_default_dao_config(Some(COUNCIL_SHARE), Some(FOUNDATION_SHARE), Some(COMMUNITY_SHARE)),
            get_default_release_config(),
            get_default_voting_policy(),
            get_default_founders_5(),
        )
    }

    fn get_default_contract_with(
        total_supply: u32,
        founders_init_distribution: u32,
        council_share: u8,
        foundation_share: u8,
        community_share: u8,
    ) -> DaoContract {
        assert!(total_supply >= (founders_init_distribution as u64 * council_share as u64) as u32 / 100);
        assert!(council_share + foundation_share + community_share <= 100);

        get_contract(
            total_supply,
            founders_init_distribution,
            get_default_metadata(),
            get_default_dao_config(Some(council_share), Some(foundation_share), Some(community_share)),
            get_default_release_config(),
            get_default_voting_policy(),
            get_default_founders_5(),
        )
    }

    fn register_user(
        context: &mut VMContextBuilder,
        contract: &mut DaoContract,
        account: AccountId,
    ) {
        testing_env!(context
            .predecessor_account_id(ValidAccountId::try_from(env::current_account_id()).unwrap())
            .attached_deposit(contract.storage_balance_bounds().min.0)
            .build());

        contract.storage_deposit(Some(ValidAccountId::try_from(account).unwrap()), None);
    }

    fn vote_as_user(
        context: &mut VMContextBuilder,
        contract: &mut DaoContract,
        account: AccountId,
        proposal_id: u32,
        vote_kind: u8,
    ) -> VoteResult {
        testing_env!(context
            .predecessor_account_id(ValidAccountId::try_from(account.to_string()).unwrap())
            .prepaid_gas(GAS_VOTE)
            .build());

        contract.vote(proposal_id, vote_kind)
    }

    fn finish_proposal_as_user(
        context: &mut VMContextBuilder,
        contract: &mut DaoContract,
        account: AccountId,
        proposal_id: u32,
        at_block_timestamp: Option<u64>,
    ) -> ProposalStatus {
        testing_env!(context
            .predecessor_account_id(ValidAccountId::try_from(account.to_string()).unwrap())
            .prepaid_gas(GAS_FINISH_PROPOSAL)
            .build());

        if let Some(timestamp) = at_block_timestamp {
            testing_env!(context.block_timestamp(timestamp).build());
        }

        contract.finish_proposal(proposal_id)
    }
    fn get_context() -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .block_timestamp(0)
            .signer_account_id(ValidAccountId::try_from(ISSUER_ACC).unwrap()) // Who started the transaction - DaoFactory in our case
            .predecessor_account_id(ValidAccountId::try_from(ISSUER_ACC).unwrap()) // Previous cross-contract caller, its called directly from DaoFactory so its same as signer
            .current_account_id(ValidAccountId::try_from(OWNER_ACC).unwrap()) // Account owning this smart contract
            .account_balance(to_yocto("10")); //10 nears
        builder
    }

    /// Helper function to reduce boilerplate code while setting up env timestamp
    fn update_timestamp(start: &Duration, add: u64) -> Duration {
        start.checked_add(Duration::from_nanos(add)).unwrap()
    }

    /*************************************************
                        UNIT TESTS
    *************************************************/

    #[test]
    fn init_distribution() {
        let context = get_context();
        testing_env!(context.build());

        let contract = get_default_contract();
        let config = Config::from(contract.config.get().unwrap());

        assert_eq!(contract.registered_accounts_count, 5);
        assert_eq!(contract.council.len(), 5);

        let expected_stats = StatsFT {
            total_supply: TOKEN_TOTAL_SUPPLY,
            decimals: METADATA_DECIMALS,
            total_distributed: INIT_DISTRIBUTION,
            council_ft_stats: ReleaseDb::new(250_000_000, 200_000_000, 200_000_000),
            council_release_model: ReleaseModel::Linear {
                duration: DURATION_2Y_S,
                release_end: DURATION_2Y_S,
            },
            foundation_ft_stats: ReleaseDb::new(150_000_000, 0, 0),
            foundation_release_model: ReleaseModel::Linear {
                duration: DURATION_3Y_S,
                release_end: DURATION_3Y_S,
            },
            community_ft_stats: ReleaseDb::new(100_000_000, 0, 0),
            community_release_model: ReleaseModel::Linear {
                duration: DURATION_3Y_S,
                release_end: DURATION_3Y_S,
            },
            public_ft_stats: ReleaseDb::new(500_000_000, 500_000_000, 0),
            public_release_model: ReleaseModel::None,
        };

        let expected_total_distributed = 200_000_000;

        assert_eq!(contract.statistics_ft(), expected_stats);
        assert_eq!(contract.ft_total_distributed, expected_total_distributed);
    }

    #[test]
    fn proposal_to_spam() {
        let mut context = get_context();
        testing_env!(context.build());

        let mut contract = get_default_contract();

        let user = accounts(1);
        register_user(&mut context, &mut contract, user.to_string());

        // add proposal
        testing_env!(context
            .predecessor_account_id(user.clone())
            .attached_deposit(DEPOSIT_ADD_PROPOSAL)
            .prepaid_gas(GAS_ADD_PROPOSAL)
            .build());

        let proposal = ProposalInput {
            tags: vec!["test".to_string(), "proposal".to_string()],
            description: Some("Guys send me NEAR. I'll pay back. Wink wink.".into()),
            description_cid: None,
        };

        let tx_input = TxInput::Pay {
            account_id: user.to_string(),
            amount_near: U128::from(9999),
        };

        let proposal_id = contract.add_proposal(proposal, tx_input);
        assert_eq!(contract.proposal_count, proposal_id);

        // council vote
        assert_eq!(
            vote_as_user(
                &mut context,
                &mut contract,
                FOUNDER_1.to_string(),
                proposal_id,
                0
            ),
            VoteResult::Ok
        );
        assert_eq!(
            vote_as_user(
                &mut context,
                &mut contract,
                FOUNDER_2.to_string(),
                proposal_id,
                0
            ),
            VoteResult::Ok
        );
        assert_eq!(
            vote_as_user(
                &mut context,
                &mut contract,
                FOUNDER_3.to_string(),
                proposal_id,
                0
            ),
            VoteResult::Ok
        );
        assert_eq!(
            vote_as_user(
                &mut context,
                &mut contract,
                FOUNDER_4.to_string(),
                proposal_id,
                0
            ),
            VoteResult::Ok
        );
        assert_eq!(
            vote_as_user(
                &mut context,
                &mut contract,
                FOUNDER_5.to_string(),
                proposal_id,
                2
            ),
            VoteResult::Ok
        );

        assert_eq!(
            vote_as_user(
                &mut context,
                &mut contract,
                FOUNDER_1.to_string(),
                proposal_id,
                2
            ),
            VoteResult::AlreadyVoted
        );

        // finish proposal
        assert_eq!(
            finish_proposal_as_user(
                &mut context,
                &mut contract,
                user.to_string(),
                proposal_id,
                None
            ),
            ProposalStatus::InProgress
        );
        assert_eq!(
            finish_proposal_as_user(
                &mut context,
                &mut contract,
                user.to_string(),
                proposal_id,
                Some(DURATION_ONE_WEEK + 1)
            ),
            ProposalStatus::Spam
        );
        assert_eq!(
            Proposal::from(contract.proposals.get(&proposal_id).unwrap()).status,
            ProposalStatus::Spam
        );
    }

    macro_rules! test_calc_percent_u128 {
        ($value:expr, $total_value:expr, $decimals:expr, $expected_percents:expr) => {
            let decimal_const = 10u128.pow($decimals);
            let total_vote = $total_value * decimal_const;
            let vote = $value * decimal_const;
            let expected_percents = $expected_percents;
            assert_eq!(
                expected_percents,
                crate::core::calc_percent_u128(vote, total_vote, decimal_const)
            );
        };
    }

    #[test]
    fn calculate_vote_weight() {
        test_calc_percent_u128!(0, 50_000_000, 0, 0);
        test_calc_percent_u128!(220_000, 50_000_000, 0, 0);
        test_calc_percent_u128!(249_999, 50_000_000, 0, 0);
        test_calc_percent_u128!(249_999, 50_000_000, 24, 0);
        test_calc_percent_u128!(250_000, 50_000_000, 0, 1);
        test_calc_percent_u128!(500_000, 50_000_000, 0, 1);
        test_calc_percent_u128!(10_000_000, 50_000_000, 0, 20);
        test_calc_percent_u128!(10_000_000, 50_000_000, 8, 20);
        test_calc_percent_u128!(49_500_000, 50_000_000, 24, 99);
        test_calc_percent_u128!(49_200_000, 50_000_000, 24, 98);
    }

    #[test]

    fn test_unlocking_with_high_share() {
        let mut context = get_context();
        testing_env!(context.build());

        let mut contract = get_default_contract_with(TOKEN_TOTAL_SUPPLY, (TOKEN_TOTAL_SUPPLY as u64 * 45 / 100) as u32,  90,5,3);

        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 0);
        
        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(900_000_000, 450_000_000, 450_000_000));
        
        testing_env!(context
            .block_timestamp((DURATION_2Y_S as u64 * 1 / 100) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 4_500_000);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(900_000_000, 454_500_000, 450_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S as u64 * 50 / 100) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 220_500_000);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(900_000_000, 675_000_000, 450_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S as u64 * 99 / 100) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 220_500_000);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(900_000_000, 895_500_000, 450_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 4_500_000);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(900_000_000, 900_000_000, 450_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S + 1) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 0);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(900_000_000, 900_000_000, 450_000_000));
    }

    #[test]
    
    fn test_unlocking_with_low_share() {
        let mut context = get_context();
        testing_env!(context.build());
        
        let mut contract = get_default_contract_with(TOKEN_TOTAL_SUPPLY, TOKEN_TOTAL_SUPPLY * 1 / 100,  5,10,11);
        
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 0);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(50_000_000, 10_000_000, 10_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S as u64 * 1 / 100) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 400_000);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(50_000_000, 10_400_000, 10_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S as u64 * 50 / 100) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 19_600_000);


        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(50_000_000, 30_000_000, 10_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S as u64 * 99 / 100) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 19_600_000);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(50_000_000, 49_600_000, 10_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 400_000);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(50_000_000, 50_000_000, 10_000_000));

        testing_env!(context
            .block_timestamp((DURATION_2Y_S + 1) as u64 * 10u64.pow(9))
            .build());
        assert_eq!(contract.unlock_tokens(TokenGroup::Council), 0);

        let current_db: ReleaseDb = contract.release_db.get(&TokenGroup::Council).unwrap().into();
        assert_eq!(current_db, ReleaseDb::new(50_000_000, 50_000_000, 10_000_000));
    }
}
