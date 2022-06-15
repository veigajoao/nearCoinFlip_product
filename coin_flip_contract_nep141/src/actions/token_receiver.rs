use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", tag = "type")]
pub enum CallType {
    FundGame { game_code: String },
    DepositBalance
}

#[near_bindgen]
impl Contract {
    #[allow(unreachable_patterns)]
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        match serde_json::from_str::<CallType>(&msg).expect(ERR_005) {
            CallType::FundGame { game_code } => {
                self.fund_game_house(env::predecessor_account_id(), amount.0, game_code);
                U128(0)
            },
            CallType::DepositBalance => {
                self.user_deposit_balance(sender_id, env::predecessor_account_id(), amount.0);
                U128(0)
            },
            _ => unimplemented!(),
        }
    }
}

