use near_sdk::{ borsh };
use borsh::{ BorshDeserialize, BorshSerialize };
use near_sdk::{
    env, near_bindgen, AccountId, Balance, PublicKey, Promise,
    collections::{ UnorderedMap },
    json_types::{ U128, Base58PublicKey },
};
use near_sdk::serde::Serialize;

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

const ONE_NEAR:u128 = 1_000_000_000_000_000_000_000_000;
const PROB:u8 = 128;
const FRACTIONAL_BASE: u128 = 10_000;

#[near_bindgen]

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SlotMachine {
    pub owner_id: AccountId,
    pub nft_id: AccountId,
    pub credits: UnorderedMap<AccountId, Balance>,
    pub nft_fee: u128, // base 10e-5
    pub dev_fee: u128, // base 10e-5
    pub house_fee: u128,
    pub win_multiplier: u128, // base 10e-5
    pub nft_balance: u128,
    pub dev_balance: u128
}

impl Default for SlotMachine {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl SlotMachine {
    #[init]
    pub fn new(owner_id: AccountId, nft_id: AccountId, nft_fee: u128, dev_fee: u128, house_fee: u128, win_multiplier: u128) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Invalid owner account");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id,
            nft_id,
            credits: UnorderedMap::new(b"credits".to_vec()),
            nft_fee, // 4000
            dev_fee, // 500
            house_fee, // 500
            win_multiplier, // 20000
            nft_balance: 0,
            dev_balance: 0
        }
    }

    #[payable]
    pub fn deposit(&mut self) {
        let account_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();
        let mut credits = self.credits.get(&account_id).unwrap_or(0);
        credits = credits + deposit;
        self.credits.insert(&account_id, &credits);
    }
    
    pub fn play(&mut self, bet_size: u128) -> bool {

        // check that user has credits
        let account_id = env::predecessor_account_id();
        let mut credits = self.credits.get(&account_id).unwrap_or(0);
        assert!(credits > 0, "no credits to play");

        // charge dev and nft fees
        let mut net_bet: u128 = bet_size;
        let nft_cut: u128 = (&net_bet * self.nft_fee) / FRACTIONAL_BASE;
        let dev_cut: u128 = (&net_bet * self.dev_fee) / FRACTIONAL_BASE;
        let house_cut: u128 = (&net_bet * self.house_fee) / FRACTIONAL_BASE;
        
        net_bet = net_bet - &nft_cut - &dev_cut - &house_cut;
        self.nft_balance = self.nft_balance + nft_cut;
        self.dev_balance = self.dev_balance + dev_cut;

        // send off credits
        credits = credits - &bet_size;
        
        let rand: u8 = *env::random_seed().get(0).unwrap();
        if rand < PROB {
            let won_value = (net_bet * self.win_multiplier) / FRACTIONAL_BASE;
            credits = credits + won_value;
        }

        self.credits.insert(&account_id, &credits);
        rand < PROB
    }

    pub fn get_credits(&self, account_id: AccountId) -> U128 {
        self.credits.get(&account_id).unwrap_or(0).into()
    }

    pub fn retrieve_credits(&mut self) -> Promise {
        let account_id = env::predecessor_account_id();
        let credits: u128 = self.credits.get(&account_id).unwrap_or(0).into();
        let new_credits: u128 = 0;
        self.credits.insert(&account_id, &new_credits);
        Promise::new(env::predecessor_account_id()).transfer(credits)
    }

    //retrieve dev funds function
    pub fn retrieve_dev_funds(&mut self) {
        //retrive value storaged for devs/nft holders
        //zero storaged values
        //send funds

        // function nft_total_supply(): string {}

        // function nft_tokens(
        // from_index: string|null, // default: "0"
        // limit: number|null, // default: unlimited (could fail due to gas limit)
        // ): Token[] {}

    }

    //update contract initialization vars
    pub fn update_contract(&mut self, nft_fee: u128, dev_fee: u128, house_fee: u128, win_multiplier: u128) {
        assert!(env::predecessor_account_id() == self.owner_id, "Only owner can call this function");
        self.nft_fee = nft_fee;
        self.dev_fee = dev_fee;
        self.house_fee = house_fee;
        self.win_multiplier = win_multiplier;
    }
}


// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    const CONTRACT_ACCOUNT: &str = "contract.testnet";
    const SIGNER_ACCOUNT: &str = "signer.testnet";
    const OWNER_ACCOUNT: &str = "owner.testnet";
    const NFT_ACCOUNT: &str = "nft.testnet";

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
    fn get_context(input: Vec<u8>, is_view: bool, attached_deposit: u128, account_balance: u128) -> VMContext {
        VMContext {
            current_account_id: CONTRACT_ACCOUNT.to_string(),
            signer_account_id:  SIGNER_ACCOUNT.to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id:  SIGNER_ACCOUNT.to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn test_deposit_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 10_000_000;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), 0);
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            nft_id: NFT_ACCOUNT.to_string(),
            credits: UnorderedMap::new(b"credits".to_vec()),
            nft_fee: 400, // base 10e-5
            dev_fee: 10, // base 10e-5
            house_fee: 10,
            win_multiplier: 200000u128, // base 10e-5
            nft_balance: 0,
            dev_balance: 0
        };
        let user_balance1: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value before deposit: {}", &user_balance1);
        contract.deposit();
        let user_balance2: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value after deposit: {}", &user_balance2);
        // confirm that we received 1 when calling get_num
        assert_eq!(BASE_DEPOSIT, user_balance2);
    }

    #[test]
    fn test_withdrawal_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 48_000;
        const CONTRACT_BALANCE: u128 = 1_000_000_000_000_000;
        const WITHDRAWAL_AMOUNT: u128 = 48_000;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            nft_id: NFT_ACCOUNT.to_string(),
            credits: UnorderedMap::new(b"credits".to_vec()),
            nft_fee: 400, // base 10e-5
            dev_fee: 10, // base 10e-5
            house_fee: 10,
            win_multiplier: 200000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0
        };
    
        contract.credits.insert(&SIGNER_ACCOUNT.to_string(), &WITHDRAWAL_AMOUNT);
        let user_balance1: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value before withdrawal: {}", &user_balance1);
        contract.retrieve_credits();
        let user_balance2: u128 = contract.credits.get(&"signer.testnet".to_string()).unwrap_or(0);
        println!("Value after withdrawal: {}", &user_balance2);
        // confirm that we received 1 when calling get_num
        assert_eq!(WITHDRAWAL_AMOUNT, user_balance1);
        assert_eq!(0, user_balance2);
    }

    #[test]
    fn test_get_credits_function() {
        // set up the mock context into the testing environment
        const BASE_DEPOSIT: u128 = 0;
        const CONTRACT_BALANCE: u128 = 0;
        let context = get_context(vec![], false, BASE_DEPOSIT.clone(), CONTRACT_BALANCE.clone());
        testing_env!(context);
        // instantiate a contract variable with the counter at zero
        let mut contract =  SlotMachine {
            owner_id: OWNER_ACCOUNT.to_string(),
            nft_id: NFT_ACCOUNT.to_string(),
            credits: UnorderedMap::new(b"credits".to_vec()),
            nft_fee: 400, // base 10e-5
            dev_fee: 10, // base 10e-5
            house_fee: 10,
            win_multiplier: 200000, // base 10e-5
            nft_balance: 0,
            dev_balance: 0
        };
        
        const BALANCE_AMOUNT: u128 = 48_000;
        contract.credits.insert(&SIGNER_ACCOUNT.to_string(), &BALANCE_AMOUNT);
        let user_balance: u128 =  contract.get_credits(SIGNER_ACCOUNT.clone().to_string()).into();

        assert_eq!(BALANCE_AMOUNT, user_balance);
    }

    //missing:
    // play
    // update contract
    // dev funds

}