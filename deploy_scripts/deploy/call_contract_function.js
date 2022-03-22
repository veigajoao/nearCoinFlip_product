import nearAPI from "near-api-js";
import loginNear from "./_login.js";

import buildContractObject from "./_contract_object.js";

async function initializeContract(ownerAccount, contractAccount, params) {
    const contract = await buildContractObject(ownerAccount, contractAccount);

    let namedArgs = {
        owner_id: ownerAccount,
        nft_fee: params.nftFee,
        dev_fee: params.devFee,
        house_fee: params.houseFee,
        win_multiplier: params.winMultiplier,
        max_bet: params.maxBet,
        min_bet: params.minBet,
        min_balance_fraction: params.minBalanceFraction
    };

    const result = await contract.new(
        namedArgs,
        "300000000000000"
    );

    console.log(result);
    return result;
}

//emergency_panic
async function emergencyPanic(ownerAccount, contractAccount, withdrawalAmount) {
    const contract = await buildContractObject(ownerAccount, contractAccount);

    const result = await contract.emergency_panic({
            withdrawal_balance: withdrawalAmount.toString()
        },
        "300000000000000",
        "1"
    );

    console.log(result);
    return result;
}

//get_contract_state
async function getContractState(ownerAccount, contractAccount) {
    const contract = await buildContractObject(ownerAccount, contractAccount);

    const result = await contract.get_contract_state({},
        "300000000000000",
    );

    console.log(result);
    return result;
}

//update_contract
async function updateContract(ownerAccount, contractAccount, params) {
    const contract = await buildContractObject(ownerAccount, contractAccount);

    let namedArgs = {
        nft_fee: params.nftFee,
        dev_fee: params.devFee,
        house_fee: params.houseFee,
        win_multiplier: params.winMultiplier,
        max_bet: params.maxBet,
        min_bet: params.minBet,
        min_balance_fraction: params.minBalanceFraction
    };

    const result = await contract.update_contract(
        namedArgs,
        "300000000000000",
        "1"
    );

    console.log(result);
    return result;
}

//retrieve_dev_funds
async function retrieveDevFunds(ownerAccount, contractAccount) {
    const contract = await buildContractObject(ownerAccount, contractAccount);

    const result = await contract.retrieve_dev_funds({},
        "300000000000000",
        "1"
    );

    console.log(result);
    return result;
}

//retrieve_nft_funds
async function retrieveNftFunds(ownerAccount, contractAccount, nftContractAccount) {
    const contract = await buildContractObject(ownerAccount, contractAccount);
    const { near, account } = await loginNear(ownerAccount);

    const tokenCount = await account.viewFunction(nftContractAccount, "nft_total_supply", {});
    let nftCount = parseInt(tokenCount);
    let pagination = 10;
    let currentSize = 0;
    let nftList = [];
    let fetchResult;
    while (currentSize < nftCount) {
        fetchResult = await account.viewFunction(nftContractAccount, "nft_tokens", { from_index: currentSize.toString(), limit: pagination })
        nftList.push(...result);
        currentSize = nftList.length;
    }

    await contract.retrieve_nft_funds({
            distribution_list: nftList
        },
        "30000000000000000",
        "1"
    );

    console.log("retrieval successfull");
}

export { initializeContract, emergencyPanic, getContractState, updateContract, retrieveDevFunds, retrieveNftFunds };