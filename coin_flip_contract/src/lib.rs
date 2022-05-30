use std::convert::TryFrom;

use near_sdk::{ borsh };
use borsh::{ BorshDeserialize, BorshSerialize };
use near_sdk::{
    env, near_bindgen, AccountId,
    collections::{ LookupMap },
    json_types::{ U128 },
};

pub mod game_interface;
pub mod owner_interface;
pub mod partner_interface;

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

// const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const FRACTIONAL_BASE: u128 = 100_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PartneredGame {
    pub partner_owner: AccountId,
    pub blocked: bool,
    pub partner_fee: u128, // base 10e-5
    pub partner_balance: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SlotMachine {
    pub owner_id: AccountId,
    pub nft_account: AccountId,
    pub panic_button: bool,
    pub bet_payment_adjustment: u128, // base 10e-5

    pub nft_fee: u128, // base 10e-5
    pub owner_fee: u128, // base 10e-5
    pub house_fee: u128,

    pub nft_balance: u128,
    pub owner_balance: u128,
    pub house_balance: u128,

    pub max_bet: u128,
    pub min_bet: u128,
    pub min_balance_fraction: u128, //fraction of min_bet that can be held as minimum balance for user
    pub max_odds: u8,
    pub min_odds: u8,

    pub game_count: u128,

    pub game_structs: LookupMap<AccountId, PartneredGame>,
    pub game_balances: LookupMap<AccountId, LookupMap<AccountId, u128>>
   
}

impl Default for SlotMachine {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl SlotMachine {
    #[init]
    pub fn new(owner_id: AccountId, nft_account: AccountId, bet_payment_adjustment:U128,  nft_fee: U128, 
               owner_fee: U128, house_fee: U128, house_balance:U128, max_bet: U128,
               min_bet: U128, min_balance_fraction: U128, max_odds: U128, min_odds: U128) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Invalid owner account");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id,
            nft_account,
            panic_button: false,
            bet_payment_adjustment: bet_payment_adjustment.0, // base 10e-5

            nft_fee: nft_fee.0, // 4000
            owner_fee: owner_fee.0, // 500
            house_fee: house_fee.0, // 500

            nft_balance: 0,
            owner_balance: 0,
            house_balance: house_balance.0,

            max_bet: max_bet.0,
            min_bet: min_bet.0,
            min_balance_fraction: min_balance_fraction.0,
            max_odds: u8::try_from(max_odds.0).unwrap(),
            min_odds: u8::try_from(min_odds.0).unwrap(),

            game_count: 0,

            game_structs: LookupMap::new(b"game_structs".to_vec()),
            game_balances: LookupMap::new(b"game_balances".to_vec())
        }
    }

}

impl SlotMachine {
    fn assert_panic_button(&self) {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");
    }

    fn only_owner(&self) {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
    }
}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    pub const CONTRACT_ACCOUNT: &str = "contract.testnet";
    pub const NFT_ACCOUNT: &str = "nft.testnet";
    pub const SIGNER_ACCOUNT: &str = "signer.testnet";
    pub const OWNER_ACCOUNT: &str = "owner.testnet";

    pub fn get_context(input: Vec<u8>, is_view: bool, attached_deposit: u128, account_balance: u128, signer_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: CONTRACT_ACCOUNT.to_string(),
            signer_account_id: signer_id.clone(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: signer_id.clone(),
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

    pub fn sample_contract() -> SlotMachine {
        SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            nft_account: NFT_ACCOUNT.to_string(),
            panic_button: false,
            bet_payment_adjustment: 100, // base 10e-5
    
            nft_fee: 300, // base 10e-5
            owner_fee: 100, // base 10e-5
            house_fee: 100,
    
            nft_balance: 0,
            owner_balance: 0,
            house_balance: 0,
    
            max_bet: 100_000_000,
            min_bet: 100_000,
            min_balance_fraction: 100, //fraction of min_bet that can be held as minimum balance for user
            max_odds: 200,
            min_odds: 20,
            
            game_count: 0,

            game_structs: LookupMap::new(b"game_structs".to_vec()),
            game_balances: LookupMap::new(b"game_balances".to_vec())
        }
    }

    #[test]
    fn test_constructor() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 10_000_000;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let initialized_value = SlotMachine::new(
            OWNER_ACCOUNT.to_string(),
            NFT_ACCOUNT.to_string(),
            U128(100),
            U128(300),
            U128(100),
            U128(100),
            U128(0),
            U128(100_000_000),
            U128(100_000),
            U128(100), //fraction of min_bet that can be held as minimum balance for user
            U128(200),
            U128(20)
        );

        let sample_contract = sample_contract();
       
        assert_eq!(initialized_value.owner_id, sample_contract.owner_id);
        assert_eq!(initialized_value.nft_account, sample_contract.nft_account);
        assert_eq!(initialized_value.panic_button, sample_contract.panic_button);
        assert_eq!(initialized_value.bet_payment_adjustment, sample_contract.bet_payment_adjustment);
        assert_eq!(initialized_value.nft_fee, sample_contract.nft_fee);
        assert_eq!(initialized_value.owner_fee, sample_contract.owner_fee);
        assert_eq!(initialized_value.house_fee, sample_contract.house_fee);
        assert_eq!(initialized_value.nft_balance, sample_contract.nft_balance);
        assert_eq!(initialized_value.owner_balance, sample_contract.owner_balance);
        assert_eq!(initialized_value.house_balance, sample_contract.house_balance);
        assert_eq!(initialized_value.max_bet, sample_contract.max_bet);
        assert_eq!(initialized_value.min_bet, sample_contract.min_bet);
        assert_eq!(initialized_value.min_balance_fraction, sample_contract.min_balance_fraction);
        assert_eq!(initialized_value.max_odds, sample_contract.max_odds);
        assert_eq!(initialized_value.min_odds, sample_contract.min_odds);
    }

}