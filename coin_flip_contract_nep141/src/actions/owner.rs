use crate::*;

#[near_bindgen]
impl  Contract {
    #[payable]
    pub fn emergency_panic(&mut self) -> bool {
        self.only_owner();
        self.panic_button = !self.panic_button;
        self.panic_button
    }

    #[payable]
    pub fn update_contract(
        &mut self,
        nft_fee: U128,
        owner_fee: U128,
    ) {
        self.only_owner();

        self.nft_fee = nft_fee.0;
        self.owner_fee = owner_fee.0;
    }

    pub fn retrieve_owner_funds(&mut self) -> PromiseOrValue<bool> {
        assert!(!self.owner_balance.is_empty(), "{}", ERR_201);
        let mut keys = self.owner_balance.keys();
        let key_1 = keys.next().unwrap();
        drop(keys);
        let value_1 = self.owner_balance.insert(&key_1, &0);
        match value_1 {
            Some(v) => PromiseOrValue::Promise(self.safe_transfer_owner(key_1, v)),
            None => {
                PromiseOrValue::Value(false)
            }
        }
    }

    pub fn retrieve_nft_funds(&mut self) -> PromiseOrValue<bool> {
        assert!(!self.nft_balance.is_empty(), "{}", ERR_201);
        let mut keys = self.owner_balance.keys();
        let key_1 = keys.next().unwrap();
        drop(keys);
        let value_1 = self.nft_balance.insert(&key_1, &0);
        match value_1 {
            Some(v) => PromiseOrValue::Promise(self.safe_transfer_nft(key_1, v)),
            None => {
                PromiseOrValue::Value(false)
            }
        }
    }

    //create new partnered game
    #[payable]
    pub fn create_new_partner(
        &mut self,
        partner_owner: AccountId,
        nft_contract: AccountId,
        token_contract: AccountId,
        partner_fee: U128,
        bet_payment_adjustment: U128,
        house_fee: U128,
        max_bet: U128,
        min_bet: U128,
        max_odds: u8,
        min_odds: u8,
    ) {
        self.only_owner();
        assert!(
            !self.games.contains_key(&nft_contract),
            "{}", ERR_003
        );
        let contract_id = env::current_account_id();
        let mut contract_account = self.internal_get_account(&contract_id).unwrap();
        let initial_storage = env::storage_usage();

        self.nft_balance.insert(&token_contract, &0);
        self.owner_balance.insert(&token_contract, &0);

        let game_settings = PartneredGame {
            partner_owner,
            blocked: false,
            house_funds: 0,
            partner_token: token_contract,
            partner_fee: partner_fee.0,
            partner_balance: 0,

            bet_payment_adjustment: bet_payment_adjustment.0,
            house_fee: house_fee.0,
            max_bet: max_bet.0,
            min_bet: min_bet.0,
            max_odds,
            min_odds,
        };
        self.games.insert(&nft_contract, &game_settings);

        contract_account.track_storage_usage(initial_storage);
        self.internal_update_account(&contract_id, &contract_account);
    }

    #[payable]
    pub fn alter_partner(
        &mut self,
        game_id: String,
        partner_owner: AccountId,
        partner_fee: U128,
        blocked: bool,
    ) {
        self.only_owner();
        assert!(
            self.games.contains_key(&game_id),
            "{}", ERR_002
        );
        let mut game = self.internal_get_game(&game_id).expect(ERR_002);
        game.partner_owner = partner_owner;
        game.partner_fee = partner_fee.0;
        game.blocked = blocked;
        self.internal_update_game(&game_id, &game);

    }
}