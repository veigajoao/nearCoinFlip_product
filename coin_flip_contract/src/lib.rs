use std::convert::TryInto;
use std::convert::TryFrom;

use near_sdk::{ borsh };
use borsh::{ BorshDeserialize, BorshSerialize };
use near_sdk::{
    env, near_bindgen, AccountId, Balance, Promise,
    collections::{ LookupMap },
    json_types::{ U128 },
    utils::assert_one_yocto
};

mod game_interface;

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

// const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const FRACTIONAL_BASE: u128 = 100_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PartneredGame {
    pub blocked: bool,
    pub affiliate_fee: u128, // base 10e-5
    pub affiliate_balance: u128,
    pub user_balance_lookup: LookupMap<AccountId, u128>
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SlotMachine {
    pub owner_id: AccountId,
    pub panic_button: bool,
    pub bet_payment_adjustment: u128, // base 10e-5

    pub nft_fee: u128, // base 10e-5
    pub owner_fee: u128, // base 10e-5
    pub house_fee: u128,

    pub nft_balance: u128,
    pub dev_balance: u128,
    pub house_balance: u128,

    pub max_bet: u128,
    pub min_bet: u128,
    pub min_balance_fraction: u128, //fraction of min_bet that can be held as minimum balance for user
    pub max_odds: u8,
    pub min_odds: u8,

    pub game_structs: LookupMap<AccountId, PartneredGame>
   
}

impl Default for SlotMachine {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl SlotMachine {
    #[init]
    pub fn new(owner_id: AccountId, bet_payment_adjustment:U128,  nft_fee: U128, owner_fee: U128,
               house_fee: U128, house_balance:U128, win_multiplier: U128, max_bet: U128,
               min_bet: U128, min_balance_fraction: U128, max_odds: U128, min_odds: U128) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Invalid owner account");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id,
            panic_button: false,
            bet_payment_adjustment: bet_payment_adjustment.0, // base 10e-5

            nft_fee: nft_fee.0, // 4000
            owner_fee: owner_fee.0, // 500
            house_fee: house_fee.0, // 500

            nft_balance: 0,
            dev_balance: 0,
            house_balance: house_balance.0,

            max_bet: max_bet.0,
            min_bet: min_bet.0,
            min_balance_fraction: min_balance_fraction.0,
            max_odds: u8::try_from(max_odds.0).unwrap(),
            min_odds: u8::try_from(min_odds.0).unwrap(),

            game_structs: LookupMap::new(b"game_structs".to_vec())
        }
    }

    //below here will exclude all

    // retrieve dev funds function
    #[payable]
    pub fn retrieve_dev_funds(&mut self) -> Promise {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();

        let dev_account_id = self.owner_id.clone();
        let withdrawal_dev_balance = self.dev_balance.clone();
        self.dev_balance = 0;

        Promise::new(dev_account_id).transfer(withdrawal_dev_balance)
    }

    // //adapt to cross contract calls
    // #[payable]
    // pub fn retrieve_nft_funds(&mut self, distribution_list: Vec<String>) {
    //     assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");
    //     assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
    //     assert_one_yocto();

    //     let withdrawal_nft_balance = self.nft_balance.clone();
    //     self.nft_balance = 0;
    //     let collection_size: u128 = distribution_list.len().try_into().unwrap(); 
    //     let piece_nft_balance: u128 = withdrawal_nft_balance / collection_size;

    //     for item in distribution_list.into_iter() {
    //         Promise::new(
    //             item
    //         ).transfer(piece_nft_balance);
    //     }
    // }

    // //update contract initialization vars
    // #[payable]
    // pub fn update_contract(&mut self, nft_fee: U128, dev_fee: U128, house_fee: U128, win_multiplier: U128, max_bet: U128, min_bet: U128, min_balance_fraction: U128) {
    //     assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
    //     assert_one_yocto();

    //     self.nft_fee = nft_fee.0;
    //     self.dev_fee = dev_fee.0;
    //     self.house_fee = house_fee.0;
    //     self.win_multiplier = win_multiplier.0;
    //     self.max_bet = max_bet.0;
    //     self.min_bet = min_bet.0;
    //     self.min_balance_fraction = min_balance_fraction.0;
    // }

    // //return current contract state
    // pub fn get_contract_state(&self) -> std::collections::HashMap<String, String> {
    //     let mut state = std::collections::HashMap::new();
        
    //     state.insert(String::from("owner_id"), self.owner_id.to_string());
    //     state.insert(String::from("nft_fee"), self.nft_fee.to_string());
    //     state.insert(String::from("dev_fee"), self.dev_fee.to_string());
    //     state.insert(String::from("house_fee"), self.house_fee.to_string());
    //     state.insert(String::from("win_multiplier"), self.win_multiplier.to_string());
    //     state.insert(String::from("nft_balance"), self.nft_balance.to_string());
    //     state.insert(String::from("dev_balance"), self.dev_balance.to_string());
    //     state.insert(String::from("max_bet"), self.max_bet.to_string());
    //     state.insert(String::from("min_bet"), self.min_bet.to_string());
    //     state.insert(String::from("min_balance_fraction"), self.min_balance_fraction.to_string());
        
    //     state
    // }

    // #[payable]
    // pub fn emergency_panic(&mut self, withdrawal_balance: U128) -> Promise {
    //     assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
    //     assert_one_yocto();

    //     if self.panic_button {
    //         self.panic_button = false;
    //     } else {
    //         self.panic_button = true;
    //     }
        
    //     Promise::new(
    //         self.owner_id.clone()
    //     ).transfer(withdrawal_balance.0)
    // }
}


// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    const CONTRACT_ACCOUNT: &str = "contract.testnet";
    const SIGNER_ACCOUNT: &str = "signer.testnet";
    const OWNER_ACCOUNT: &str = "owner.testnet";

    fn get_context(input: Vec<u8>, is_view: bool, attached_deposit: u128, account_balance: u128) -> VMContext {
        VMContext {
            current_account_id: CONTRACT_ACCOUNT.to_string(),
            signer_account_id:  SIGNER_ACCOUNT.to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id:  SIGNER_ACCOUNT.to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn test_deposit_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 10_000_000;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0);
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            credits: LookupMap::new(b"credits".to_vec()),
            nft_fee: 400, // base 10e-5
            dev_fee: 10, // base 10e-5
            house_fee: 10,
            win_multiplier: 200000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0,
            max_bet: 100_000_000,
            min_bet: 100_000,
            min_balance_fraction: 100,
            panic_button: false
        };
        let user_balance1: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value before deposit: {}", &user_balance1);
        contract.deposit();
        let user_balance2: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value after deposit: {}", &user_balance2);
        // confirm that we received 1 when calling get_num
        assert_eq!(BASE_DEPOSIT, user_balance2);
    }

}