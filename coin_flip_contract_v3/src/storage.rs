use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

#[derive(Clone)]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[derive(Clone)]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

#[near_bindgen]
impl SlotMachine {
    // if `registration_only=true` MUST refund above the minimum balance if the account didn't exist and
    //     refund full deposit if the account exists.
    #[payable]
    pub fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let account = account_id.unwrap_or(env::predecessor_account_id());
        let registration_bool = registration_only.unwrap_or(false);
        let minimum_balance = self.storage_balance_bounds.min.0;
        let deposit = env::attached_deposit();

        match self.storage_balances.get(&account) {
            Some(i) => {
                if registration_bool {
                    Promise::new( env::predecessor_account_id() ).transfer(deposit);
                    i
                } else {
                    let mut struct_clone = i.clone();
                    struct_clone.available = U128(struct_clone.available.0 + deposit);
                    struct_clone.available = U128(struct_clone.total.0 + deposit);
                    self.storage_balances.insert(&account, &struct_clone);
                    struct_clone
                }
            },
            None => {
                assert!(deposit >= minimum_balance, "minimum deposit is {}", minimum_balance);
                if registration_bool {
                    let surplus = deposit - minimum_balance;
                    let balance_struct = StorageBalance {
                        total: U128(minimum_balance),
                        available: U128(minimum_balance)
                    };
                    self.storage_balances.insert(&account, &balance_struct);
                    Promise::new( env::predecessor_account_id() ).transfer(surplus);
                    balance_struct
                } else {
                    let balance_struct = StorageBalance {
                        total: U128(deposit),
                        available: U128(deposit)
                    };
                    self.storage_balances.insert(&account, &balance_struct);
                    balance_struct
                }
            }
        }
    }

    /// Withdraw specified amount of available Ⓝ for predecessor account.
    ///
    /// This method is safe to call. It MUST NOT remove data.
    ///
    /// `amount` is sent as a string representing an unsigned 128-bit integer. If
    /// omitted, contract MUST refund full `available` balance. If `amount` exceeds
    /// predecessor account's available balance, contract MUST panic.
    ///
    /// If predecessor account not registered, contract MUST panic.
    ///
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted
    /// function-call access-key call (UX wallet security)
    ///
    /// Returns the StorageBalance structure showing updated balances.
    #[payable]
    pub fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let account = env::predecessor_account_id();
        let mut storage_balance = self.storage_balances.get(&account).expect("Accont not registered");

        if let Some(i) = amount {
            assert!(i.0 <= storage_balance.available.0, "Not enough balance");
            storage_balance.available = U128(storage_balance.available.0 - i.0);
            storage_balance.total = U128(storage_balance.total.0 - i.0);
            self.storage_balances.insert(&account, &storage_balance);
            Promise::new( account.clone() ).transfer(i.0);
            storage_balance
        } else {
            let transfer_amount = storage_balance.available.0;
            storage_balance.available = U128(0);
            storage_balance.total = U128(storage_balance.total.0 - transfer_amount);
            self.storage_balances.insert(&account, &storage_balance);
            Promise::new( account.clone() ).transfer(transfer_amount);
            storage_balance
        }

    }

    /// Unregisters the predecessor account and returns the storage NEAR deposit back.
    ///
    /// If the predecessor account is not registered, the function MUST return `false` without panic.
    ///
    /// If `force=true` the function SHOULD ignore account balances (burn them) and close the account.
    /// Otherwise, MUST panic if caller has a positive registered balance (eg token holdings) or
    ///     the contract doesn't support force unregistration.
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted function-call access-key call
    /// (UX wallet security)
    /// Returns `true` iff the account was unregistered.
    /// Returns `false` iff account was not registered before.

    pub fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        let account_id = env::predecessor_account_id();
        match self.storage_balances.get(&account_id) {
            Some(i) => panic!("Action not supported"),
            None => false
        }

    }

    pub fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.storage_balance_bounds.clone()
    }

    pub fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.storage_balances.get(&account_id)
    }
}

impl SlotMachine {

    pub fn charge_storage_cost(&mut self, initial_storage: u64, final_storage: u64, account: AccountId) {
        let mut user_struct = self.storage_balances.get(&account).unwrap();
        let user_balance = user_struct.available.0;
        if initial_storage > final_storage {
            let released_cost = (initial_storage as u128 - final_storage as u128) * env::storage_byte_cost();
            user_struct.available = U128(user_balance + released_cost);
            self.storage_balances.insert(&account, &user_struct);
        } else {
            let new_cost = (final_storage as u128 - initial_storage as u128) * env::storage_byte_cost();
            assert!(user_balance >= new_cost, "Insufficient storage balance, needs total {} yoctoNear, you currently have {}", new_cost, user_balance);
            user_struct.available = U128(user_balance + new_cost);
            self.storage_balances.insert(&account, &user_struct);
        }
    }

}