use crate::*;
use super::PartnerInterface;
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
impl PartnerInterface for SlotMachine {
    
    fn view_partner_data(&self, nft_contract: AccountId) -> HashMap<String, String> {
        let mut state = std::collections::HashMap::new();

        let game_struct = self.game_structs.get(&nft_contract).expect("no partner found for this contract");

        state.insert(String::from("partner_owner"), game_struct.partner_owner.to_string());
        state.insert(String::from("only_nft_owners_can_play"), game_struct.only_nft_owners_can_play.to_string());
        state.insert(String::from("bet_payment_adjustment"), game_struct.blocked.to_string());
        
        state.insert(String::from("partner_fee"), game_struct.partner_fee.to_string());
        state.insert(String::from("partner_balance"), game_struct.partner_balance.to_string());
        
        state
    }
    
    fn retrieve_partner_balance(&mut self, nft_contract: AccountId) -> Promise {
        let mut game_struct = self.game_structs.get(&nft_contract).expect("no partner found for this contract");
        assert!(game_struct.partner_owner == env::predecessor_account_id(), "only owner of partnered game can call this function");
        assert_one_yocto();

        let balance = game_struct.partner_balance.clone();
        game_struct.partner_balance = 0;
        Promise::new(game_struct.partner_owner.clone()).transfer(balance)
    }

}
