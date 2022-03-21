use std::convert::TryInto;
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U128, U64};
pub use near_sdk::serde_json::{json, value::Value};
pub use near_sdk_sim::{call, view, deploy, init_simulator, to_yocto, UserAccount, 
    ContractAccount, DEFAULT_GAS, ViewResult, ExecutionResult};
pub use near_sdk::AccountId;
use near_contract_standards::non_fungible_token::{ Token, TokenId };

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    COIN_BYTES => "./target/wasm32-unknown-unknown/release/classy_kangaroo_coin_flip.wasm",
    NFT_BYTES => "../nep_171/target/wasm32-unknown-unknown/release/non_fungible_token.wasm",
}


const NFT_FEE: u128 = 4_000;
const DEV_FEE: u128 = 500;
const HOUSE_FEE: u128 = 500;
const WIN_MULTIPLIER: u128 = 200_000;
const FRACTIONAL_BASE: u128 = 100_000;
const MIN_BALANCE_FRACTION: u128 = 100;
const NFT_MAPPING_SIZE: u128 = 50;

const GAS_ATTACHMENT: u64 = 300_000_000_000_000;

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
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    let max_bet: u128 = to_yocto("5");
    let min_bet: u128 = to_yocto("0.05");
    root.call(
        coin_account.account_id(), 
        "new", 
        &json!({"owner_id": dev_account.account_id(),
                "nft_id": nft_account.account_id(),
                "nft_fee": NFT_FEE.to_string(),
                "dev_fee": DEV_FEE.to_string(),
                "house_fee": HOUSE_FEE.to_string(),
                "win_multiplier": WIN_MULTIPLIER.to_string(),
                "base_gas": GAS_ATTACHMENT.to_string(),
                "max_bet": max_bet.to_string(),
                "min_bet": min_bet.to_string(),
                "min_balance_fraction": MIN_BALANCE_FRACTION.to_string(),
                "nft_mapping_size": NFT_MAPPING_SIZE.to_string()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
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
            GAS_ATTACHMENT, 
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
        GAS_ATTACHMENT, 
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
            GAS_ATTACHMENT, 
            0
        ).unwrap_json_value().as_bool().unwrap();
    
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
        GAS_ATTACHMENT, 
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
        GAS_ATTACHMENT, 
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

    consumer3.call(
        coin_account.account_id(), 
        "retrieve_nft_funds", 
        &json!({
            "distribution_list": vec![&consumer1.account_id, &consumer1.account_id, &consumer2.account_id]
        }).to_string().into_bytes(), 
        GAS_ATTACHMENT, 
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
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    let max_bet: u128 = to_yocto("5");
    let min_bet: u128 = to_yocto("0.05");
    root.call(
        coin_account.account_id(), 
        "new", 
        &json!({"owner_id": dev_account.account_id(),
                "nft_id": nft_account.account_id(),
                "nft_fee": NFT_FEE.to_string(),
                "dev_fee": DEV_FEE.to_string(),
                "house_fee": HOUSE_FEE.to_string(),
                "win_multiplier": WIN_MULTIPLIER.to_string(),
                "base_gas": GAS_ATTACHMENT.to_string(),
                "max_bet": max_bet.to_string(),
                "min_bet": min_bet.to_string(),
                "min_balance_fraction": MIN_BALANCE_FRACTION.to_string(),
                "nft_mapping_size": NFT_MAPPING_SIZE.to_string()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
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
            GAS_ATTACHMENT, 
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
        GAS_ATTACHMENT, 
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
            GAS_ATTACHMENT, 
            0
        ).unwrap_json_value().as_bool().unwrap();

    
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
        GAS_ATTACHMENT, 
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
        &json!({
            "distribution_list": vec![&consumer1.account_id, &consumer1.account_id, &consumer2.account_id]
        }).to_string().into_bytes(), 
        GAS_ATTACHMENT, 
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
        GAS_ATTACHMENT, 
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

#[test]
fn test_preservation_of_state_in_play_function() {
        //Test deposit/play/withdrawal flow
        //check that contract state is maitained apart from items tha SHOULD change
        //for instance, one player depositing cannot alter the state of other players deposits

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
            GAS_ATTACHMENT, 
            0
        ).assert_success();

        let max_bet: u128 = to_yocto("5");
        let min_bet: u128 = to_yocto("0.05");
        root.call(
            coin_account.account_id(), 
            "new", 
            &json!({"owner_id": dev_account.account_id(),
                    "nft_id": nft_account.account_id(),
                    "nft_fee": NFT_FEE.to_string(),
                    "dev_fee": DEV_FEE.to_string(),
                    "house_fee": HOUSE_FEE.to_string(),
                    "win_multiplier": WIN_MULTIPLIER.to_string(),
                    "base_gas": GAS_ATTACHMENT.to_string(),
                    "max_bet": max_bet.to_string(),
                    "min_bet": min_bet.to_string(),
                    "min_balance_fraction": MIN_BALANCE_FRACTION.to_string(),
                    "nft_mapping_size": NFT_MAPPING_SIZE.to_string()
            }).to_string().into_bytes(),
            GAS_ATTACHMENT, 
            0
        ).assert_success();
        
        //deposit
        let call_deposit = |consumer: &UserAccount, deposit_amount: u128| {
            consumer.call(
                coin_account.account_id(), 
                "deposit", 
                &json!({}).to_string().into_bytes(),
                GAS_ATTACHMENT, 
                deposit_amount
            ).assert_success();
        };

        let view_balance = |consumer: &UserAccount| -> String {
            consumer.view(
                coin_account.account_id(), 
                "get_credits", 
                &json!({
                    "account_id": consumer.account_id()
                }).to_string().into_bytes(),
            ).unwrap_json()
        };
        
        let deposit_amount1 = to_yocto("10");
        let deposit_amount2 = to_yocto("20");
        let deposit_amount3 = to_yocto("50");

        let consumer1_balance0 = view_balance(&consumer1);
        let consumer2_balance0 = view_balance(&consumer2);
        let consumer3_balance0 = view_balance(&consumer3);

        assert_eq!(consumer1_balance0, "0");
        assert_eq!(consumer2_balance0, "0");
        assert_eq!(consumer3_balance0, "0");

        call_deposit(&consumer1, deposit_amount1);

        let consumer1_balance1 = view_balance(&consumer1);
        let consumer2_balance1 = view_balance(&consumer2);
        let consumer3_balance1 = view_balance(&consumer3);

        assert_eq!(consumer1_balance1, deposit_amount1.to_string());
        assert_eq!(consumer2_balance1, "0");
        assert_eq!(consumer3_balance1, "0");

        call_deposit(&consumer2, deposit_amount2);

        let consumer1_balance2 = view_balance(&consumer1);
        let consumer2_balance2 = view_balance(&consumer2);
        let consumer3_balance2 = view_balance(&consumer3);

        assert_eq!(consumer1_balance2, deposit_amount1.to_string());
        assert_eq!(consumer2_balance2, deposit_amount2.to_string());
        assert_eq!(consumer3_balance2, "0");

        call_deposit(&consumer3, deposit_amount3);

        let consumer1_balance3 = view_balance(&consumer1);
        let consumer2_balance3 = view_balance(&consumer2);
        let consumer3_balance3 = view_balance(&consumer3);

        assert_eq!(consumer1_balance3, deposit_amount1.to_string());
        assert_eq!(consumer2_balance3, deposit_amount2.to_string());
        assert_eq!(consumer3_balance3, deposit_amount3.to_string());


        //play repeatedly and check user balance changes
        //check immutability of other user balances
        //check nft and dev balances
        let bet_size: u128 = to_yocto("1");
        let game_result: bool;

        let net_bet: u128;
        let expected_balance: u128;

        let nft_fee: u128;
        let dev_fee: u128;
        let house_fee: u128;
        
        nft_fee = (bet_size * NFT_FEE) / FRACTIONAL_BASE;
        dev_fee = (bet_size * DEV_FEE) / FRACTIONAL_BASE;
        house_fee = (bet_size * HOUSE_FEE) / FRACTIONAL_BASE;
    
        let consumer1_balance4: u128 = view_balance(&consumer1).parse().unwrap();
        let consumer2_balance4: u128 = view_balance(&consumer2).parse().unwrap();
        let consumer3_balance4: u128 = view_balance(&consumer3).parse().unwrap();
    
        game_result = consumer3.call(
            coin_account.account_id(), 
            "play", 
            &json!({
                "_bet_type": true,
                "bet_size": bet_size.to_string()
            }).to_string().into_bytes(),
            GAS_ATTACHMENT, 
            0
        ).unwrap_json_value().as_bool().unwrap();
    
        let consumer1_balance5: u128 = view_balance(&consumer1).parse().unwrap();
        let consumer2_balance5: u128 = view_balance(&consumer2).parse().unwrap();
        let consumer3_balance5: u128 = view_balance(&consumer3).parse().unwrap();
    
        net_bet = bet_size - nft_fee - dev_fee - house_fee;
    
        if game_result {
            expected_balance = consumer3_balance4 - bet_size + ( (net_bet * WIN_MULTIPLIER) / FRACTIONAL_BASE );
        } else {
            expected_balance = consumer3_balance4 - bet_size;
        }
    
        assert_eq!(expected_balance, consumer3_balance5);
        assert_eq!(consumer2_balance4, consumer2_balance5);
        assert_eq!(consumer1_balance4, consumer1_balance5);

        //withdrawal funds
        let consumer1_near_balance0: u128 = consumer1.account().unwrap().amount;
        let consumer2_near_balance0: u128 = consumer2.account().unwrap().amount;
        let consumer3_near_balance0: u128 = consumer3.account().unwrap().amount;

        let consumer1_balance6: u128 = view_balance(&consumer1).parse().unwrap();
        let consumer2_balance6: u128 = view_balance(&consumer2).parse().unwrap();
        let consumer3_balance6: u128 = view_balance(&consumer3).parse().unwrap();
            
        //withdrawal
        consumer3.call(
            coin_account.account_id(), 
            "retrieve_credits", 
            &json!({}).to_string().into_bytes(),
            GAS_ATTACHMENT, 
            0
        ).assert_success();

        let consumer1_near_balance1: u128 = consumer1.account().unwrap().amount;
        let consumer2_near_balance1: u128 = consumer2.account().unwrap().amount;
        let consumer3_near_balance1: u128 = consumer3.account().unwrap().amount;

        let consumer1_balance7: u128 = view_balance(&consumer1).parse().unwrap();
        let consumer2_balance7: u128 = view_balance(&consumer2).parse().unwrap();
        let consumer3_balance7: u128 = view_balance(&consumer3).parse().unwrap();

        assert_eq!(consumer1_balance6, consumer1_balance7);
        assert_eq!(consumer2_balance6, consumer2_balance7);
        assert_eq!(0, consumer3_balance7);

        assert_eq!(consumer1_near_balance1, consumer1_near_balance0);
        assert_eq!(consumer2_near_balance1, consumer2_near_balance0);
        assert_eq!(consumer3_near_balance1, consumer3_near_balance0 + consumer3_balance6);

    }


#[test]
fn simulate_n_nft_holders() {
    //Test full flow from deploying app (without dev retrieval)
    //n different users mint nfts
    //1 user deposits balance into the game
    //1 user plays the game multiple times
    //contract owner retrieves nft balance
    //check if contract handles the volume

    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = u64::MAX;
    genesis.gas_price = 0;

    const N: u128 = 550;

    let root = init_simulator(Some(genesis));

    let dev_account = root.create_user("dev_account".to_string(), to_yocto("100"));
    
    let consumer = root.create_user("consumer".to_string(), to_yocto("100000"));

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
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    let max_bet: u128 = to_yocto("100000000");
    let min_bet: u128 = to_yocto("0.05");
    root.call(
        coin_account.account_id(), 
        "new", 
        &json!({"owner_id": dev_account.account_id(),
                "nft_id": nft_account.account_id(),
                "nft_fee": NFT_FEE.to_string(),
                "dev_fee": DEV_FEE.to_string(),
                "house_fee": HOUSE_FEE.to_string(),
                "win_multiplier": WIN_MULTIPLIER.to_string(),
                "base_gas": GAS_ATTACHMENT.to_string(),
                "max_bet": max_bet.to_string(),
                "min_bet": min_bet.to_string(),
                "min_balance_fraction": MIN_BALANCE_FRACTION.to_string(),
                "nft_mapping_size": NFT_MAPPING_SIZE.to_string()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    //user 1 gets 2 NFTs, user 2 gets 1, user 3 gets none
    let nft_mint_token = | account: String, token_id: u128 | {
        dev_account.call(
            nft_account.account_id(), 
            "nft_mint", 
            &json!({
                "token_id": U128(token_id),
                "receiver_id": account,
                "token_metadata": {}
            }).to_string().into_bytes(),
            GAS_ATTACHMENT, 
            5870000000000000000000
        ).assert_success();
    };

    let mut account_vector: Vec<UserAccount> = Vec::new();
    let mut counter: u128 = 1;
    let mut account_id: String;
    while counter < N {
        account_id = format!("consumer{}", counter);
        account_vector.push(root.create_user(account_id.clone(), to_yocto("100")));
        nft_mint_token(account_id, counter);
        counter += 1;
    }

    let mut account_arr: Vec<String> = Vec::<String>::new();

    for item in account_vector.iter() {
        account_arr.push(item.account_id.clone());
    }
    
    let deposit_amount = to_yocto("10000");

    consumer.call(
        coin_account.account_id(), 
        "deposit", 
        &json!({}).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        deposit_amount
    ).assert_success();

    //play repeatedly
    let bet_size: u128 = to_yocto("1000");
    let mut consumer_balance: u128;

    loop {
     
        consumer.call(
            coin_account.account_id(), 
            "play", 
            &json!({
                "_bet_type": true,
                "bet_size": bet_size.to_string()
            }).to_string().into_bytes(),
            GAS_ATTACHMENT, 
            0
        ).unwrap_json_value().as_bool().unwrap();
    
        consumer_balance = ViewResult::unwrap_json::<String>(&consumer.view(
            coin_account.account_id(), 
            "get_credits", 
            &json!({
                "account_id": consumer.account_id()
            }).to_string().into_bytes(),
        )).parse().unwrap();

        if consumer_balance <= bet_size {
            break
        }

    }

    //withdrawal nft funds

    let retrieved_state: std::collections::HashMap<String, String> = consumer.view(
        coin_account.account_id(), 
        "get_contract_state", 
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();

    let nft_balance: u128 = retrieved_state.get("nft_balance").unwrap().parse().unwrap();
    let ideal_share: u128 = nft_balance / (N - 1);

    let get_near_balance = |account: &UserAccount| -> u128 {
        account.account().unwrap().amount
    };

    let mut initial_balances_vector: Vec<u128> = Vec::new();
    for item in account_vector.iter() {
        initial_balances_vector.push(get_near_balance(item));
    }

    root.call(
        coin_account.account_id(), 
        "retrieve_nft_funds", 
        &json!({
            "distribution_list": account_arr
        }).to_string().into_bytes(), 
        GAS_ATTACHMENT, 
        0
    );

    // println!("{:#?}", exe_result);

    let mut final_balances_vector: Vec<u128> = Vec::new();
    for item in account_vector.iter() {
        final_balances_vector.push(get_near_balance(item));
    }

    let mut index_counter: usize = 0;
    println!("{}", ideal_share);
    while index_counter < (N - 1).try_into().unwrap() {
        println!("{}", account_vector[index_counter].account_id);
        assert_eq!(initial_balances_vector[index_counter] + ideal_share, final_balances_vector[index_counter], "{}", account_vector[index_counter].account_id );
        index_counter += 1;
    }

}