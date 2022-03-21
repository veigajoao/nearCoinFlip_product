import nearAPI from "near-api-js";
import BN from "bn.js";
import BigNumber from "bignumber.js";
import fs from "fs";
import assert from "assert";

const { keyStores, KeyPair } = nearAPI;
const keyStore = new keyStores.InMemoryKeyStore();
const PRIVATE_KEY =
    "ed25519:4V94VcSiteis8KGAS3Xwe2XBWg8oNuPcyeYUb2ssxruztRipLMX9z5kTHpYKQWk9diFFcvfnYoRrEJtVShxyvaTw";
// creates a public / private key pair using the provided private key
const keyPair = KeyPair.fromString(PRIVATE_KEY);
// adds the keyPair you created to keyStore
await keyStore.setKey("testnet", "example-account.testnet", keyPair);

const config = {
    networkId: "mainnet",
    keyStore, // optional if not signing transactions
    nodeUrl: "https://rpc.mainnet.near.org",
    walletUrl: "https://wallet.mainnet.near.org",
    helperUrl: "https://helper.mainnet.near.org",
    explorerUrl: "https://explorer.mainnet.near.org",
};
const near = await nearAPI.connect(config);

let connector = new nearAPI.Account(near.connection, "classykangaroos1.near");

let nftCount = 550;
let pagination = 10;
let currentSize = 0;
let nftList = [];
let result;
while (currentSize < nftCount) {
    result = await connector.viewFunction("classykangaroos1.near", "nft_tokens", { from_index: currentSize.toString(), limit: pagination })
    nftList.push(...result);
    currentSize = nftList.length;
}

let nftDistributionArgument = nftList.map((item) => {
    return item.owner_id
});

console.log(nftDistributionArgument);

//send transaction to distribute funds
let sendCall = connector.callFunction("contract.address");