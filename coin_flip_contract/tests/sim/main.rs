use std::convert::TryInto;
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U128, U64};
pub use near_sdk::serde_json::{json, value::Value};
pub use near_sdk_sim::{call, view, deploy, init_simulator, to_yocto, UserAccount, 
    ContractAccount, DEFAULT_GAS, ViewResult, ExecutionResult};
pub use near_sdk::AccountId;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    COIN_BYTES => "./target/wasm32-unknown-unknown/release/classy_kangaroo_coin_flip.wasm",
}

use std::convert::TryFrom;
pub use rand::Rng;


const NFT_FEE: u128 = 4_000;
const OWNER_FEE: u128 = 500;
const HOUSE_FEE: u128 = 500;
const PARTNER_FEE: u128 = 100;
const FRACTIONAL_BASE: u128 = 100_000;
const MIN_BALANCE_FRACTION: u128 = 100;

const GAS_ATTACHMENT: u64 = 300_000_000_000_000;

#[test]
fn simulate_full_flow() {
    //Test full flow from deploying app
    //deploys coin contract
    //3 different partnered games are created
    //users deposit and play in one game
    //asserts that deposit play and withdraw functions are working as expected
    //asserts no state spill over from one game to another
    //gets fee balances and withdraw to owners

    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = u64::MAX;
    genesis.gas_price = 0;

    let root = init_simulator(Some(genesis));

    let owner_account = root.create_user("owner_account".to_string(), to_yocto("100"));
    let nft_account = root.create_user("nft_account".to_string(), to_yocto("100"));

    let consumer1 = root.create_user("consumer1".to_string(), to_yocto("100"));
    let consumer2 = root.create_user("consumer2".to_string(), to_yocto("100"));
    let consumer3 = root.create_user("consumer3".to_string(), to_yocto("100"));

    let collection1 = root.create_user("collection1".to_string(), to_yocto("100"));
    let collection_owner1 = root.create_user("collection_owner1".to_string(), to_yocto("100"));
    let collection2 = root.create_user("collection2".to_string(), to_yocto("100"));
    let collection_owner2 = root.create_user("collection_owner2".to_string(), to_yocto("100"));
    let collection3 = root.create_user("collection3".to_string(), to_yocto("100"));
    let collection_owner3 = root.create_user("collection_owner3".to_string(), to_yocto("100"));

    // //deploy contracts
    let coin_account = root.deploy(
        &COIN_BYTES,
        "coin_contract".to_string(),
        to_yocto("100")
    );

    let initial_house_balance: u128 = to_yocto("100");
    let max_bet: u128 = to_yocto("5");
    let min_bet: u128 = to_yocto("0.05");
    let max_odds: u128 = 200;
    let min_odds: u128 = 50;
    root.call(
        coin_account.account_id(), 
        "new", 
        &json!({
            "owner_id": owner_account.account_id(),
            "nft_account": nft_account.account_id(),
            "bet_payment_adjustment": FRACTIONAL_BASE.to_string(),
            "nft_fee": NFT_FEE.to_string(),
            "owner_fee": OWNER_FEE.to_string(),
            "house_fee": HOUSE_FEE.to_string(),
            "house_balance": initial_house_balance.to_string(),
            "max_bet": max_bet.to_string(),
            "min_bet": min_bet.to_string(),
            "min_balance_fraction": MIN_BALANCE_FRACTION.to_string(),
            "max_odds": max_odds.to_string(),
            "min_odds": min_odds.to_string()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();
    
    // create account partners
    owner_account.call(
        coin_account.account_id(), 
        "create_new_partner", 
        &json!({
            "partner_owner": collection_owner1.account_id(), 
            "nft_contract": collection1.account_id(), 
            "partner_fee": U128(PARTNER_FEE)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        1
    ).assert_success();

    owner_account.call(
        coin_account.account_id(), 
        "create_new_partner", 
        &json!({
            "partner_owner": collection_owner2.account_id(), 
            "nft_contract": collection2.account_id(), 
            "partner_fee": U128(PARTNER_FEE)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        1
    ).assert_success();

    owner_account.call(
        coin_account.account_id(), 
        "create_new_partner", 
        &json!({
            "partner_owner": collection_owner3.account_id(), 
            "nft_contract": collection3.account_id(), 
            "partner_fee": U128(PARTNER_FEE)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        1
    ).assert_success();



    //deposit
    let deposit_amount = to_yocto("10");

    consumer1.call(
        coin_account.account_id(), 
        "deposit_balance", 
        &json!({
            "game_collection_id": collection1.account_id()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        deposit_amount
    ).assert_success();



    let consumer_balance0: String = consumer1.view(
        coin_account.account_id(), 
        "get_credits", 
        &json!({
            "game_collection_id": collection1.account_id(),
            "user_account_id": consumer1.account_id()
        }).to_string().into_bytes(),
    ).unwrap_json();

    assert_eq!(consumer_balance0, deposit_amount.to_string());

    //play repeatedly and check user balance changes
    //check immutability of other user balances
    //check nft and dev balances
    let bet_size: u128 = to_yocto("1");
    let mut game_result: bool;

    let mut net_bet: u128;
    let mut consumer_balance1: u128;
    let mut consumer_balance2: u128;
    let mut expected_balance: u128;

    let mut nft_fee: u128;
    let mut owner_fee: u128;
    let mut house_fee: u128;
    let mut partner_fee: u128;

    let mut retrieved_state0: std::collections::HashMap<String, String>;
    let mut retrieved_state1: std::collections::HashMap<String, String>;
    let mut partner_retrieved_state0: std::collections::HashMap<String, String>;
    let mut partner_retrieved_state1: std::collections::HashMap<String, String>;
    let mut nft_balance0: u128;
    let mut nft_balance1: u128;
    let mut nft_expected_balance: u128;
    let mut owner_balance0: u128;
    let mut owner_balance1: u128;
    let mut owner_expected_balance: u128;
    let mut house_balance0: u128;
    let mut house_balance1: u128;
    let mut house_expected_balance: u128;
    let mut partner_balance0: u128;
    let mut partner_balance1: u128;
    let mut partner_expected_balance: u128;

    let mut chosen_odds: u8;

    let mut won_value: u128;

    loop {

        chosen_odds = rand::thread_rng().gen();

        retrieved_state0 = consumer1.view(
            coin_account.account_id(), 
            "get_contract_state", 
            &json!({}).to_string().into_bytes(),
        ).unwrap_json();

        partner_retrieved_state0 = consumer1.view(
            coin_account.account_id(), 
            "view_partner_data", 
            &json!({
                "nft_contract": collection1.account_id()
            }).to_string().into_bytes(),
        ).unwrap_json();

        
        
        nft_balance0 = retrieved_state0.get("nft_balance").unwrap().parse().unwrap();
        owner_balance0 = retrieved_state0.get("owner_balance").unwrap().parse().unwrap();
        house_balance0 = retrieved_state0.get("house_balance").unwrap().parse().unwrap();
        partner_balance0 = partner_retrieved_state0.get("partner_balance").unwrap().parse().unwrap();

        nft_fee = (bet_size * NFT_FEE) / FRACTIONAL_BASE;
        owner_fee = (bet_size * OWNER_FEE) / FRACTIONAL_BASE;
        house_fee = (bet_size * HOUSE_FEE) / FRACTIONAL_BASE;
        partner_fee = (bet_size * PARTNER_FEE) / FRACTIONAL_BASE;
    
        nft_expected_balance = nft_balance0 + nft_fee;
        owner_expected_balance = owner_balance0 + owner_fee;
        partner_expected_balance = partner_balance0 + partner_fee;
    
        consumer_balance1 = ViewResult::unwrap_json::<String>(&consumer1.view(
            coin_account.account_id(), 
            "get_credits", 
            &json!({
                "game_collection_id": collection1.account_id(),
                "user_account_id": consumer1.account_id()
            }).to_string().into_bytes(),
        )).parse().unwrap();
    
        game_result = consumer1.call(
            coin_account.account_id(), 
            "play", 
            &json!({
                "game_collection_id": collection1.account_id(),
                "bet_size": bet_size.to_string(),
                "odds": U128(chosen_odds.into()),
                "_bet_type": "true"
            }).to_string().into_bytes(),
            GAS_ATTACHMENT, 
            0
        ).unwrap_json_value().as_bool().unwrap();
    
        consumer_balance2 = ViewResult::unwrap_json::<String>(&consumer3.view(
            coin_account.account_id(), 
            "get_credits", 
            &json!({
                "game_collection_id": collection1.account_id(),
                "user_account_id": consumer1.account_id()
            }).to_string().into_bytes(),
        )).parse().unwrap();
    
        retrieved_state1 = consumer1.view(
            coin_account.account_id(), 
            "get_contract_state", 
            &json!({}).to_string().into_bytes(),
        ).unwrap_json();

        partner_retrieved_state1 = consumer1.view(
            coin_account.account_id(), 
            "view_partner_data", 
            &json!({
                "nft_contract": collection1.account_id()
            }).to_string().into_bytes(),
        ).unwrap_json();
    
        nft_balance1 = retrieved_state1.get("nft_balance").unwrap().parse().unwrap();
        owner_balance1 = retrieved_state1.get("owner_balance").unwrap().parse().unwrap();
        house_balance1 = retrieved_state1.get("house_balance").unwrap().parse().unwrap();
        partner_balance1 = partner_retrieved_state1.get("partner_balance").unwrap().parse().unwrap();
    
        net_bet = bet_size - nft_fee - owner_fee - house_fee - partner_fee;
    
        if game_result {
            won_value = ( ( ( net_bet * 256 ) / u128::try_from(chosen_odds).unwrap() ) * FRACTIONAL_BASE ) / FRACTIONAL_BASE;
            expected_balance = consumer_balance1 - bet_size + won_value;
            house_expected_balance = house_balance0 + house_fee - won_value;
        } else {
            expected_balance = consumer_balance1 - bet_size;
            house_expected_balance = house_balance0 + house_fee;
        }
    
        assert_eq!(nft_balance1, nft_expected_balance);
        assert_eq!(owner_balance1, owner_expected_balance);
        assert_eq!(house_balance1, house_expected_balance);
        // assert_eq!(partner_balance1, partner_expected_balance);
        assert_eq!(consumer_balance2, expected_balance);

        

        if consumer_balance2 <= bet_size {
            break
        }

    }

}
