use std::convert::TryFrom;

pub use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    env, ext_contract,
    json_types::{ValidAccountId, U128, U64},
    near_bindgen,
    serde::{Serialize, Deserialize},
    serde_json::{self, json},
    utils::{assert_one_yocto, is_promise_success},
    AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

pub use crate::account::Account;
pub use crate::errors::*;
pub use crate::partnered_game::PartneredGame;

pub mod account;
pub mod actions;
pub mod errors;
pub mod ext_interface;
pub mod partnered_game;

// const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const FRACTIONAL_BASE: u128 = 100_000;

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Accounts,
    PartneredGames,
    AccountBalances { account_id: AccountId },
    OwnerFunds,
    NftFunds,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Contract {
    pub owner_id: AccountId,
    pub nft_account: AccountId,
    pub panic_button: bool,
    pub bet_payment_adjustment: u128, // base 10e-5

    pub nft_fee: u128,   // base 10e-5
    pub owner_fee: u128, // base 10e-5
    pub house_fee: u128,
    pub max_bet: u128,
    pub min_bet: u128,
    pub min_balance_fraction: u128, //fraction of min_bet that can be held as minimum balance for user
    pub max_odds: u8,
    pub min_odds: u8,

    pub game_count: u128,

    #[serde(skip)]
    pub accounts: LookupMap<AccountId, Account>,
    #[serde(skip)]
    pub games: LookupMap<String, PartneredGame>,
    #[serde(skip)]
    pub nft_balance: UnorderedMap<AccountId, u128>,
    #[serde(skip)]
    pub owner_balance: UnorderedMap<AccountId, u128>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        nft_account: AccountId,
        bet_payment_adjustment: U128,
        nft_fee: U128,
        owner_fee: U128,
        house_fee: U128,
        max_bet: U128,
        min_bet: U128,
        min_balance_fraction: U128,
        max_odds: U128,
        min_odds: U128,
    ) -> Self {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Invalid owner account"
        );
        assert!(!env::state_exists(), "Already initialized");
        let mut contract = Self {
            owner_id,
            nft_account,
            panic_button: false,
            bet_payment_adjustment: bet_payment_adjustment.0, // base 10e-5

            nft_fee: nft_fee.0,     // 4000
            owner_fee: owner_fee.0, // 500
            house_fee: house_fee.0, // 500

            nft_balance: UnorderedMap::new(StorageKey::NftFunds),
            owner_balance: UnorderedMap::new(StorageKey::OwnerFunds),

            max_bet: max_bet.0,
            min_bet: min_bet.0,
            min_balance_fraction: min_balance_fraction.0,
            max_odds: u8::try_from(max_odds.0).unwrap(),
            min_odds: u8::try_from(min_odds.0).unwrap(),

            game_count: 0,

            accounts: LookupMap::new(StorageKey::Accounts),
            games: LookupMap::new(StorageKey::PartneredGames),
        };
        let contract_address = env::current_account_id();
        let mut contract_account_entry = Account::new(&contract_address, env::account_balance());
        contract.internal_update_account(&contract_address, &contract_account_entry);
        contract_account_entry.track_storage_usage(0);
        contract.internal_update_account(&contract_address, &contract_account_entry);
        contract
    }
}

// account related methods
impl Contract {
    pub fn internal_get_account(&self, account_id: &AccountId) -> Option<Account> {
        self.accounts.get(account_id)
    }

    pub fn internal_update_account(&mut self, account_id: &AccountId, account: &Account) {
        self.accounts.insert(account_id, account);
    }

    pub fn internal_deposit_storage_account(&mut self, account_id: &AccountId, deposit: u128) {
        let account = match self.internal_get_account(account_id) {
            Some(mut account) => {
                account.deposit_storage_funds(deposit);
                account
            }
            None => Account::new(&account_id.clone(), deposit),
        };
        self.accounts.insert(account_id, &account);
    }

    pub fn internal_storage_withdraw_account(
        &mut self,
        account_id: &AccountId,
        amount: u128,
    ) -> u128 {
        let mut account = self.internal_get_account(&account_id).expect(ERR_001);
        let available = account.storage_funds_available();
        assert!(
            available > 0,
            "{}. No funds available for withdraw",
            ERR_101
        );
        let mut withdraw_amount = amount;
        if amount == 0 {
            withdraw_amount = available;
        }
        assert!(
            withdraw_amount <= available,
            "{}. Only {} available for withdraw",
            ERR_101,
            available
        );
        account.withdraw_storage_funds(withdraw_amount);
        self.internal_update_account(account_id, &account);
        withdraw_amount
    }
}

