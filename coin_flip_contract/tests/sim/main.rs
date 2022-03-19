use std::convert::TryInto;
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U128, U64};
pub use near_sdk::serde_json::json;
pub use near_sdk_sim::{call, view, deploy, init_simulator, to_yocto, UserAccount, ContractAccount, DEFAULT_GAS, ViewResult};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    COIN_BYTES => "./target/wasm32-unknown-unknown/release/classy_kangaroo_coin_flip.wasm",
    NFT_BYTES => "../nep_171/target/wasm32-unknown-unknown/release/non_fungible_token.wasm",
}

//to build standard deployment function

//test to check immutability of state other than player playing
//revert order nft and dev (one test in each order)
const NFT_FEE: u128 = 4_000;
const DEV_FEE: u128 = 500;
const HOUSE_FEE: u128 = 500;
const WIN_MULTIPLIER: u128 = 200_000;
const FRACTIONAL_BASE: u128 = 100_000;

#[test]
fn simulate_full_flow_1() {
    //Test full flow from deploying app
    //users mint nfts
    //user deposits balance into the game
    //user plays the game multiple times
    //user retrieves his balance
    //contract owner retrieves dev balance
    //contract owner retrieves nft balance
    //this test is similar to simulate_full_flow_1, just changing order
    //of dev and nft balance retrievals because of runtime error observed in past versions

    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = u64::MAX;
    genesis.gas_price = 0;

    let root = init_simulator(Some(genesis));

    let dev_account = root.create_user("dev_account".to_string(), to_yocto("100"));
    
    let consumer1 = root.create_user("consumer1".to_string(), to_yocto("100"));
    let consumer2 = root.create_user("consumer2".to_string(), to_yocto("100"));
    let consumer3 = root.create_user("consumer3".to_string(), to_yocto("100"));

    // //deploy contracts
    let nft_account = root.deploy(
        &NFT_BYTES,
        "nft_contract".to_string(),
        to_yocto("100")
    );

    let coin_account = root.deploy(
        &COIN_BYTES,
        "coin_contract".to_string(),
        to_yocto("100")
    );

    root.call(
        nft_account.account_id(), 
        "new_default_meta", 
        &json!({
            "owner_id": dev_account.account_id()
        }).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    root.call(
        coin_account.account_id(), 
        "new", 
        &json!({"owner_id": dev_account.account_id(),
                "nft_id": nft_account.account_id(),
                "nft_fee": NFT_FEE.to_string(),
                "dev_fee": DEV_FEE.to_string(),
                "house_fee": HOUSE_FEE.to_string(),
                "win_multiplier": WIN_MULTIPLIER.to_string(),
                "base_gas": DEFAULT_GAS.to_string()
        }).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    //user 1 gets 2 NFTs, user 2 gets 1, user 3 gets none
    let nft_mint_token = | account: String, token_id: String | {
        dev_account.call(
            nft_account.account_id(), 
            "nft_mint", 
            &json!({
                "token_id": token_id,
                "receiver_id": account,
                "token_metadata": {}
            }).to_string().into_bytes(),
            DEFAULT_GAS, 
            5870000000000000000000
        ).assert_success();
    };

    nft_mint_token(consumer1.account_id(), "1".to_string());
    nft_mint_token(consumer1.account_id(), "2".to_string());
    nft_mint_token(consumer2.account_id(), "3".to_string());
    
    //user 3 will be responsible for playing. (1) deposit test, (2) play test, (3) withdrawal test
    //besides testing expected state changes, also test if state thet shouldn't mutate has done so
    
    //deposit
    let deposit_amount = to_yocto("10");

    consumer3.call(
        coin_account.account_id(), 
        "deposit", 
        &json!({}).to_string().into_bytes(),
        DEFAULT_GAS, 
        deposit_amount
    ).assert_success();

    let consumer_balance0: String = consumer3.view(
        coin_account.account_id(), 
        "get_credits", 
        &json!({
            "account_id": consumer3.account_id()
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
    let mut dev_fee: u128;
    let mut house_fee: u128;

    let mut retrieved_state0: std::collections::HashMap<String, String>;
    let mut retrieved_state1: std::collections::HashMap<String, String>;
    let mut nft_balance0: u128;
    let mut nft_balance1: u128;
    let mut nft_expected_balance: u128;
    let mut dev_balance0: u128;
    let mut dev_balance1: u128;
    let mut dev_expected_balance: u128;

    loop {
        retrieved_state0 = consumer3.view(
            coin_account.account_id(), 
            "get_contract_state", 
            &json!({}).to_string().into_bytes(),
        ).unwrap_json();
    
        nft_balance0 = retrieved_state0.get("nft_balance").unwrap().parse().unwrap();
        dev_balance0 = retrieved_state0.get("dev_balance").unwrap().parse().unwrap();
    
        nft_fee = (bet_size * NFT_FEE) / FRACTIONAL_BASE;
        dev_fee = (bet_size * DEV_FEE) / FRACTIONAL_BASE;
        house_fee = (bet_size * HOUSE_FEE) / FRACTIONAL_BASE;
    
        nft_expected_balance = nft_balance0 + nft_fee;
        dev_expected_balance = dev_balance0 + dev_fee;
    
        consumer_balance1 = ViewResult::unwrap_json::<String>(&consumer3.view(
            coin_account.account_id(), 
            "get_credits", 
            &json!({
                "account_id": consumer3.account_id()
            }).to_string().into_bytes(),
        )).parse().unwrap();
    
        game_result = consumer3.call(
            coin_account.account_id(), 
            "play", 
            &json!({
                "_bet_type": true,
                "bet_size": bet_size.to_string()
            }).to_string().into_bytes(),
            DEFAULT_GAS, 
            0
        ).unwrap_json_value().as_bool().unwrap();
    
        println!("game won? {}", game_result);
    
        consumer_balance2 = ViewResult::unwrap_json::<String>(&consumer3.view(
            coin_account.account_id(), 
            "get_credits", 
            &json!({
                "account_id": consumer3.account_id()
            }).to_string().into_bytes(),
        )).parse().unwrap();
    
        retrieved_state1 = consumer3.view(
            coin_account.account_id(), 
            "get_contract_state", 
            &json!({}).to_string().into_bytes(),
        ).unwrap_json();
    
        nft_balance1 = retrieved_state1.get("nft_balance").unwrap().parse().unwrap();
        dev_balance1 = retrieved_state1.get("dev_balance").unwrap().parse().unwrap();
    
        net_bet = bet_size - nft_fee - dev_fee - house_fee;
    
        if game_result {
            expected_balance = consumer_balance1 - bet_size + ( (net_bet * WIN_MULTIPLIER) / FRACTIONAL_BASE );
        } else {
            expected_balance = consumer_balance1 - bet_size;
        }
    
        assert_eq!(nft_balance1, nft_expected_balance);
        assert_eq!(dev_balance1, dev_expected_balance);
        assert_eq!(consumer_balance2, expected_balance);

        if consumer_balance2 <= bet_size {
            break
        }

    }

    //withdrawal funds
    let consumer_near_balance0: u128 = consumer3.account().unwrap().amount;

    let consumer_balance3: u128 = ViewResult::unwrap_json::<String>(&consumer3.view(
        coin_account.account_id(), 
        "get_credits", 
        &json!({
            "account_id": consumer3.account_id()
        }).to_string().into_bytes(),
    )).parse().unwrap();
        
    //withdrawal
    consumer3.call(
        coin_account.account_id(), 
        "retrieve_credits", 
        &json!({}).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    let consumer_near_balance1: u128 = consumer3.account().unwrap().amount;

    let consumer_balance4: u128 = ViewResult::unwrap_json::<String>(&consumer3.view(
        coin_account.account_id(), 
        "get_credits", 
        &json!({
            "account_id": consumer3.account_id()
        }).to_string().into_bytes(),
    )).parse().unwrap();

    assert_eq!(consumer_balance4, 0);
    assert_eq!(consumer_near_balance0 + consumer_balance3, consumer_near_balance1);

    //withdrawal nft funds

    let state_view0: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let initial_dev_funds: u128 = state_view0.get("dev_balance").unwrap().parse().unwrap();
    let initial_nft_funds: u128 = state_view0.get("nft_balance").unwrap().parse().unwrap();

    let initial_dev_near_balance: u128 = dev_account.account().unwrap().amount;

    consumer3.call(
        coin_account.account_id(), 
        "retrieve_dev_funds", 
        &json!({}).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    let state_view1: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let final_dev_funds: u128 = state_view1.get("dev_balance").unwrap().parse().unwrap();
    let final_nft_funds: u128 = state_view1.get("nft_balance").unwrap().parse().unwrap();

    let final_dev_near_balance: u128 = dev_account.account().unwrap().amount;

    assert_eq!(final_dev_funds, 0);
    assert_eq!(final_dev_near_balance, initial_dev_near_balance + initial_dev_funds);
    assert_eq!(final_nft_funds, initial_nft_funds);
    assert!(final_nft_funds > 0);

    //withdrawal nft funds
    let state_view0: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let initial_dev_funds: u128 = state_view0.get("dev_balance").unwrap().parse().unwrap();
    let initial_nft_funds: u128 = state_view0.get("nft_balance").unwrap().parse().unwrap();

    let initial_consumer1_near_balance: u128 = consumer1.account().unwrap().amount;
    let initial_consumer2_near_balance: u128 = consumer2.account().unwrap().amount;
    let initial_consumer3_near_balance: u128 = consumer3.account().unwrap().amount;

    println!("A");

    consumer3.call(
        coin_account.account_id(), 
        "retrieve_nft_funds", 
        &json!({}).to_string().into_bytes(), 
        10000000000000000000, 
        0
    ).assert_success();

    println!("B");

    let state_view1: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let final_dev_funds: u128 = state_view1.get("dev_balance").unwrap().parse().unwrap();
    let final_nft_funds: u128 = state_view1.get("nft_balance").unwrap().parse().unwrap();

    let final_consumer1_near_balance: u128 = consumer1.account().unwrap().amount;
    let final_consumer2_near_balance: u128 = consumer2.account().unwrap().amount;
    let final_consumer3_near_balance: u128 = consumer3.account().unwrap().amount;

    assert_eq!(final_nft_funds, 0);
    assert_eq!(final_consumer1_near_balance, initial_consumer1_near_balance + 2 * ( initial_nft_funds / 3 ) );
    assert_eq!(final_consumer2_near_balance, initial_consumer2_near_balance + 1 * ( initial_nft_funds / 3 ) );
    assert_eq!(final_consumer3_near_balance, initial_consumer3_near_balance);
    assert_eq!(final_dev_funds, initial_dev_funds);
}

#[test]
fn simulate_full_flow_2() {
    //Test full flow from deploying app
    //users mint nfts
    //user deposits balance into the game
    //user plays the game multiple times
    //user retrieves his balance
    //contract owner retrieves nft balance
    //contract owner retrieves dev balance
    //this test is similar to simulate_full_flow_2, just changing order
    //of dev and nft balance retrievals because of runtime error observed in past versions

    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = u64::MAX;
    genesis.gas_price = 0;

    let root = init_simulator(Some(genesis));

    let dev_account = root.create_user("dev_account".to_string(), to_yocto("100"));
    
    let consumer1 = root.create_user("consumer1".to_string(), to_yocto("100"));
    let consumer2 = root.create_user("consumer2".to_string(), to_yocto("100"));
    let consumer3 = root.create_user("consumer3".to_string(), to_yocto("100"));

    // //deploy contracts
    let nft_account = root.deploy(
        &NFT_BYTES,
        "nft_contract".to_string(),
        to_yocto("100")
    );

    let coin_account = root.deploy(
        &COIN_BYTES,
        "coin_contract".to_string(),
        to_yocto("100")
    );

    root.call(
        nft_account.account_id(), 
        "new_default_meta", 
        &json!({
            "owner_id": dev_account.account_id()
        }).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    root.call(
        coin_account.account_id(), 
        "new", 
        &json!({"owner_id": dev_account.account_id(),
                "nft_id": nft_account.account_id(),
                "nft_fee": NFT_FEE.to_string(),
                "dev_fee": DEV_FEE.to_string(),
                "house_fee": HOUSE_FEE.to_string(),
                "win_multiplier": WIN_MULTIPLIER.to_string(),
                "base_gas": DEFAULT_GAS.to_string()
        }).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    //user 1 gets 2 NFTs, user 2 gets 1, user 3 gets none
    let nft_mint_token = | account: String, token_id: String | {
        dev_account.call(
            nft_account.account_id(), 
            "nft_mint", 
            &json!({
                "token_id": token_id,
                "receiver_id": account,
                "token_metadata": {}
            }).to_string().into_bytes(),
            DEFAULT_GAS, 
            5870000000000000000000
        ).assert_success();
    };

    nft_mint_token(consumer1.account_id(), "1".to_string());
    nft_mint_token(consumer1.account_id(), "2".to_string());
    nft_mint_token(consumer2.account_id(), "3".to_string());
    
    //user 3 will be responsible for playing. (1) deposit test, (2) play test, (3) withdrawal test
    //besides testing expected state changes, also test if state thet shouldn't mutate has done so
    
    //deposit
    let deposit_amount = to_yocto("10");

    consumer3.call(
        coin_account.account_id(), 
        "deposit", 
        &json!({}).to_string().into_bytes(),
        DEFAULT_GAS, 
        deposit_amount
    ).assert_success();

    let consumer_balance0: String = consumer3.view(
        coin_account.account_id(), 
        "get_credits", 
        &json!({
            "account_id": consumer3.account_id()
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
    let mut dev_fee: u128;
    let mut house_fee: u128;

    let mut retrieved_state0: std::collections::HashMap<String, String>;
    let mut retrieved_state1: std::collections::HashMap<String, String>;
    let mut nft_balance0: u128;
    let mut nft_balance1: u128;
    let mut nft_expected_balance: u128;
    let mut dev_balance0: u128;
    let mut dev_balance1: u128;
    let mut dev_expected_balance: u128;

    loop {
        retrieved_state0 = consumer3.view(
            coin_account.account_id(), 
            "get_contract_state", 
            &json!({}).to_string().into_bytes(),
        ).unwrap_json();
    
        nft_balance0 = retrieved_state0.get("nft_balance").unwrap().parse().unwrap();
        dev_balance0 = retrieved_state0.get("dev_balance").unwrap().parse().unwrap();
    
        nft_fee = (bet_size * NFT_FEE) / FRACTIONAL_BASE;
        dev_fee = (bet_size * DEV_FEE) / FRACTIONAL_BASE;
        house_fee = (bet_size * HOUSE_FEE) / FRACTIONAL_BASE;
    
        nft_expected_balance = nft_balance0 + nft_fee;
        dev_expected_balance = dev_balance0 + dev_fee;
    
        consumer_balance1 = ViewResult::unwrap_json::<String>(&consumer3.view(
            coin_account.account_id(), 
            "get_credits", 
            &json!({
                "account_id": consumer3.account_id()
            }).to_string().into_bytes(),
        )).parse().unwrap();
    
        game_result = consumer3.call(
            coin_account.account_id(), 
            "play", 
            &json!({
                "_bet_type": true,
                "bet_size": bet_size.to_string()
            }).to_string().into_bytes(),
            DEFAULT_GAS, 
            0
        ).unwrap_json_value().as_bool().unwrap();
    
        println!("game won? {}", game_result);
    
        consumer_balance2 = ViewResult::unwrap_json::<String>(&consumer3.view(
            coin_account.account_id(), 
            "get_credits", 
            &json!({
                "account_id": consumer3.account_id()
            }).to_string().into_bytes(),
        )).parse().unwrap();
    
        retrieved_state1 = consumer3.view(
            coin_account.account_id(), 
            "get_contract_state", 
            &json!({}).to_string().into_bytes(),
        ).unwrap_json();
    
        nft_balance1 = retrieved_state1.get("nft_balance").unwrap().parse().unwrap();
        dev_balance1 = retrieved_state1.get("dev_balance").unwrap().parse().unwrap();
    
        net_bet = bet_size - nft_fee - dev_fee - house_fee;
    
        if game_result {
            expected_balance = consumer_balance1 - bet_size + ( (net_bet * WIN_MULTIPLIER) / FRACTIONAL_BASE );
        } else {
            expected_balance = consumer_balance1 - bet_size;
        }
    
        assert_eq!(nft_balance1, nft_expected_balance);
        assert_eq!(dev_balance1, dev_expected_balance);
        assert_eq!(consumer_balance2, expected_balance);

        if consumer_balance2 <= bet_size {
            break
        }

    }

    //withdrawal funds
    let consumer_near_balance0: u128 = consumer3.account().unwrap().amount;

    let consumer_balance3: u128 = ViewResult::unwrap_json::<String>(&consumer3.view(
        coin_account.account_id(), 
        "get_credits", 
        &json!({
            "account_id": consumer3.account_id()
        }).to_string().into_bytes(),
    )).parse().unwrap();
        
    //withdrawal
    consumer3.call(
        coin_account.account_id(), 
        "retrieve_credits", 
        &json!({}).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    let consumer_near_balance1: u128 = consumer3.account().unwrap().amount;

    let consumer_balance4: u128 = ViewResult::unwrap_json::<String>(&consumer3.view(
        coin_account.account_id(), 
        "get_credits", 
        &json!({
            "account_id": consumer3.account_id()
        }).to_string().into_bytes(),
    )).parse().unwrap();

    assert_eq!(consumer_balance4, 0);
    assert_eq!(consumer_near_balance0 + consumer_balance3, consumer_near_balance1);

    //withdrawal nft funds
    let state_view0: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let initial_dev_funds: u128 = state_view0.get("dev_balance").unwrap().parse().unwrap();
    let initial_nft_funds: u128 = state_view0.get("nft_balance").unwrap().parse().unwrap();

    let initial_consumer1_near_balance: u128 = consumer1.account().unwrap().amount;
    let initial_consumer2_near_balance: u128 = consumer2.account().unwrap().amount;
    let initial_consumer3_near_balance: u128 = consumer3.account().unwrap().amount;

    consumer3.call(
        coin_account.account_id(), 
        "retrieve_nft_funds", 
        &json!({}).to_string().into_bytes(), 
        10000000000000000000, 
        0
    ).assert_success();

    let state_view1: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let final_dev_funds: u128 = state_view1.get("dev_balance").unwrap().parse().unwrap();
    let final_nft_funds: u128 = state_view1.get("nft_balance").unwrap().parse().unwrap();

    let final_consumer1_near_balance: u128 = consumer1.account().unwrap().amount;
    let final_consumer2_near_balance: u128 = consumer2.account().unwrap().amount;
    let final_consumer3_near_balance: u128 = consumer3.account().unwrap().amount;

    assert_eq!(final_nft_funds, 0);
    assert_eq!(final_consumer1_near_balance, initial_consumer1_near_balance + 2 * ( initial_nft_funds / 3 ) );
    assert_eq!(final_consumer2_near_balance, initial_consumer2_near_balance + 1 * ( initial_nft_funds / 3 ) );
    assert_eq!(final_consumer3_near_balance, initial_consumer3_near_balance);
    assert_eq!(final_dev_funds, initial_dev_funds);
    assert!(final_dev_funds > 0);

    //withdrawal dev funds
    let state_view0: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let initial_dev_funds: u128 = state_view0.get("dev_balance").unwrap().parse().unwrap();
    let initial_nft_funds: u128 = state_view0.get("nft_balance").unwrap().parse().unwrap();

    let initial_dev_near_balance: u128 = dev_account.account().unwrap().amount;

    consumer3.call(
        coin_account.account_id(), 
        "retrieve_dev_funds", 
        &json!({}).to_string().into_bytes(),
        DEFAULT_GAS, 
        0
    ).assert_success();

    let state_view1: std::collections::HashMap<String, String> = consumer3.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let final_dev_funds: u128 = state_view1.get("dev_balance").unwrap().parse().unwrap();
    let final_nft_funds: u128 = state_view1.get("nft_balance").unwrap().parse().unwrap();

    let final_dev_near_balance: u128 = dev_account.account().unwrap().amount;

    assert_eq!(final_dev_funds, 0);
    assert_eq!(final_dev_near_balance, initial_dev_near_balance + initial_dev_funds);
    assert_eq!(final_nft_funds, initial_nft_funds);

}