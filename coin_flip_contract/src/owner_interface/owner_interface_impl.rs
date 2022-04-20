use crate::*;
use super::OwnerInterface;
use near_sdk::{
    env, near_bindgen, AccountId, Promise,
    utils::assert_one_yocto
};
use near_sdk::{ borsh };
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
        state.insert(String::from("nft_account"), self.nft_account.to_string());
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
        assert!(self.owner_balance >= amount.0, "Insufficient balance for this withdrawal");
        self.owner_balance = self.owner_balance - amount.0;
        Promise::new(self.owner_id.clone()).transfer(amount.0)
    }

    fn retrieve_nft_funds(&mut self, amount: U128) -> Promise {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(self.nft_balance >= amount.0, "Insufficient balance for this withdrawal");
        self.nft_balance = self.nft_balance - amount.0;
        Promise::new(self.nft_account.clone()).transfer(amount.0)
    }

    fn retrieve_house_funds(&mut self, amount: U128) -> Promise {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(self.house_balance >= amount.0, "Insufficient balance for this withdrawal");
        self.house_balance = self.house_balance - amount.0;
        Promise::new(self.owner_id.clone()).transfer(amount.0)
    }

    //create new partnered game
    fn create_new_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, partner_fee: U128) -> bool {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(!self.game_structs.contains_key(&nft_contract), "Collection is already initialized");

        let game_settings = PartneredGame {
            partner_owner,
            blocked: false,
            partner_fee: partner_fee.0, // base 10e-5
            partner_balance: 0,
        };
        self.game_balances.insert(&nft_contract, &LookupMap::new(format!("{}{}", nft_contract, "game_struct".to_string()).into_bytes() ) );
        self.game_structs.insert(&nft_contract, &game_settings);
        true
    }

    fn alter_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, partner_fee: U128, 
                        blocked: bool) -> bool {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        assert_one_yocto();
        assert!(self.game_structs.contains_key(&nft_contract), "Collection isn't yet initialized");
        let game_settings = PartneredGame {
            partner_owner,
            blocked,
            partner_fee: partner_fee.0, // base 10e-5
            partner_balance: 0,
        };
        self.game_structs.insert(&nft_contract, &game_settings);
        true
    }
    
}

#[cfg(test)]
mod tests {

