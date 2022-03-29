import connectNear from "./_connect.js";

async function loginNear(accountP) {
    const near = await connectNear();
    const account = await near.account(accountP);
    return {
        near,
        account
    }
}

export default loginNear;