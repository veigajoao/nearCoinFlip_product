use crate::*;
use super::OwnerInterface;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, Promise,
    collections::{ LookupMap },
    json_types::{ U128 },
    utils::assert_one_yocto
};
use near_sdk::{ borsh };
use borsh::{ BorshDeserialize, BorshSerialize };
use std::convert::TryFrom;
use std::collections::HashMap;

#[near_bindgen]
impl OwnerInterface for SlotMachine {
    
    fn emergency_panic(&mut self) -> bool {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        self.panic_button = !self.panic_button;
        self.panic_button
    }

    //retrieves contract state variables
    fn get_contract_state(&self) -> std::collections::HashMap<String, String> {
        let mut state = std::collections::HashMap::new();

        state.insert(String::from("owner_id"), self.owner_id.to_string());
        state.insert(String::from("panic_button"), self.panic_button.to_string());
        state.insert(String::from("bet_payment_adjustment"), self.bet_payment_adjustment.to_string());
        
        state.insert(String::from("nft_fee"), self.nft_fee.to_string());
        state.insert(String::from("owner_fee"), self.owner_fee.to_string());
        state.insert(String::from("house_fee"), self.house_fee.to_string());
        state.insert(String::from("nft_balance"), self.nft_balance.to_string());
        state.insert(String::from("owner_balance"), self.owner_balance.to_string());
        state.insert(String::from("house_balance"), self.house_balance.to_string());
        
        state.insert(String::from("max_bet"), self.max_bet.to_string());
        state.insert(String::from("min_bet"), self.min_bet.to_string());
        state.insert(String::from("min_balance_fraction"), self.min_balance_fraction.to_string());
        state.insert(String::from("max_odds"), self.max_odds.to_string());
        state.insert(String::from("min_odds"), self.min_odds.to_string());
        
        state
    }

    //update contracts state variables
    fn update_contract(&mut self, bet_payment_adjustment: U128, nft_fee: U128, owner_fee: U128, house_fee: U128, 
                    max_bet: U128, min_bet: U128, min_balance_fraction: U128, max_odds: U128, min_odds: U128) -> bool {
        
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();

        self.bet_payment_adjustment = bet_payment_adjustment.0;

        self.nft_fee = nft_fee.0;
        self.owner_fee = owner_fee.0;
        self.house_fee = house_fee.0;

        self.max_bet = max_bet.0;
        self.min_bet = min_bet.0;
        self.min_balance_fraction = min_balance_fraction.0;
        self.max_odds = u8::try_from(max_odds.0).unwrap();
        self.min_odds = u8::try_from(min_odds.0).unwrap();

        true
    }
    
    //retrieve contract funds
    fn retrieve_owner_funds(&mut self, amount: U128) -> Promise {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(self.owner_balance >= amount.0, "insufficient balance for this withdrawal");
        self.owner_balance = self.owner_balance - amount.0;
        Promise::new(self.owner_id.clone()).transfer(amount.0)
    }

    fn retrieve_nft_funds(&mut self, amount: U128) -> Promise {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(self.nft_balance >= amount.0, "insufficient balance for this withdrawal");
        self.nft_balance = self.nft_balance - amount.0;
        Promise::new(self.nft_account.clone()).transfer(amount.0)
    }

    fn retrieve_house_funds(&mut self, amount: U128) -> Promise {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(self.house_balance >= amount.0, "insufficient balance for this withdrawal");
        self.house_balance = self.house_balance - amount.0;
        Promise::new(self.owner_id.clone()).transfer(amount.0)
    }

    //create new partnered game
    fn create_new_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, partner_fee: U128, 
                            only_nft_owners_can_play: bool) -> bool {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(!self.game_structs.contains_key(&nft_contract), "collections is already initialized");

        let game_settings = PartneredGame {
            partner_owner,
            only_nft_owners_can_play,
            blocked: false,
            partner_fee: partner_fee.0, // base 10e-5
            partner_balance: 0,
        };
        self.game_balances.insert(&nft_contract, &LookupMap::new(format!("{}{}", nft_contract, "game_struct".to_string()).into_bytes() ) );
        self.game_structs.insert(&nft_contract, &game_settings);
        true
    }

    fn alter_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, partner_fee: U128, 
                        only_nft_owners_can_play: bool , blocked: bool) -> bool {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(self.game_structs.contains_key(&nft_contract), "Collections isn't yet initialized");
        let game_settings = PartneredGame {
            partner_owner,
            only_nft_owners_can_play,
            blocked,
            partner_fee: partner_fee.0, // base 10e-5
            partner_balance: 0,
        };
        self.game_structs.insert(&nft_contract, &game_settings);
        true

    }
    
}
