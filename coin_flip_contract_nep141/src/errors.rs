// contract errors
pub const ERR_001: &str = "ERR_001: Account is not registered";
pub const ERR_002: &str = "ERR_002: No partner registered for this address";
pub const ERR_003: &str = "ERR_003: Partner already registered for this address";
pub const ERR_004: &str = "ERR_004: Only partner game owner can call this method";
pub const ERR_005: &str = "ERR_005: ft_on_transfer msg parameter could not be parsed";


// storage errors
pub const ERR_101: &str = "ERR_101: Insufficient storage deposit";
pub const ERR_102: &str = "ERR_102: Must attach at leas the minimum deposit value";
pub const ERR_103: &str = "ERR_103: Cannot unregister storage while user still has token balances to withdraw";

// owner actions errors
pub const ERR_201: &str = "ERR_201: No owner funds to withdraw";
pub const ERR_202: &str = "ERR_202: No NFT funds to withdraw";

// partnered game errors
pub const ERR_301: &str = "ERR_301: Token sent is not the registered token type for game";