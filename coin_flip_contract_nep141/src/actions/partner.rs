use crate::*;

#[near_bindgen]
impl Contract {
    pub fn retrieve_partner_balance(&mut self, game_code: String) -> Promise {
        let mut game = self.internal_get_game(&game_code).expect(ERR_003);
        assert!(
            game.partner_owner == env::predecessor_account_id(),
            "{}",
            ERR_004
        );

        let balance = game.partner_balance;
        game.partner_balance = 0;
        self.internal_update_game(&game_code, &game);
        self.safe_transfer_project(game.partner_token, balance, game_code, game.partner_owner)
    }
}

// methods to be called through token receiver
impl Contract {
    pub fn fund_game_house(&mut self, token_contract: AccountId, amount: u128, game_code: String) {
        let mut game = self.internal_get_game(&game_code).expect(ERR_003);
        assert_eq!(game.partner_token, token_contract, "{}", ERR_301);
        game.house_funds += amount;
        self.internal_update_game(&game_code, &game);
    }
}

// #[cfg(test)]
// mod tests {

//     use crate::*;
//     use crate::tests::{
//         get_context, sample_contract,
//         OWNER_ACCOUNT
//     };
//     use super::*;
//     use crate::owner_interface::OwnerInterface;
//     use near_sdk::MockedBlockchain;
//     use near_sdk::{testing_env};

//     #[test]
//     fn test_view_partner_data() {
//         const BASE_DEPOSIT: u128 = 1;
//         let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0, OWNER_ACCOUNT.to_string());
//         testing_env!(context);
//         let mut contract = sample_contract();

//         let partner_account: AccountId = "partner.testnet".to_string();
//         let partner_contract: AccountId = "partner_account.testnet".to_string();
//         let partner_fee: U128 = U128(100);

//         contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());

//         let game_struct = contract.games.get(&partner_contract).unwrap();
//         assert_eq!(game_struct.partner_owner, partner_account.clone());
//         assert_eq!(game_struct.blocked, false);
//         assert_eq!(game_struct.partner_fee, partner_fee.clone().0);
//         assert_eq!(game_struct.partner_balance, 0);
//         assert!(contract.game_balances.contains_key(&partner_contract));

//         let view_call = contract.view_partner_data(partner_contract.clone());
//         assert_eq!(&game_struct.partner_owner.to_string(), view_call.get("partner_owner").unwrap());
//         assert_eq!(&game_struct.blocked.to_string(), view_call.get("blocked").unwrap());
//         assert_eq!(&game_struct.partner_fee.to_string(), view_call.get("partner_fee").unwrap());
//         assert_eq!(&game_struct.partner_balance.to_string(), view_call.get("partner_balance").unwrap());

//     }

//     #[test]
//     fn test_retrieve_partner_balance() {
//         const BASE_DEPOSIT: u128 = 1;
//         let partner_contract: AccountId = "partner_account.testnet".to_string();
//         let partner_fee: U128 = U128(100);
//         let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
//         testing_env!(context);
//         let mut contract = sample_contract();
//         contract.create_new_partner(OWNER_ACCOUNT.to_string(), partner_contract.clone(), partner_fee.clone());
//         let mut partner_struct = contract.games.get(&partner_contract).unwrap();
//         partner_struct.partner_balance = 100;
//         contract.retrieve_partner_balance(partner_contract);
//     }

//     #[test]
//     #[should_panic(expected = "No partner found for this contract")]
//     fn test_retrieve_partner_balance_no_partner() {
//         const BASE_DEPOSIT: u128 = 1;
//         let partner_contract: AccountId = "partner_account.testnet".to_string();
//         let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
//         testing_env!(context);
//         let mut contract = sample_contract();
//         contract.retrieve_partner_balance(partner_contract);
//     }

//     #[test]
//     #[should_panic(expected = "Only owner of partnered game can call this function")]
//     fn test_retrieve_partner_balance_only_owner() {
//         const BASE_DEPOSIT: u128 = 1;
//         let partner_account: AccountId = "partner.testnet".to_string();
//         let partner_contract: AccountId = "partner_account.testnet".to_string();
//         let partner_fee: U128 = U128(100);
//         let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
//         testing_env!(context);
//         let mut contract = sample_contract();
//         contract.create_new_partner(partner_account.clone(), partner_contract.clone(), partner_fee.clone());
//         let mut partner_struct = contract.games.get(&partner_contract).unwrap();
//         partner_struct.partner_balance = 100;
//         contract.retrieve_partner_balance(partner_contract);
//     }

//     #[test]
//     #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
//     fn test_retrieve_partner_balance_one_yocto() {
//         const BASE_DEPOSIT: u128 = 2;
//         let partner_contract: AccountId = "partner_account.testnet".to_string();
//         let partner_fee: U128 = U128(100);
//         let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 1000, OWNER_ACCOUNT.to_string());
//         testing_env!(context);
//         let mut contract = sample_contract();
//         contract.create_new_partner(OWNER_ACCOUNT.to_string(), partner_contract.clone(), partner_fee.clone());
//         let mut partner_struct = contract.games.get(&partner_contract).unwrap();
//         partner_struct.partner_balance = 100;
//         contract.retrieve_partner_balance(partner_contract);
//     }

// }
