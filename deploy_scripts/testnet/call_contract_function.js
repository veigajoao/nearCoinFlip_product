import nearAPI from "near-api-js";
import loginNear from "./_login.js";

import buildContractObject from "./_contract_object.js";
import { BN } from "bn.js";

async function initializeContract(ownerAccount, contractAccount, params) {
    const contract = await buildContractObject(ownerAccount, contractAccount);
    console.log(params);
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
    console.log(namedArgs);

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
    console.log(tokenCount);

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
        // fetchResult = await account.viewFunction(nftContractAccount, "nft_tokens", { from_index: currentSize.toString(), limit: pagination })
        // nftList.push(...fetchResult);
        currentSize = nftList.length;
    }
    console.log(nftList);

    //get current state
    let contractState = await contract.get_contract_state({},
        "300000000000000",
    );
    let nftBalance = contractState.nft_balance
    console.log(nftBalance);


    //retrieve funds
    await contract.retrieve_nft_funds({
            distribution_list: [account.accountId]
        },
        "300000000000000",
        "1"
    );

    //get value per account
    let bnNftCount = new BN(tokenCount);
    let bnNftBalance = new BN(nftBalance);
    let idealShare = bnNftBalance.div(bnNftCount);
    let idealShareString = idealShare.toString(10);

    let receipt;
    let receiptList = [];
    let objectTransaction;

    for (let holderAccount of nftList) {
        console.log(`${holderAccount.token_id}/${tokenCount}`);
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

async function deposit(ownerAccount, contractAccount) {
    const contract = await buildContractObject(ownerAccount, contractAccount);
    const { near, account } = await loginNear(ownerAccount);

    await contract.deposit({},
        "300000000000000",
        nearAPI.utils.format.parseNearAmount("50")
    );
}

async function play(ownerAccount, contractAccount, choice, value) {
    const contract = await buildContractObject(ownerAccount, contractAccount);
    const { near, account } = await loginNear(ownerAccount);

    const result = await contract.play({
            _bet_type: choice,
            bet_size: nearAPI.utils.format.parseNearAmount(value)
        },
        "300000000000000",
        "0"
    );
}

async function initializeNft(nftContract, sender) {
    const contract = await buildContractObject(sender, nftContract);
    const { near, account } = await loginNear(sender);

    const result = await contract.new_default_meta({
            owner_id: sender
        },
        "300000000000000",
        "0"
    );
}

async function mintNft(nftContract, sender, receiver, id) {
    const contract = await buildContractObject(sender, nftContract);
    const { near, account } = await loginNear(sender);

    const result = await contract.nft_mint({
            token_id: id,
            receiver_id: receiver,
            token_metadata: {}
        },
        "300000000000000",
        "55100000000000000000000"
    );
}

export { initializeContract, emergencyPanic, getContractState, updateContract, retrieveDevFunds, retrieveNftFunds, deposit, play, mintNft, initializeNft };