// partnered_game related methods
impl Contract {
    pub fn internal_get_game(&self, code: &String) -> Option<PartneredGame> {
        self.games.get(code)
    }

    pub fn internal_update_game(&mut self, code: &String, game: &PartneredGame) {
        self.games.insert(code, game);
    }
}

// helper methods
impl Contract {
    fn assert_panic_button(&self) {
        assert!(
            !self.panic_button,
            "Panic mode is on, contract has been paused by owner"
        );
    }

    fn only_owner(&self) {
        assert_one_yocto();
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "Only owner can call this function"
        );
    }
}

// use the attribute below for unit tests
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::MockedBlockchain;
//     use near_sdk::{testing_env, VMContext};

//     pub const CONTRACT_ACCOUNT: &str = "contract.testnet";
//     pub const NFT_ACCOUNT: &str = "nft.testnet";
//     pub const SIGNER_ACCOUNT: &str = "signer.testnet";
//     pub const OWNER_ACCOUNT: &str = "owner.testnet";

//     pub fn get_context(input: Vec<u8>, is_view: bool, attached_deposit: u128, account_balance: u128, signer_id: AccountId) -> VMContext {
//         VMContext {
//             current_account_id: CONTRACT_ACCOUNT.to_string(),
//             signer_account_id: signer_id.clone(),
//             signer_account_pk: vec![0, 1, 2],
//             predecessor_account_id: signer_id.clone(),
//             input,
//             block_index: 0,
//             block_timestamp: 0,
//             account_balance,
//             account_locked_balance: 0,
//             storage_usage: 0,
//             attached_deposit,
//             prepaid_gas: 10u64.pow(18),
//             random_seed: vec![0, 1, 2],
//             is_view,
//             output_data_receivers: vec![],
//             epoch_height: 19,
//         }
//     }

//     pub fn sample_contract() -> Contract {
//         Contract {
//             owner_id: OWNER_ACCOUNT.to_string(),
//             nft_account: NFT_ACCOUNT.to_string(),
//             panic_button: false,
//             bet_payment_adjustment: 100, // base 10e-5
//             nft_fee: 300, // base 10e-5
//             owner_fee: 100, // base 10e-5
//             house_fee: 100,
//             nft_balance: UnorderedMap::new(StorageKey::NftFunds),
//             owner_balance: UnorderedMap::new(StorageKey::OwnerFunds),
//             max_bet: 100_000_000,
//             min_bet: 100_000,
//             min_balance_fraction: 100, //fraction of min_bet that can be held as minimum balance for user
//             max_odds: 200,
//             min_odds: 20,
//             game_count: 0,

//             accounts: LookupMap::new(StorageKey::Accounts),
//             games: LookupMap::new(StorageKey::PartneredGames),
//         }
//     }

//     #[test]
//     fn test_constructor() {
//         // set up the mock context into the testing environment
//         const BASE_DEPOSIT: u128 = 10_000_000;
//         let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
//         testing_env!(context);
//         // instantiate a contract variable with the counter at zero
//         let initialized_value = Contract::new(
//             OWNER_ACCOUNT.to_string(),
//             NFT_ACCOUNT.to_string(),
//             U128(100),
//             U128(300),
//             U128(100),
//             U128(100),
//             U128(0),
//             U128(100_000_000),
//             U128(100_000),
//             U128(100), //fraction of min_bet that can be held as minimum balance for user
//             U128(200),
//             U128(20)
//         );

//         let sample_contract = sample_contract();
//         assert_eq!(initialized_value.owner_id, sample_contract.owner_id);
//         assert_eq!(initialized_value.nft_account, sample_contract.nft_account);
//         assert_eq!(initialized_value.panic_button, sample_contract.panic_button);
//         assert_eq!(initialized_value.bet_payment_adjustment, sample_contract.bet_payment_adjustment);
//         assert_eq!(initialized_value.nft_fee, sample_contract.nft_fee);
//         assert_eq!(initialized_value.owner_fee, sample_contract.owner_fee);
//         assert_eq!(initialized_value.house_fee, sample_contract.house_fee);
//         assert_eq!(initialized_value.nft_balance, sample_contract.nft_balance);
//         assert_eq!(initialized_value.owner_balance, sample_contract.owner_balance);
//         assert_eq!(initialized_value.max_bet, sample_contract.max_bet);
//         assert_eq!(initialized_value.min_bet, sample_contract.min_bet);
//         assert_eq!(initialized_value.min_balance_fraction, sample_contract.min_balance_fraction);
//         assert_eq!(initialized_value.max_odds, sample_contract.max_odds);
//         assert_eq!(initialized_value.min_odds, sample_contract.min_odds);
//     }

// }
