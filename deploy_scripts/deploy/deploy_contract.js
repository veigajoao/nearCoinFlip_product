import { KeyPair, utils } from "near-api-js";
import { randomBytes } from "crypto";
import fs from "fs";

import loginNear from "./_login.js";


async function deployContractAccountExists(accountToDeploy, binaryLocation) {
    const { near, account } = await loginNear(accountToDeploy);
    const response = await account.deployContract(fs.readFileSync(binaryLocation));
    console.log(response);
}

async function deployContractNewAccount(mainAccount, accountToDeploy, binaryLocation, nearToSend) {
    const { near, account } = loginNear(mainAccount);
    const keyPair = KeyPair.fromRandom(randomBytes(256));
    const publicKey = keyPair.publicKey.toString();
    const deployAccount = await account.createAccount(
        accountToDeploy, // new account name
        publicKey, // public key for new account
        utils.parseNearAmount(nearToSend) // initial balance for new account in yoctoNEAR
    );
    const response = await deployAccount.deployContract(fs.readFileSync(binaryLocation));
    console.log(response);
}

export { deployContractAccountExists, deployContractNewAccount };