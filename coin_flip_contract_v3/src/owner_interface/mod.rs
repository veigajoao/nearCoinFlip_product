use near_sdk::{
    AccountId, Promise,
    json_types::{ U128, },
};
use std::collections::HashMap;

pub mod owner_interface_impl;

//offers methods responsible for depositing assets and playing the game
pub trait OwnerInterface {
    
    fn emergency_panic(&mut self) -> bool;

    //retrieves contract state variables
    fn get_contract_state(&self) -> HashMap<String, String>;

    //update contracts state variables
    fn update_contract(&mut self, bet_payment_adjustment: U128, nft_fee: U128, owner_fee: U128, house_fee: U128, 
        max_bet: U128, min_bet: U128, min_balance_fraction: U128, max_odds: U128, min_odds: U128) -> bool;
    
    //retrieve contract funds
    fn retrieve_owner_funds(&mut self) -> Promise;
    fn retrieve_nft_funds(&mut self) -> Promise;
    
    //create new partnered game
    fn create_new_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, token_contract: AccountId, partner_fee: U128) -> bool;

    fn alter_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, partner_fee: U128, blocked: bool) -> bool;
    
}