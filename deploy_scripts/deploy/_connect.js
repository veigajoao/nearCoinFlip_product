import * as nearAPI from "near-api-js";
import os from "os";
import path from "path";

async function connectNear() {
    const { keyStores } = nearAPI;
    const homedir = os.homedir();
    const CREDENTIALS_DIR = ".near-credentials";
    const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
    const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

    const { connect } = nearAPI;

    const config = {
        networkId: "mainnet",
        keyStore, // optional if not signing transactions
        nodeUrl: "https://rpc.mainnet.near.org",
        walletUrl: "https://wallet.mainnet.near.org",
        helperUrl: "https://helper.mainnet.near.org",
        explorerUrl: "https://explorer.mainnet.near.org",
    };
    const near = await connect(config);
    return near;
}

export default connectNear;