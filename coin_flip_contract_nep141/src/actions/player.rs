use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_credits(&self, token_type: AccountId, account_id: AccountId) -> U128 {
        U128(
            self.internal_get_account(&account_id)
                .expect(ERR_001)
                .balances
                .get(&token_type)
                .unwrap_or(0),
        )
    }

    pub fn retrieve_credits(&mut self, token_contract: AccountId, amount: U128) -> Promise {
        self.assert_panic_button();
        let account_id = env::predecessor_account_id();
        let account = self.internal_get_account(&account_id).expect(ERR_001);
        let current_balance = account.balances.get(&token_contract).unwrap_or(0);
        assert!(current_balance >= amount.0, "{}", ERR_401);
        account.balances.insert(&token_contract, &(current_balance - amount.0));
        self.internal_update_account(&account_id, &account);
        self.safe_transfer_user(token_contract, amount.0, account_id)
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

// methods to be called through token receiver
impl Contract {
    pub fn user_deposit_balance(
        &mut self,
        account_id: AccountId,
        token_contract: AccountId,
        amount: u128,
    ) {
        self.assert_panic_button();

        let initial_storage = env::storage_usage();
        let account = self.internal_get_account(&account_id).expect(ERR_001);

        let credits = account.balances.get(&token_contract).unwrap_or(0);
        account
            .balances
            .insert(&token_contract, &(credits + amount));

        self.internal_update_account(&account_id, &account);
        account.track_storage_usage(initial_storage);
        self.internal_update_account(&account_id, &account);
    }
}
