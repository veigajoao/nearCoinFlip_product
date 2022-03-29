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
        networkId: "testnet",
        keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
    };
    const near = await connect(config);
    return near;
}

connectNear();
export default connectNear;