use near_sdk::{
    AccountId, Promise,
    json_types::{ U128, },
};

pub mod game_interface_impl;

//offers methods responsible for depositing assets and playing the game
pub trait GameInterface {
    
    //deposits assets into account for one specific game partner
    fn deposit_balance(&mut self, game_collection_id: AccountId) -> U128;

    //retrieves the balance for one specific user in a specific partnered game
    fn get_credits(&self, game_collection_id: AccountId, user_account_id: AccountId) -> U128;

    //retrieves the balance of the sender in the specified game
    fn retrieve_credits(&mut self, game_collection_id: AccountId) -> Promise;

    //plays the game, user can choose the game collection to play within, size of the bet,
    //the odds that they eant to take (the smallet the odds, the greater prize).
    //_bet_type is a dummy param for indexers to display the bet choice the user made, but are
    //irrelevant for game logic
    fn play(&mut self, game_collection_id: AccountId, bet_size: U128, odds: U128, _bet_type: String) -> bool;
}