use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PartneredGame {
    pub partner_owner: AccountId,
    pub blocked: bool,
    pub house_funds: u128,
    pub partner_token: AccountId,
    pub partner_fee: u128, // base 10e-5
    pub partner_balance: u128,

    pub bet_payment_adjustment: u128, // base 10e-5
    pub house_fee: u128,
    pub max_bet: u128,
    pub min_bet: u128,
    pub max_odds: u8,
    pub min_odds: u8,
}
