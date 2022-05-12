use crate::*;
use crate::TokenType::FT;

#[near_bindgen]
impl SlotMachine {

    /*
    msg should follow json format:
    method: string,
    args: {}
    */

    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> String {

        let initial_storage = env::storage_usage();
        
        let parsed_message: serde_json::Value = serde_json::from_str(&msg).expect("Unable to deserialize msg");
        let method = parsed_message["method"].as_str().expect("Couldn't deserealize method in msg");

        if method == "deposit_sub_house_funds" {
            let target_game = parsed_message["target_game"].as_str().expect("Couldn't deserialize target_game in msg");
            let token = env::predecessor_account_id();

            let mut partnered_game = self.game_structs.get(&target_game.to_string()).expect("target_game does not exist").clone();

            if let FT(i) = partnered_game.clone().token {
                assert!(i == token, "Token deposited does not match PartneredGame's token");
            } else {
                panic!("Not implemented");
            }
            
            partnered_game.deposit_sub_house_funds(amount.0);
            self.game_structs.insert(&target_game.to_string(), &partnered_game);

        } else if method == "deposit_user_funds" {

            let target_game = parsed_message["target_game"].as_str().expect("Couldn't deserialize target_game in msg");
            let token = env::predecessor_account_id();

            let partnered_game = self.game_structs.get(&target_game.to_string()).expect("target_game does not exist");

            if let FT(i) = partnered_game.token {
                assert!(i == token, "Token deposited does not match PartneredGame's token");
            } else {
                panic!("Not implemented");
            }

            self.deposit_balance(sender_id.clone(), amount.0, target_game.to_string());

        }

        let final_storage = env::storage_usage();
        self.charge_storage_cost(initial_storage, final_storage, sender_id.clone());

        "0".to_string()

    }

}
