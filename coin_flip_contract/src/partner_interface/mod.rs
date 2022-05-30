use near_sdk::{
    AccountId, Promise
};
use std::collections::HashMap;

pub mod partner_interface_impl;

//offers methods responsible for depositing assets and playing the game
pub trait PartnerInterface {

    fn view_partner_data(&self, nft_contract: AccountId) -> HashMap<String, String>;
    
    fn retrieve_partner_balance(&mut self, nft_contract: AccountId) -> Promise;

}