    use crate::*;
    use crate::tests::{
        get_context, sample_contract,
        SIGNER_ACCOUNT, OWNER_ACCOUNT
    };
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env};

    #[test]
    fn test_panic_button() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        assert!(!contract.panic_button, "panic button is being initialized as true");

        contract.emergency_panic();
        assert!(contract.panic_button, "panic button didn't change to true");

        contract.emergency_panic();
        assert!(!contract.panic_button, "panic button didn't change back to false");
    }

    #[test]
    #[should_panic(expected = "Only owner can call this function")]
    fn test_panic_button_only_owner() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.emergency_panic();
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_panic_button_one_yocto() {
        const BASE_DEPOSIT: u128 = 10;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.emergency_panic();
    }

    #[test]
    fn test_get_contract_state() {
        const BASE_DEPOSIT: u128 = 0;
        let context = get_context(vec![], true, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let contract = sample_contract();
        let view_call = contract.get_contract_state();

        assert_eq!(&contract.owner_id.to_string(), view_call.get("owner_id").unwrap());
        assert_eq!(&contract.nft_account.to_string(), view_call.get("nft_account").unwrap());
        assert_eq!(&contract.panic_button.to_string(), view_call.get("panic_button").unwrap());
        assert_eq!(&contract.bet_payment_adjustment.to_string(), view_call.get("bet_payment_adjustment").unwrap());
        assert_eq!(&contract.nft_fee.to_string(), view_call.get("nft_fee").unwrap());
        assert_eq!(&contract.owner_fee.to_string(), view_call.get("owner_fee").unwrap());
        assert_eq!(&contract.house_fee.to_string(), view_call.get("house_fee").unwrap());
        assert_eq!(&contract.nft_balance.to_string(), view_call.get("nft_balance").unwrap());
        assert_eq!(&contract.owner_balance.to_string(), view_call.get("owner_balance").unwrap());
        assert_eq!(&contract.house_balance.to_string(), view_call.get("house_balance").unwrap());
        assert_eq!(&contract.max_bet.to_string(), view_call.get("max_bet").unwrap());
        assert_eq!(&contract.min_bet.to_string(), view_call.get("min_bet").unwrap());
        assert_eq!(&contract.min_balance_fraction.to_string(), view_call.get("min_balance_fraction").unwrap());
        assert_eq!(&contract.max_odds.to_string(), view_call.get("max_odds").unwrap());
        assert_eq!(&contract.min_odds.to_string(), view_call.get("min_odds").unwrap());
    }

    #[test]
    fn test_update_contract() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        let new_bet_payment_adjustment = U128(42069);
        let new_nft_fee = U128(42070);
        let new_owner_fee = U128(42071);
        let new_house_fee = U128(42072);
        let new_max_bet = U128(42073);
        let new_min_bet = U128(42074);
        let new_min_balance_fraction = U128(42075);
        let new_max_odds = U128(10);
        let new_min_odds = U128(10);

        assert_ne!(contract.bet_payment_adjustment, new_bet_payment_adjustment.0);
        assert_ne!(contract.nft_fee, new_nft_fee.0);
        assert_ne!(contract.owner_fee, new_owner_fee.0);
        assert_ne!(contract.house_fee, new_house_fee.0);
        assert_ne!(contract.max_bet, new_max_bet.0);
        assert_ne!(contract.min_bet, new_min_bet.0);
        assert_ne!(contract.min_balance_fraction, new_min_balance_fraction.0);
        assert_ne!(contract.max_odds,  u8::try_from(new_max_odds.0).unwrap());
        assert_ne!(contract.min_odds,  u8::try_from(new_min_odds.0).unwrap());

        contract.update_contract(new_bet_payment_adjustment, new_nft_fee, new_owner_fee, new_house_fee, 
            new_max_bet, new_min_bet, new_min_balance_fraction, new_max_odds, new_min_odds);

        assert_eq!(contract.bet_payment_adjustment, new_bet_payment_adjustment.0);
        assert_eq!(contract.nft_fee, new_nft_fee.0);
        assert_eq!(contract.owner_fee, new_owner_fee.0);
        assert_eq!(contract.house_fee, new_house_fee.0);
        assert_eq!(contract.max_bet, new_max_bet.0);
        assert_eq!(contract.min_bet, new_min_bet.0);
        assert_eq!(contract.min_balance_fraction, new_min_balance_fraction.0);
        assert_eq!(contract.max_odds,  u8::try_from(new_max_odds.0).unwrap());
        assert_eq!(contract.min_odds,  u8::try_from(new_min_odds.0).unwrap());
    }

    #[test]
    #[should_panic(expected = "Only owner can call this function")]
    fn test_update_contract_only_owner() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.update_contract(U128(1), U128(1), U128(1), U128(1), 
        U128(1), U128(1), U128(1), U128(1), U128(1));
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_update_contract_one_yocto() {
        const BASE_DEPOSIT: u128 = 10;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.update_contract(U128(1), U128(1), U128(1), U128(1), 
        U128(1), U128(1), U128(1), U128(1), U128(1));
    }

    #[test]
    fn test_retrieve_owner_funds() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.owner_balance = 21;
        contract.retrieve_owner_funds(U128(20));
    }

    #[test]
    #[should_panic(expected = "Only owner can call this function")]
    fn test_retrieve_owner_funds_only_owner() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.retrieve_owner_funds(U128(1));
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_retrieve_owner_funds_one_yocto() {
        const BASE_DEPOSIT: u128 = 10;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.retrieve_owner_funds(U128(1));
    }

    #[test]
    #[should_panic(expected = "Insufficient balance for this withdrawal")]
    fn test_retrieve_owner_funds_min_balance() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.owner_balance = 15;
        contract.retrieve_owner_funds(U128(20));
    }

    #[test]
    fn test_retrieve_nft_funds() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.nft_balance = 21;
        contract.retrieve_nft_funds(U128(20));
    }

    #[test]
    #[should_panic(expected = "Only owner can call this function")]
    fn test_retrieve_nft_funds_only_owner() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.retrieve_nft_funds(U128(1));
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_retrieve_nft_funds_one_yocto() {
        const BASE_DEPOSIT: u128 = 10;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.retrieve_nft_funds(U128(1));
    }

    #[test]
    #[should_panic(expected = "Insufficient balance for this withdrawal")]
    fn test_retrieve_nft_funds_min_balance() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.nft_balance = 15;
        contract.retrieve_nft_funds(U128(20));
    }

    #[test]
    fn test_retrieve_house_funds() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.house_balance = 21;
        contract.retrieve_house_funds(U128(20));
    }

    #[test]
    #[should_panic(expected = "Only owner can call this function")]
    fn test_retrieve_house_funds_only_owner() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.retrieve_house_funds(U128(1));
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_retrieve_house_funds_one_yocto() {
        const BASE_DEPOSIT: u128 = 10;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.retrieve_house_funds(U128(1));
    }

    #[test]
    #[should_panic(expected = "Insufficient balance for this withdrawal")]
    fn test_retrieve_house_funds_min_balance() {
        const BASE_DEPOSIT: u128 = 1;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.house_balance = 15;
        contract.retrieve_house_funds(U128(20));
    }

    #[test]
    fn test_create_new_partner() {
        const BASE_DEPOSIT: u128 = 1;
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());

        let game_struct = contract.game_structs.get(&partner_contract).unwrap();
        assert_eq!(game_struct.partner_owner, partner_account.clone());
        assert_eq!(game_struct.blocked, false);
        assert_eq!(game_struct.partner_fee, partner_fee.clone().0);
        assert_eq!(game_struct.partner_balance, 0);
    }

    #[test]
    #[should_panic(expected = "Only owner can call this function")]
    fn test_create_new_partner_only_owner() {
        const BASE_DEPOSIT: u128 = 1;
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_create_new_partner_one_yocto() {
        const BASE_DEPOSIT: u128 = 10;
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());
    }

    #[test]
    #[should_panic(expected = "Collection is already initialized")]
    fn test_create_new_partner_already_created() {
        const BASE_DEPOSIT: u128 = 1;
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());
    }

    #[test]
    fn test_alter_partner() {
        const BASE_DEPOSIT: u128 = 1;
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());

        let game_struct = contract.game_structs.get(&partner_contract).unwrap();
        let new_owner = "new_owner.testnet".to_string();
        let new_fee = U128(50);
        let new_blocked = !game_struct.blocked;

        contract.alter_partner(new_owner.clone(), partner_contract.clone(), new_fee, new_blocked);
        let game_struct2 = contract.game_structs.get(&partner_contract).unwrap();
        
        assert_eq!(game_struct2.partner_owner, new_owner);
        assert_eq!(game_struct2.blocked, new_blocked);
        assert_eq!(game_struct2.partner_fee, new_fee.0);
        assert_eq!(game_struct2.partner_balance, game_struct.partner_balance);
    }

    #[test]
    #[should_panic(expected = "Only owner can call this function")]
    fn test_alter_partner_only_owner() {
        const BASE_DEPOSIT: u128 = 1;
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, SIGNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        let new_owner = "new_owner.testnet".to_string();
        let new_fee = U128(50);
        let new_blocked = true;

        contract.alter_partner(new_owner.clone(), partner_contract.clone(), new_fee, new_blocked);
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_alter_partner_one_yocto() {
        const BASE_DEPOSIT: u128 = 10;
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        let new_owner = "new_owner.testnet".to_string();
        let new_fee = U128(50);
        let new_blocked = true;

        contract.alter_partner(new_owner.clone(), partner_contract.clone(), new_fee, new_blocked);
    }

    #[test]
    #[should_panic(expected = "Collection isn't yet initialized")]
    fn test_alter_partner_not_yet_created() {
        const BASE_DEPOSIT: u128 = 1;
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        let new_owner = "new_owner.testnet".to_string();
        let new_fee = U128(50);
        let new_blocked = true;
        contract.alter_partner(new_owner.clone(), partner_contract.clone(), new_fee, new_blocked);
    }

}