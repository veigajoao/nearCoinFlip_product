use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    fn deposit_balance(&mut self, game_collection_id: AccountId) -> U128 {
        self.assert_panic_button();
        let account_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();

        assert!(
            deposit > (self.min_bet / self.min_balance_fraction),
            "Minimum accepted deposit is {}",
            (self.min_bet / self.min_balance_fraction)
        );

        let mut credits = self
            .game_balances
            .get(&game_collection_id)
            .unwrap()
            .get(&account_id)
            .unwrap_or(0);
        credits = credits + deposit;
        self.game_balances
            .get(&game_collection_id)
            .unwrap()
            .insert(&account_id, &credits);
        credits.into()
    }

    fn get_credits(&self, game_collection_id: AccountId, user_account_id: AccountId) -> U128 {
        U128(
            self.game_balances
                .get(&game_collection_id)
                .unwrap()
                .get(&user_account_id)
                .unwrap_or(0),
        )
    }

    fn retrieve_credits(&mut self, game_collection_id: AccountId) -> Promise {
        self.assert_panic_button();
        let account_id = env::predecessor_account_id();

        let credits = self
            .game_balances
            .get(&game_collection_id)
            .expect("Partnered game does not exist")
            .remove(&account_id)
            .unwrap_or(0);
        Promise::new(env::predecessor_account_id()).transfer(credits)
    }

    //plays the game, user can choose the game collection to play within, size of the bet,
    //the odds that they eant to take (the smallet the odds, the greater prize).
    //_bet_type is a dummy param for indexers to display the bet choice the user made, but are
    //irrelevant for game logic
    fn play(
        &mut self,
        game_collection_id: AccountId,
        bet_size: U128,
        odds: U128,
        _bet_type: String,
    ) -> bool {
        self.assert_panic_button();

        // check that user has credits
        let account_id = env::predecessor_account_id();

        let mut game_struct = self
            .game_structs
            .get(&game_collection_id)
            .expect("provided game_collection_id does not exist");
        let mut credits = self
            .game_balances
            .get(&game_collection_id)
            .unwrap()
            .get(&account_id)
            .unwrap_or(0);
        assert!(credits >= bet_size.0, "no credits to play");
        assert!(
            bet_size.0 >= self.min_bet,
            "minimum bet_size is {} yoctonear",
            self.min_bet
        );
        assert!(
            bet_size.0 <= self.max_bet,
            "maximum bet_size is {} yoctonear",
            self.max_bet
        );

        // charge dev and nft fees
        let mut net_bet: u128 = bet_size.0;
        let nft_cut: u128 = (&net_bet * self.nft_fee) / FRACTIONAL_BASE;
        let owner_cut: u128 = (&net_bet * self.owner_fee) / FRACTIONAL_BASE;
        let house_cut: u128 = (&net_bet * self.house_fee) / FRACTIONAL_BASE;
        let partner_cut: u128 = (&net_bet * &game_struct.partner_fee) / FRACTIONAL_BASE;
        net_bet = net_bet - &nft_cut - &owner_cut - &house_cut - &partner_cut;
        self.nft_balance = self.nft_balance + nft_cut;
        self.owner_balance = self.owner_balance + owner_cut;
        self.house_balance = self.house_balance + house_cut;
        game_struct.partner_balance = game_struct.partner_balance + partner_cut;
        self.game_structs.insert(&game_collection_id, &game_struct);

        // send off credits
        credits = credits - bet_size.0;
        let rand: u8 = *env::random_seed().get(0).unwrap();
        let u8_odds = u8::try_from(odds.0).unwrap();
        let random_hash = env::keccak256(&[rand, (self.game_count % 256) as u8]);
        let mut byte_sum: u128 = 0;
        random_hash.iter().for_each(|v| byte_sum += *v as u128);
        let rand_shuffled: u8 = (byte_sum % 256) as u8;
        let outcome: bool = rand_shuffled < u8_odds;
        if outcome {
            let won_value =
                (((net_bet * 256) / (odds.0)) * self.bet_payment_adjustment) / FRACTIONAL_BASE;
            credits = credits + won_value;
            self.house_balance = self.house_balance - won_value;
        }

        self.game_count += 1;
        self.game_balances
            .get(&game_collection_id)
            .unwrap()
            .insert(&account_id, &credits);
        outcome
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::owner_interface::OwnerInterface;
    use crate::tests::{get_context, sample_contract, OWNER_ACCOUNT};
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;

    #[test]
    fn test_deposit_balance() {
        const BASE_DEPOSIT: u128 = 10_000;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(
            vec![],
            false,
            ONE_YOCTO_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(
            partner_account.clone(),
            partner_contract.clone(),
            partner_fee.clone(),
        );

        //deposit to that game
        context = get_context(
            vec![],
            false,
            BASE_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
        let game_balance = contract
            .game_balances
            .get(&partner_contract.clone())
            .unwrap()
            .get(&OWNER_ACCOUNT.to_string())
            .unwrap();
        assert_eq!(BASE_DEPOSIT, game_balance);
    }

    #[test]
    #[should_panic(expected = "Minimum accepted deposit is 1000")]
    fn test_deposit_balance_minimum_deposit() {
        const BASE_DEPOSIT: u128 = 10;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(
            vec![],
            false,
            ONE_YOCTO_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(
            partner_account.clone(),
            partner_contract.clone(),
            partner_fee.clone(),
        );

        //deposit to that game
        context = get_context(
            vec![],
            false,
            BASE_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
    }

    #[test]
    #[should_panic(expected = "Panic mode is on, contract has been paused by owner")]
    fn test_deposit_balance_panic_mode() {
        const BASE_DEPOSIT: u128 = 10;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(
            vec![],
            false,
            ONE_YOCTO_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(
            partner_account.clone(),
            partner_contract.clone(),
            partner_fee.clone(),
        );
        contract.emergency_panic();

        //deposit to that game
        context = get_context(
            vec![],
            false,
            BASE_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
    }

    #[test]
    fn test_get_credits() {
        const BASE_DEPOSIT: u128 = 10_000;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(
            vec![],
            false,
            ONE_YOCTO_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(
            partner_account.clone(),
            partner_contract.clone(),
            partner_fee.clone(),
        );

        //deposit to that game
        context = get_context(
            vec![],
            false,
            BASE_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
        let game_balance = contract
            .game_balances
            .get(&partner_contract.clone())
            .unwrap()
            .get(&OWNER_ACCOUNT.to_string())
            .unwrap();
        let get_credits_balance = contract
            .get_credits(partner_contract.clone(), OWNER_ACCOUNT.to_string())
            .0;
        let zero_get_credits = contract
            .get_credits(partner_contract.clone(), "420".to_string())
            .0;

        assert_eq!(BASE_DEPOSIT, game_balance);
        assert_eq!(BASE_DEPOSIT, get_credits_balance);
        assert_eq!(0, zero_get_credits);
    }

    #[test]
    fn test_retrieve_credits() {
        const BASE_DEPOSIT: u128 = 10_000;
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let mut context = get_context(
            vec![],
            false,
            ONE_YOCTO_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(
            partner_account.clone(),
            partner_contract.clone(),
            partner_fee.clone(),
        );

        //deposit to that game
        context = get_context(
            vec![],
            false,
            BASE_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        contract.deposit_balance(partner_contract.clone());
        let game_balance = contract
            .game_balances
            .get(&partner_contract.clone())
            .unwrap()
            .get(&OWNER_ACCOUNT.to_string())
            .unwrap();
        assert_eq!(BASE_DEPOSIT, game_balance);

        contract.retrieve_credits(partner_contract.clone());

        let game_balance2 = contract
            .game_balances
            .get(&partner_contract.clone())
            .unwrap()
            .get(&OWNER_ACCOUNT.to_string())
            .unwrap_or(0);
        assert_eq!(0, game_balance2);
    }

    #[test]
    #[should_panic(expected = "Panic mode is on, contract has been paused by owner")]
    fn test_retrieve_credits_panic_mode() {
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let context = get_context(
            vec![],
            false,
            ONE_YOCTO_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_account: AccountId = "partner.testnet".to_string();
        let partner_contract: AccountId = "partner_account.testnet".to_string();
        let partner_fee: U128 = U128(100);
        contract.create_new_partner(
            partner_account.clone(),
            partner_contract.clone(),
            partner_fee.clone(),
        );
        contract.emergency_panic();
        contract.retrieve_credits(partner_contract.clone());
    }

    #[test]
    #[should_panic(expected = "Partnered game does not exist")]
    fn test_retrieve_credits_non_existing_partner() {
        const ONE_YOCTO_DEPOSIT: u128 = 1;
        let context = get_context(
            vec![],
            false,
            ONE_YOCTO_DEPOSIT.clone(),
            10_000,
            OWNER_ACCOUNT.to_string(),
        );
        testing_env!(context);
        let mut contract = sample_contract();
        //create a game
        let partner_contract: AccountId = "partner_account.testnet".to_string();

        //call retrieve_credits on non existing partner
        contract.retrieve_credits(partner_contract.clone());
    }
}
