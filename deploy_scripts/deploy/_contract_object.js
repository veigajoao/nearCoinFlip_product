import nearAPI from "near-api-js";

import loginNear from "./_login.js";

async function buildContractObject(ownerAccount, contractAccount) {
    const { near, account } = await loginNear(ownerAccount);

    const contract = new nearAPI.Contract(
        account, // the account object that is connecting
        contractAccount, {
            viewMethods: [],
            changeMethods: [
                "new", "retrieve_dev_funds", "retrieve_nft_funds",
                "update_contract", "emergency_panic", "get_contract_state"
            ],
            sender: account, // account object to initialize and sign transactions.
        }
    );
    return contract
}

export default buildContractObject;