use near_sdk::{
    AccountId, Promise,
    json_types::{ U128, },
};

pub mod partner_interface_impl;

//offers methods responsible for depositing assets and playing the game
pub trait PartnerInterface {

    fn view_partner_data(&self, nft_contract: AccountId) -> HashMap<String, String>;
    
    fn retrieve_partner_balance(&mut self, partner_contract: AccountId) -> Promise;

    fn alter_partner_fee(&mut self, new_partner_fee: U128) -> bool;

    
}