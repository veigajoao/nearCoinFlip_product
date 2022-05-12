use crate::*;
use crate::TokenType::FT;
use super::GameInterface;
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
    

    fn get_credits(&self, game_collection_id: AccountId, user_account_id: AccountId) -> U128 {
        U128(self.game_balances.get(&game_collection_id).unwrap().get(&user_account_id).unwrap_or(0))
    }

    #[payable]
    fn retrieve_credits(&mut self, game_collection_id: AccountId) -> Promise {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");
        assert_one_yocto();
        let initial_storage = env::storage_usage();

        let account_id = env::predecessor_account_id();

        let credits = self.game_balances.get(&game_collection_id).expect("Partnered game does not exist").remove(&account_id).unwrap_or(0);
        
        let game_struct = self.game_structs.get(&game_collection_id).expect("No partner found for this contract");
        
        let final_storage = env::storage_usage();
        self.charge_storage_cost(initial_storage, final_storage, account_id.clone());
        
        match game_struct.token {
            FT(i) => (
                Promise::new(i)
                    .function_call(
                        b"ft_transfer".to_vec(),
                        json!({
                            "receiver_id": account_id,
                            "amount": credits.to_string(),
                            "memo": "coin flip user withdraw"
                        }).to_string().as_bytes().to_vec(),
                        1,
                        BASE_GAS,
                    )
            )
        }
        
    }

    //plays the game, user can choose the game collection to play within, size of the bet,
    //the odds that they eant to take (the smallet the odds, the greater prize).
    //_bet_type is a dummy param for indexers to display the bet choice the user made, but are
    //irrelevant for game logic
    fn play(&mut self, game_collection_id: AccountId, bet_size: U128, odds: U128, _bet_type: String) -> bool {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");

        // check that user has credits
        let account_id = env::predecessor_account_id();
        
        let mut game_struct = self.game_structs.get(&game_collection_id).expect("provided game_collection_id does not exist").clone();        
        let balance_address: String;
        match game_struct.token {
            FT(ref i) => (
                balance_address = i.clone()
            ),
            _ => panic!("Unimplemented")
        }
        let mut owner_balance = self.owner_balance.get(&balance_address).unwrap_or(0);
        let mut nft_balance = self.nft_balance.get(&balance_address).unwrap_or(0);

        let mut credits = self.game_balances.get(&game_collection_id).unwrap().get(&account_id).unwrap_or(0);
        assert!(credits > bet_size.0, "no credits to play");
        assert!(bet_size.0 >= self.min_bet, "minimum bet_size is {} yoctonear", self.min_bet);
        assert!(bet_size.0 <= self.max_bet, "maximum bet_size is {} yoctonear", self.max_bet);

        // charge dev and nft fees
        let mut net_bet: u128 = bet_size.0;
        let nft_cut: u128 = (&net_bet * self.nft_fee) / FRACTIONAL_BASE;
        let owner_cut: u128 = (&net_bet * self.owner_fee) / FRACTIONAL_BASE;
        let house_cut: u128 = (&net_bet * self.house_fee) / FRACTIONAL_BASE;
        let partner_cut: u128 = (&net_bet * &game_struct.partner_fee) / FRACTIONAL_BASE;
        
        net_bet = net_bet - &nft_cut - &owner_cut - &house_cut - &partner_cut;
        nft_balance = nft_balance + nft_cut;
        owner_balance = owner_balance + owner_cut;
        game_struct.sub_house_balance = game_struct.sub_house_balance + house_cut;
        game_struct.partner_balance = game_struct.partner_balance + partner_cut;

        let won_value = ( ( ( net_bet * 256 ) / (odds.0) ) * self.bet_payment_adjustment ) / FRACTIONAL_BASE;
        assert!(won_value >= game_struct.sub_house_balance, "insufficient sub_house_balance to cover bet");

        // send off credits
        credits = credits - bet_size.0;
        
        let rand: u8 = *env::random_seed().get(0).unwrap();
        let u8_odds = u8::try_from(odds.0).unwrap();
        let outcome: bool = rand < u8_odds;
        if outcome {
            credits = credits + won_value;
            game_struct.sub_house_balance = game_struct.sub_house_balance - won_value;
        }

        self.nft_balance.insert(&balance_address, &nft_balance);
        self.owner_balance.insert(&balance_address, &owner_balance);
        self.game_structs.insert(&game_collection_id, &game_struct);
        self.game_balances.get(&game_collection_id).unwrap().insert(&account_id, &credits);
        outcome
    }
    
}

