import { deployContractAccountExists, deployContractNewAccount } from './deploy_contract.js';
import { initializeContract, emergencyPanic, getContractState, updateContract, retrieveDevFunds, retrieveNftFunds, deposit, play, mintNft, initializeNft } from "./call_contract_function.js";

// //deploy nft
// await deployContractAccountExists("coinfliptest-nft.testnet", "/home/jveiga/CKcoinFlip/coin_flip/nep_171/target/wasm32-unknown-unknown/release/non_fungible_token.wasm");

//mint nfts
// await initializeNft("coinfliptest-nft.testnet", "ckcoinflip.testnet");
// await mintNft("coinfliptest-nft.testnet", "ckcoinflip.testnet", "ckcoinflip.testnet", "1");
// await mintNft("coinfliptest-nft.testnet", "ckcoinflip.testnet", "ckcoinflip.testnet", "2");
// await mintNft("coinfliptest-nft.testnet", "ckcoinflip.testnet", "coinflip-test.testnet", "3");
// await mintNft("coinfliptest-nft.testnet", "ckcoinflip.testnet", "coinflip-test.testnet", "4");
// await mintNft("coinfliptest-nft.testnet", "ckcoinflip.testnet", "coinflip-test2.testnet", "5");

// let counter = 10;
// while (counter < 700) {
//     await mintNft("coinfliptest-nft.testnet", "ckcoinflip.testnet", "coinflip-test.testnet", counter.toString());
//     counter += 1;
// }



// //deploy coin flip
// await deployContractAccountExists("coinfliptest-contract.testnet", "/home/jveiga/CKcoinFlip/coin_flip/coin_flip_contract/target/wasm32-unknown-unknown/release/classy_kangaroo_coin_flip.wasm");

//initialize coin flip
// let params = {
//     nftFee: "4000",
//     devFee: "500",
//     houseFee: "500",
//     winMultiplier: "200000",
//     maxBet: "5000000000000000000000000",
//     minBet: "100000000000000000000000",
//     minBalanceFraction: "100"
// }
// await initializeContract("coinfliptest-dev.testnet", "coinfliptest-contract.testnet", params);

// loop play
// await deposit("ckcoinflip.testnet", "coinfliptest-contract.testnet");

// while (true) {
//     await play("ckcoinflip.testnet", "coinfliptest-contract.testnet", true, "5");
// }

//check contract state
// console.log(await getContractState("coinfliptest-dev.testnet", "coinfliptest-contract.testnet"));

// //withdraw
// retrieveNftFunds("coinfliptest-dev.testnet", "coinfliptest-contract.testnet", "coinfliptest-nft.testnet");

// nft balance