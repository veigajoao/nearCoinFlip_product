const nearAPI = require("near-api-js");
const BN = require("bn.js");
const fs = require("fs").promises;
const assert = require("assert").strict;

function getConfig(env) {
    switch (env) {
        case "sandbox":
        case "local":
            return {
                networkId: "sandbox",
                nodeUrl: "http://localhost:3030",
                masterAccount: "test.near",
                coinContractAccount: "coin.test.near",
                nftContractAccount: "nft.test.near",
                keyPath: "/tmp/near-sandbox/validator_key.json",
            };
    }
}

const coinContractMethods = {
    viewMethods: ["get_credits", "get_contract_state"],
    changeMethods: ["deposit", "retrieve_credits", "play", "retrieve_dev_funds", "retrieve_nft_funds", "update_contract"],
};

const nftContractMethods = {
    viewMethods: ["nft_tokens"],
    changeMethods: ["nft_mint", "new_default_meta"],
};

let config;
let masterAccount;
let masterKey;
let pubKey;
let keyStore;
let near;

async function initNear() {
    config = getConfig(process.env.NEAR_ENV || "sandbox");
    const keyFile = require(config.keyPath);
    masterKey = nearAPI.utils.KeyPair.fromString(
        keyFile.secret_key || keyFile.private_key
    );
    pubKey = masterKey.getPublicKey();
    keyStore = new nearAPI.keyStores.InMemoryKeyStore();
    keyStore.setKey(config.networkId, config.masterAccount, masterKey);
    near = await nearAPI.connect({
        deps: {
            keyStore,
        },
        networkId: config.networkId,
        nodeUrl: config.nodeUrl,
    });
    masterAccount = new nearAPI.Account(near.connection, config.masterAccount);
    console.log("Finish init NEAR");
}

async function createUser(accountPrefix) {
    let accountId = accountPrefix + "." + config.masterAccount;
    await masterAccount.createAccount(
        accountId,
        pubKey,
        new BN(10).pow(new BN(25))
    );
    keyStore.setKey(config.networkId, accountId, masterKey);
    const account = new nearAPI.Account(near.connection, accountId);
    return account;
}

async function createContractUser(
    account,
    contractAccountId,
    contractMethods
) {
    const accountUseContract = new nearAPI.Contract(
        account,
        contractAccountId,
        contractMethods
    );
    return accountUseContract;
}

async function initTest() {
    const coinContractBinary = await fs.readFile("../coin_flip_contract/target/wasm32-unknown-unknown/release/classy_kangaroo_coin_flip.wasm");
    const _coinContractAccount = await masterAccount.createAndDeployContract(
        config.coinContractAccount,
        pubKey,
        coinContractBinary,
        new BN(10).pow(new BN(25))
    );

    const nftContractBinary = await fs.readFile("../nep_171/target/wasm32-unknown-unknown/release/non_fungible_token.wasm");
    const _nftContractAccount = await masterAccount.createAndDeployContract(
        config.nftContractAccount,
        pubKey,
        nftContractBinary,
        new BN(10).pow(new BN(25))
    );

    const devUser = await createUser("dev");

    const hodler1 = await createUser("hodler1");
    const hodler2 = await createUser("hodler2");
    const hodler3 = await createUser("hodler3");

    const devNftUser = await createContractUser(devUser, config.nftContractAccount, nftContractMethods);
    const devCoinUser = await createContractUser(devUser, config.coinContractAccount, coinContractMethods);

    const hodler1NftUser = await createContractUser(hodler1, config.nftContractAccount, nftContractMethods);
    const hodler1CoinUser = await createContractUser(hodler1, config.coinContractAccount, coinContractMethods);

    const hodler2NftUser = await createContractUser(hodler2, config.nftContractAccount, nftContractMethods);
    const hodler2CoinUser = await createContractUser(hodler2, config.coinContractAccount, coinContractMethods);

    const hodler3NftUser = await createContractUser(hodler3, config.nftContractAccount, nftContractMethods);
    const hodler3CoinUser = await createContractUser(hodler3, config.coinContractAccount, coinContractMethods);

    console.log("Finish deploy contracts and create test accounts");
    return {
        devUser,
        hodler1,
        hodler2,
        hodler3,
        devNftUser,
        devCoinUser,
        hodler1NftUser,
        hodler1CoinUser,
        hodler2NftUser,
        hodler2CoinUser,
        hodler3NftUser,
        hodler3CoinUser
    };
}

async function test() {
    // 1. Creates testing accounts and deploys a contract
    await initNear();
    const {
        devUser,
        hodler1,
        hodler2,
        hodler3,
        devNftUser,
        devCoinUser,
        hodler1NftUser,
        hodler1CoinUser,
        hodler2NftUser,
        hodler2CoinUser,
        hodler3NftUser,
        hodler3CoinUser
    } = await initTest();

    // 2. initialize deployed contracts
    await hodler1NftUser.new_default_meta({ args: { owner_id: hodler1.accountId } });
    await devCoinUser.new({
        args: {
            owner_id: config.coinContractAccount,
            nft_id: config.nftContractAccount,
            nft_fee: "4000",
            dev_fee: "500",
            house_fee: "500",
            win_multiplier: "20000",
            base_gas: nearAPI.utils.format.parseNearAmount("0.0005")
        }
    });

    // 2. mints 1 nft for each account
    await hodler1NftUser.nft_mint({ args: { token_id: "1", receiver_id: hodler1.accountId, token_metadata: {} }, amount: "15350000000000000000000" });
    await hodler1NftUser.nft_mint({ args: { token_id: "2", receiver_id: hodler2.accountId, token_metadata: {} }, amount: "15350000000000000000000" });
    await hodler1NftUser.nft_mint({ args: { token_id: "3", receiver_id: hodler3.accountId, token_metadata: {} }, amount: "15350000000000000000000" });

    let nftsBlob = await hodler1NftUser.nft_tokens({ args: {} });

    assert.equal(nftsBlob[0].owner_id, hodler1.accountId);
    assert.equal(nftsBlob[1].owner_id, hodler2.accountId);
    assert.equal(nftsBlob[2].owner_id, hodler3.accountId);

    // 3. check initial balance for dev accounts and hodler accounts
    let devBalance = await devUser.getAccountBalance();
    let hodler1Balance = await hodler1.getAccountBalance();
    let hodler2Balance = await hodler2.getAccountBalance();
    let hodler3Balance = await hodler3.getAccountBalance();

    devBalance = devBalance.total;
    hodler1Balance = hodler1Balance.total;
    hodler2Balance = hodler2Balance.total;
    hodler3Balance = hodler3Balance.total;

    // 4. test deposit function
    let depositAmount = nearAPI.utils.format.parseNearAmount("1");


    // 5. play games and check that balance was added to dev and nft holders in game

    // 6. test dev distribution

    // 7. test nft holders distribution

    // 8. test withdrawal function

    // 9. test state changing function
}

test();