impl SlotMachine {

    pub fn deposit_balance(&mut self, account_id: AccountId, deposit: u128, game_collection_id: AccountId) -> U128 {
        assert!(!self.panic_button, "Panic mode is on, contract has been paused by owner");

        assert!(deposit > (self.min_bet / self.min_balance_fraction), "Minimum accepted deposit is {}", (self.min_bet / self.min_balance_fraction) );

        let mut credits = self.game_balances.get(&game_collection_id).unwrap().get(&account_id).unwrap_or(0);
        credits = credits + deposit;
        self.game_balances.get(&game_collection_id).unwrap().insert(&account_id, &credits);
        credits.into()
    }

}



#[cfg(test)]
mod tests {

    use crate::*;
    use crate::tests::{
        get_context, sample_contract,
        SIGNER_ACCOUNT, OWNER_ACCOUNT
    };
    use crate::owner_interface::OwnerInterface;
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env};

    #[test]
    fn test_deposit_balance() {
        const BASE_DEPOSIT: u128 = 10_000;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(vec![], false, ONE_YOCTO_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());

        //deposit to that game
        context = get_context(vec![], false, BASE_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
        let game_balance = contract.game_balances.get(&partner_contract.clone()).unwrap().get(&OWNER_ACCOUNT.to_string()).unwrap();
        assert_eq!(BASE_DEPOSIT, game_balance);
    }

    #[test]
    #[should_panic(expected = "Minimum accepted deposit is 1000")]
    fn test_deposit_balance_minimum_deposit() {
        const BASE_DEPOSIT: u128 = 10;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(vec![], false, ONE_YOCTO_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());

        //deposit to that game
        context = get_context(vec![], false, BASE_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
    }

    #[test]
    #[should_panic(expected = "Panic mode is on, contract has been paused by owner")]
    fn test_deposit_balance_panic_mode() {
        const BASE_DEPOSIT: u128 = 10;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(vec![], false, ONE_YOCTO_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());
        contract.emergency_panic();

        //deposit to that game
        context = get_context(vec![], false, BASE_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
    }

    #[test]
    fn test_get_credits() {
        const BASE_DEPOSIT: u128 = 10_000;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(vec![], false, ONE_YOCTO_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());

        //deposit to that game
        context = get_context(vec![], false, BASE_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
        let game_balance = contract.game_balances.get(&partner_contract.clone()).unwrap().get(&OWNER_ACCOUNT.to_string()).unwrap();
        let get_credits_balance = contract.get_credits(partner_contract.clone(), OWNER_ACCOUNT.to_string()).0;
        let zero_get_credits = contract.get_credits(partner_contract.clone(), "420".to_string()).0;

        assert_eq!(BASE_DEPOSIT, game_balance);
        assert_eq!(BASE_DEPOSIT, get_credits_balance);
        assert_eq!(0, zero_get_credits);
    }

    #[test]
    fn test_retrieve_credits() {
        const BASE_DEPOSIT: u128 = 10_000;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(vec![], false, ONE_YOCTO_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());

        //deposit to that game
        context = get_context(vec![], false, BASE_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
        let game_balance = contract.game_balances.get(&partner_contract.clone()).unwrap().get(&OWNER_ACCOUNT.to_string()).unwrap();
        assert_eq!(BASE_DEPOSIT, game_balance);

        contract.retrieve_credits(partner_contract.clone());

        let game_balance2 = contract.game_balances.get(&partner_contract.clone()).unwrap().get(&OWNER_ACCOUNT.to_string()).unwrap_or(0);
        assert_eq!(0, game_balance2);
    }

    #[test]
    #[should_panic(expected = "Panic mode is on, contract has been paused by owner")]
    fn test_retrieve_credits_panic_mode() {
        const BASE_DEPOSIT: u128 = 10;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(vec![], false, ONE_YOCTO_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());
        contract.emergency_panic();
        contract.retrieve_credits(partner_contract.clone());
    }

    #[test]
    #[should_panic(expected = "Partnered game does not exist")]
    fn test_retrieve_credits_non_existing_partner() {
        const BASE_DEPOSIT: u128 = 10;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(vec![], false, ONE_YOCTO_DEPOSIT.clone(), 10_000, OWNER_ACCOUNT.to_string());
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);

        //call retrieve_credits on non existing partner
        contract.retrieve_credits(partner_contract.clone());
    }

}