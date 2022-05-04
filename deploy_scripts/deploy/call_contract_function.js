import nearAPI from "near-api-js";
import loginNear from "./_login.js";

import buildContractObject from "./_contract_object.js";
import { BN } from "bn.js";

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

    //get nft count
    const tokenCount = await account.viewFunction(nftContractAccount, "nft_total_supply", {});
    let nftCount = parseInt(tokenCount);

    //get list of all hodlers
    let pagination = 1;
    let currentSize = 0;
    let nftList = [];
    let fetchResult;

    let currentPagination = 0;

    while (currentSize < nftCount) {
        console.log(currentSize);
        fetchResult = await account.viewFunction(nftContractAccount, "nft_token", { token_id: currentPagination.toString() });
        currentPagination += 1;
        if (fetchResult === null) {
            continue
        }
        nftList.push(fetchResult);
        currentSize = nftList.length;
    }

    //get current state
    let contractState = await account.viewFunction(contractAccount, "get_contract_state", {});
    let nftBalance = contractState.nft_balance


    //retrieve funds
    await contract.retrieve_nft_funds({
            distribution_list: [account.accountId]
        },
        "300000000000000",
        "1"
    );

    let nftListNoMarketplace = [];
    for (let holderAccount of nftList) {
        if (Object.keys(holderAccount.approved_account_ids).length === 0) nftListNoMarketplace.push(holderAccount);
    }


    //get value per account
    let bnNftCount = new BN(nftListNoMarketplace.length);
    let bnNftBalance = new BN(nftBalance);
    let idealShare = bnNftBalance.div(bnNftCount);
    let idealShareString = idealShare.toString(10);

    let receipt;
    let receiptList = [];
    let objectTransaction;

    for (let holderAccount of nftListNoMarketplace) {
        console.log(`${holderAccount.token_id}/${nftListNoMarketplace.length}`);
        try {
            receipt = await account.sendMoney(
                holderAccount.owner_id, // receiver account
                idealShareString // amount in yoctoNEAR
            );
        } catch (err) {
            console.log(err);
            console.log(receiptList);
        }

        objectTransaction = {
            owner_id: holderAccount.owner_id,
            nft_id: holderAccount.token_id,
            transfer_value: idealShareString,
            receipt: receipt.transaction.hash
        }
        receiptList.push(objectTransaction);
    }

    console.log("retrieval successfull");
    console.log(JSON.stringify(receiptList));
}

export { initializeContract, emergencyPanic, getContractState, updateContract, retrieveDevFunds, retrieveNftFunds };