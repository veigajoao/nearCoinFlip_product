use super::GameInterface;
use crate::FRACTIONAL_BASE;
use crate::PartneredGame;
use crate::SlotMachineContract;
use crate::SlotMachine;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, Promise,
    collections::{ LookupMap },
    json_types::{ U128 },
    utils::assert_one_yocto
};
use near_sdk::{ borsh };
use borsh::{ BorshDeserialize, BorshSerialize };
use std::convert::TryFrom;

#[near_bindgen]
impl PartnerInterface for SlotMachine {
    
   //makes contract panic on all non owner calls
   fn emergency_panic(&mut self) -> bool;

   //retrieves contract state variables
   fn get_contract_state(&self) -> std::collections::HashMap<String, String>;

   //update contracts state variables
   fn update_contract(&mut self, bet_payment_adjustment: U128, nft_fee: U128, owner_fee: U128, house_fee: U128, 
       max_bet: U128, min_bet: U128, min_balance_fraction: U128, max_odds: U128, min_odds: U128) -> bool;
   
   //retrieve contract funds
   fn retrieve_owner_funds(&mut self, amount: U128) -> Promise;
   fn retrieve_nft_funds(&mut self, amount: U128) -> Promise;
   fn retrieve_house_funds(&mut self, amount: U128) -> Promise;

   //create new partnered game
   fn create_new_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, partner_fee: U128) -> bool;

   fn alter_partner(&mut self, partner_owner: AccountId, nft_contract: AccountId, partner_fee: U128, blocked: bool) -> bool;

    
}
