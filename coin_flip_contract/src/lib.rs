use std::convert::TryInto;

use near_sdk::{ borsh };
use borsh::{ BorshDeserialize, BorshSerialize };
use near_sdk::{
    env, near_bindgen, AccountId, Balance, Promise,
    collections::{ LookupMap },
    json_types::{ U128 },
    utils::assert_one_yocto
};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

// const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
const PROB: u8 = 128;
const FRACTIONAL_BASE: u128 = 100_000;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
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
    pub house_balance: u128

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
               min_bet: U128, min_balance_fraction: U128, max_odds: U8, min_odds: U8) -> Self {
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
            house_balance: house_balance.0

            max_bet: max_bet.0,
            min_bet: min_bet.0,
            min_balance_fraction: min_balance_fraction.0,
            max_odds: max_odds.0,
            min_odds: min_odds.0,

            game_structs: LookupMap::new(b"game_structs".to_vec())
        }
    }

    //below here will exclude all

    //retrieve dev funds function
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

    //adapt to cross contract calls
    #[payable]
    pub fn retrieve_nft_funds(&mut self, distribution_list: Vec<String>) {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();

        let withdrawal_nft_balance = self.nft_balance.clone();
        self.nft_balance = 0;
        let collection_size: u128 = distribution_list.len().try_into().unwrap(); 
        let piece_nft_balance: u128 = withdrawal_nft_balance / collection_size;

        for item in distribution_list.into_iter() {
            Promise::new(
                item
            ).transfer(piece_nft_balance);
        }
    }

    //update contract initialization vars
    #[payable]
    pub fn update_contract(&mut self, nft_fee: U128, dev_fee: U128, house_fee: U128, win_multiplier: U128, max_bet: U128, min_bet: U128, min_balance_fraction: U128) {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();

        self.nft_fee = nft_fee.0;
        self.dev_fee = dev_fee.0;
        self.house_fee = house_fee.0;
        self.win_multiplier = win_multiplier.0;
        self.max_bet = max_bet.0;
        self.min_bet = min_bet.0;
        self.min_balance_fraction = min_balance_fraction.0;
    }

    //return current contract state
    pub fn get_contract_state(&self) -> std::collections::HashMap<String, String> {
        let mut state = std::collections::HashMap::new();
        
        state.insert(String::from("owner_id"), self.owner_id.to_string());
        state.insert(String::from("nft_fee"), self.nft_fee.to_string());
        state.insert(String::from("dev_fee"), self.dev_fee.to_string());
        state.insert(String::from("house_fee"), self.house_fee.to_string());
        state.insert(String::from("win_multiplier"), self.win_multiplier.to_string());
        state.insert(String::from("nft_balance"), self.nft_balance.to_string());
        state.insert(String::from("dev_balance"), self.dev_balance.to_string());
        state.insert(String::from("max_bet"), self.max_bet.to_string());
        state.insert(String::from("min_bet"), self.min_bet.to_string());
        state.insert(String::from("min_balance_fraction"), self.min_balance_fraction.to_string());
        
        state
    }

    #[payable]
    pub fn emergency_panic(&mut self, withdrawal_balance: U128) -> Promise {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();

        if self.panic_button {
            self.panic_button = false;
        } else {
            self.panic_button = true;
        }
        
        Promise::new(
            self.owner_id.clone()
        ).transfer(withdrawal_balance.0)
    }
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

    #[test]
    #[should_panic]
    fn test_deposit_function_minimum() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 999;
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
        contract.deposit();
    }

    #[test]
    fn test_withdrawal_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 48_000;
        const CONTRACT_BALANCE: u128 = 1_000_000_000_000_000;
        const WITHDRAWAL_AMOUNT: u128 = 48_000;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
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
    
        contract.credits.insert(&SIGNER_ACCOUNT.to_string(), &WITHDRAWAL_AMOUNT);
        let user_balance1: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value before withdrawal: {}", &user_balance1);
        contract.retrieve_credits();
        let user_balance2: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value after withdrawal: {}", &user_balance2);
        // confirm that we received 1 when calling get_num
        assert_eq!(WITHDRAWAL_AMOUNT, user_balance1);
        assert_eq!(0, user_balance2);
    }

    #[test]
    fn test_get_credits_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
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
        
        const BALANCE_AMOUNT: u128 = 48_000;
        contract.credits.insert(&SIGNER_ACCOUNT.to_string(), &BALANCE_AMOUNT);
        let user_balance: u128 =  contract.get_credits(SIGNER_ACCOUNT.clone().to_string()).into();

        assert_eq!(BALANCE_AMOUNT, user_balance);
    }

    #[test]
    fn test_get_credits_function_assert_view() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], true, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let contract =  SlotMachine {
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
        
        let user_balance: u128 =  contract.get_credits(SIGNER_ACCOUNT.clone().to_string()).into();

        assert_eq!(0, user_balance);
    }

    //missing:
    // play

    #[test]
    fn test_play_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            credits: LookupMap::new(b"credits".to_vec()),
            nft_fee: 4000, // base 10e-5
            dev_fee: 500, // base 10e-5
            house_fee: 500,
            win_multiplier: 20000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0,
            max_bet: 100_000_000,
            min_bet: 100_000,
            min_balance_fraction: 100,
            panic_button: false
        };
        println!("Game won: {}", 20000 / FRACTIONAL_BASE);
        const BALANCE_AMOUNT: u128 = 100_000_000;
        contract.credits.insert(&SIGNER_ACCOUNT.to_string(), &BALANCE_AMOUNT);

        const BET_AMOUNT: u128 = 100_000;

        let mut start_balance: u128;
        let mut end_balance: u128;
        let mut game_won: bool;

        let dev_fee: u128 = (&BET_AMOUNT * contract.dev_fee) / FRACTIONAL_BASE;
        let nft_fee: u128 = (&BET_AMOUNT * contract.nft_fee) / FRACTIONAL_BASE;
        let house_fee: u128 = (&BET_AMOUNT * contract.house_fee) / FRACTIONAL_BASE;
        let net_bet: u128 = BET_AMOUNT.clone() - dev_fee - nft_fee - house_fee;
        let net_won: u128 = (net_bet * contract.win_multiplier) / FRACTIONAL_BASE ;

        let total_count: u128 = 30;
        let mut loop_counter: u128 = 0;
        while loop_counter < total_count {

            start_balance = contract.get_credits(SIGNER_ACCOUNT.clone().to_string()).into();
            game_won = contract.play(true, U128(BET_AMOUNT));
            end_balance = contract.get_credits(SIGNER_ACCOUNT.clone().to_string()).into();
                
            if game_won {
                assert_eq!(start_balance - BET_AMOUNT + net_won, end_balance, "user balance doesn't match play result");
            } else {
                assert_eq!(start_balance - BET_AMOUNT, end_balance, "user balance doesn't match play result");
            }
            loop_counter = loop_counter + 1;
        }
        
        assert_eq!(contract.nft_balance, nft_fee * total_count, "nft_fee failure");
        assert_eq!(contract.dev_balance, dev_fee * total_count, "dev_fee failure");
    }

    #[test]
    #[should_panic]
    fn test_play_function_panic_min_bet() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            credits: LookupMap::new(b"credits".to_vec()),
            nft_fee: 4000, // base 10e-5
            dev_fee: 500, // base 10e-5
            house_fee: 500,
            win_multiplier: 20000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0,
            max_bet: 100_000_000,
            min_bet: 100_000,
            min_balance_fraction: 100,
            panic_button: false
        };
        const BALANCE_AMOUNT: u128 = 100_000_000;
        contract.credits.insert(&SIGNER_ACCOUNT.to_string(), &BALANCE_AMOUNT);

        const BET_AMOUNT: u128 = 99_999;
        contract.play(true, U128(BET_AMOUNT));
    }

    #[test]
    #[should_panic]
    fn test_play_function_panic_max_bet() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            credits: LookupMap::new(b"credits".to_vec()),
            nft_fee: 4000, // base 10e-5
            dev_fee: 500, // base 10e-5
            house_fee: 500,
            win_multiplier: 20000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0,
            max_bet: 100_000_000,
            min_bet: 100_000,
            min_balance_fraction: 100,
            panic_button: false
        };
        const BALANCE_AMOUNT: u128 = 100_000_000_000;
        contract.credits.insert(&SIGNER_ACCOUNT.to_string(), &BALANCE_AMOUNT);

        const BET_AMOUNT: u128 = 100_000_001;
        contract.play(true, U128(BET_AMOUNT));
    }

    // update contract
    // assert panic when no owner calls
    // assert change when owner calls
    #[test]
    #[should_panic]
    fn test_update_contract_function_assert_panic_no_owner() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            credits: LookupMap::new(b"credits".to_vec()),
            nft_fee: 4000, // base 10e-5
            dev_fee: 500, // base 10e-5
            house_fee: 500,
            win_multiplier: 20000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0,
            max_bet: 100_000_000,
            min_bet: 100_000,
            min_balance_fraction: 100, 
            panic_button: false
        };
        
        contract.update_contract(U128(10), U128(11), U128(12), U128(13), U128(15), U128(16), U128(17));
    }

    #[test]
    fn test_update_contract_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 1;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: SIGNER_ACCOUNT.to_string(),
            credits: LookupMap::new(b"credits".to_vec()),
            nft_fee: 4000, // base 10e-5
            dev_fee: 500, // base 10e-5
            house_fee: 500,
            win_multiplier: 20000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0,
            max_bet: 100_000_000,
            min_bet: 100_000,
            min_balance_fraction: 100,
            panic_button: false
        };
        
        contract.update_contract(U128(10), U128(11), U128(12), U128(13), U128(15), U128(16), U128(17));

        assert_eq!(contract.nft_fee, 10, "nft_fee");
        assert_eq!(contract.dev_fee, 11, "dev_fee");
        assert_eq!(contract.house_fee, 12, "house_fee");
        assert_eq!(contract.win_multiplier, 13, "win_multiplier");
        assert_eq!(contract.max_bet, 15, "max_bet");
        assert_eq!(contract.min_bet, 16, "min_bet");
        assert_eq!(contract.min_balance_fraction, 17, "min_balance_fraction");
        
    }

    #[test]
    fn test_get_contract_state() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let contract =  SlotMachine {
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
        
        let contract_copy: std::collections::HashMap<String, String> =  contract.get_contract_state();

        assert_eq!(contract_copy.get("owner_id").unwrap().clone(), contract.owner_id.to_string());

        assert_eq!(contract_copy.get("nft_fee").unwrap().clone(), contract.nft_fee.to_string());
        assert_eq!(contract_copy.get("dev_fee").unwrap().clone(), contract.dev_fee.to_string());
        assert_eq!(contract_copy.get("house_fee").unwrap().clone(), contract.house_fee.to_string());
        assert_eq!(contract_copy.get("win_multiplier").unwrap().clone(), contract.win_multiplier.to_string());
        assert_eq!(contract_copy.get("nft_balance").unwrap().clone(), contract.nft_balance.to_string());
        assert_eq!(contract_copy.get("dev_balance").unwrap().clone(), contract.dev_balance.to_string());
        assert_eq!(contract_copy.get("max_bet").unwrap().clone(), contract.max_bet.to_string());
        assert_eq!(contract_copy.get("min_bet").unwrap().clone(), contract.min_bet.to_string());
        assert_eq!(contract_copy.get("min_balance_fraction").unwrap().clone(), contract.min_balance_fraction.to_string());
        
    }

    //functions that use cross contract calls are tested using sim-tests
}