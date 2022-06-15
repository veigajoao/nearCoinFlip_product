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
        let initial_storage = env::storage_usage();
        let mut account = self.internal_get_account(&account_id).expect(ERR_001);
        let current_balance = account.balances.get(&token_contract).unwrap_or(0);
        assert!(current_balance >= amount.0, "{}", ERR_401);

        let new_balance = current_balance - amount.0;
        if new_balance == 0 {
            account.balances.remove(&token_contract);
        } else {
            account.balances.insert(&token_contract, &new_balance);
        }
        self.internal_update_account_storage_check(&account_id, account, initial_storage);
        self.safe_transfer_user(token_contract, amount.0, account_id)
    }

    //plays the game, user can choose the game collection to play within, size of the bet,
    //the odds that they eant to take (the smallet the odds, the greater prize).
    //_bet_type is a dummy param for indexers to display the bet choice the user made, but are
    //irrelevant for game logic
    pub fn play(
        &mut self,
        game_code: AccountId,
        bet_size: U128,
        odds: u8,
        _bet_type: String,
    ) -> bool {
        self.assert_panic_button();

        // check that user has credits
        let account_id = env::predecessor_account_id();

        let mut account = self.internal_get_account(&account_id).expect(ERR_001);
        let mut game = self.internal_get_game(&game_code).expect(ERR_002);
        let mut credits = account.balances.get(&game.partner_token).unwrap_or(0);
        assert!(credits >= bet_size.0, "{}", ERR_402);
        assert!(
            bet_size.0 >= game.min_bet,
            "{}. Minimum is {}",
            ERR_403,
            game.min_bet
        );
        assert!(
            bet_size.0 <= game.max_bet,
            "{}. Maximum is {}",
            ERR_404,
            game.max_bet
        );

        // charge dev and nft fees
        let mut net_bet = bet_size.0;
        let nft_cut = (net_bet * self.nft_fee) / FRACTIONAL_BASE;
        let owner_cut = (net_bet * self.owner_fee) / FRACTIONAL_BASE;
        let house_cut = (net_bet * game.house_fee) / FRACTIONAL_BASE;
        let partner_cut = (net_bet * game.partner_fee) / FRACTIONAL_BASE;
        net_bet = net_bet - nft_cut - owner_cut - house_cut - partner_cut;
        let nft_balance = self.nft_balance.get(&game.partner_token).unwrap_or(0);
        self.nft_balance
            .insert(&game.partner_token, &(nft_balance + nft_cut));

        let owner_balance = self.owner_balance.get(&game.partner_token).unwrap_or(0);
        self.owner_balance
            .insert(&game.partner_token, &(owner_balance + owner_cut));
        game.house_funds += house_cut;
        game.partner_balance += partner_cut;

        // send off credits
        credits = credits - bet_size.0;
        let rand: u8 = *env::random_seed().get(0).unwrap();
        let random_hash = u128::from_be_bytes(
            env::keccak256(&[rand, (self.game_count % 256) as u8])
                .try_into()
                .unwrap(),
        );
        let rand_shuffled: u8 = (random_hash % 256) as u8;
        let outcome: bool = rand_shuffled < odds;
        if outcome {
            let won_value =
                (((net_bet * 256) / (odds as u128)) * game.bet_payment_adjustment) / FRACTIONAL_BASE;
            credits = credits + won_value;
            game.house_funds -= won_value;
        }

        account.balances.insert(&game.partner_token, &credits);
        self.internal_update_account(&account_id, &account);
        self.internal_update_game(&game_code, &game);
        self.game_count += 1;
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
        let mut account = self.internal_get_account(&account_id).expect(ERR_001);

        let credits = account.balances.get(&token_contract).unwrap_or(0);
        account
            .balances
            .insert(&token_contract, &(credits + amount));

        self.internal_update_account_storage_check(&account_id, account, initial_storage);
    }
}
