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
impl GameInterface for SlotMachine {
    
    #[payable]
    fn deposit_balance(&mut self, game_collection_id: AccountId) -> U128 {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");
        let account_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();

        assert!(deposit > (self.min_bet / self.min_balance_fraction), "Minimum accepted deposit is {}", (self.min_bet / self.min_balance_fraction) );

        let mut game_struct = self.game_structs.get(&game_collection_id).expect("provided game_collection_id does not exist");
        let mut credits = self.game_balances.get(&game_collection_id).unwrap().get(&account_id).unwrap_or(0);
        credits = credits + deposit;
        game_struct.user_balance_lookup.insert(&account_id, &credits);
        credits.into()
    }

    //retrieves the balance for one specific user in a specific partnered game
    fn get_credits(&mut self, game_collection_id: AccountId, user_account_id: AccountId) -> U128 {
        self.game_balances.get(&game_collection_id).unwrap().get(&user_account_id).unwrap_or(0);
    }

    //retrieves the balance of the sender in the specified game
    fn retrieve_credits(&mut self, game_collection_id: AccountId) -> Promise {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");
        let account_id = env::predecessor_account_id();

        let mut game_struct = self.game_structs.get(&game_collection_id).expect("provided game_collection_id does not exist");        

        let mut credits = self.game_balances.get(&game_collection_id).unwrap().get(&account_id).unwrap_or(0);
        self.credits.remove(&account_id);
        Promise::new( env::predecessor_account_id() ).transfer(credits)
    }

    //plays the game, user can choose the game collection to play within, size of the bet,
    //the odds that they eant to take (the smallet the odds, the greater prize).
    //_bet_type is a dummy param for indexers to display the bet choice the user made, but are
    //irrelevant for game logic
    fn play(&mut self, game_collection_id: AccountId, bet_size: U128, odds: U128, _bet_type: String) -> bool {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");

        // check that user has credits
        let account_id = env::predecessor_account_id();
        
        let mut game_struct = self.game_structs.get(&game_collection_id).expect("provided game_collection_id does not exist");        
        let mut credits = self.game_balances.get(&game_collection_id).unwrap().get(&account_id).unwrap_or(0);
        assert!(credits > bet_size.0, "no credits to play");
        assert!(bet_size.0 >= self.min_bet, "minimum bet_size is {} yoctonear", self.min_bet);
        assert!(bet_size.0 <= self.max_bet, "maximum bet_size is {} yoctonear", self.max_bet);

        // charge dev and nft fees
        let mut net_bet: u128 = bet_size.0;
        let nft_cut: u128 = (&net_bet * self.nft_fee) / FRACTIONAL_BASE;
        let owner_cut: u128 = (&net_bet * self.owner_fee) / FRACTIONAL_BASE;
        let house_cut: u128 = (&net_bet * self.house_fee) / FRACTIONAL_BASE;
        let partner_cut: u128 = (&net_bet * &game_struct.affiliate_fee) / FRACTIONAL_BASE;
        
        net_bet = net_bet - &nft_cut - &owner_cut - &house_cut - &partner_cut;
        self.nft_balance = self.nft_balance + nft_cut;
        self.owner_balance = self.owner_balance + owner_cut;
        self.house_balance = self.house_balance + house_cut;
        game_struct.affiliate_balance = game_struct.affiliate_balance + partner_cut;

        // send off credits
        credits = credits - bet_size.0;
        
        let rand: u8 = *env::random_seed().get(0).unwrap();
        let u8_odds = u8::try_from(odds.0).unwrap();
        let outcome: bool = rand < u8_odds;
        if outcome {
            // add odds logic here
            let won_value = ( ( (net_bet * 256 ) / odds.0 ) * self.bet_payment_adjustment ) / FRACTIONAL_BASE;
            credits = credits + won_value;
            self.house_balance = self.house_balance - won_value;
        }

        self.game_balances.get(&game_collection_id).unwrap().insert(&account_id, &credits);
        outcome
    }
    
}
