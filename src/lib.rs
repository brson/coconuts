#![allow(unused)]

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, wee_alloc, AccountId, Balance, Promise, StorageUsage};
use near_sdk::BlockHeight;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Coconuts {
    citizens: LookupMap<AccountId, Citizen>,
}

impl Default for Coconuts {
    fn default() -> Coconuts {
        Coconuts {
            citizens: LookupMap::new(Vec::from(b"citizens".as_ref())),
        }
    }
}

#[near_bindgen]
impl Coconuts {
    pub fn signer_create_citizen(&mut self) {
        let account_id = env::signer_account_id();
        if self.is_citizen(&account_id) {
            env::panic(b"Account already exists");
        }
        let new_account = Citizen::default();
        self.citizens.insert(&account_id, &new_account);
    }
}

#[near_bindgen]
impl Coconuts {
    pub fn is_citizen(&self, account_id: &AccountId) -> bool {
        self.citizens.contains_key(account_id)
    }

    pub fn init_block_index(&self, account_id: &AccountId) -> U64 {
        U64(self.citizen(account_id).init_block_index)
    }

    pub fn init_coconut_balance(&self, account_id: &AccountId) -> U128 {
        U128(self.citizen(account_id).init_coconut_balance)
    }

    pub fn coconut_balance(&self, account_id: &AccountId) -> U128 {
        U128(self.citizen(account_id).coconut_balance())
    }
}

impl Coconuts {
    fn citizen(&self, account_id: &AccountId) -> Citizen {
        if let Some(account) = self.citizens.get(account_id) {
            account
        } else {
            env::panic(b"Account does not exist")
        }
    }
}


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Citizen {
    init_block_index: BlockHeight,
    init_coconut_balance: Balance,
}

impl Default for Citizen {
    fn default() -> Citizen {
        Citizen {
            init_block_index: env::block_index(),
            init_coconut_balance: 0,
        }
    }
}

impl Citizen {
    fn coconut_balance(&self) -> Balance {
        let block_index = env::block_index();
        assert!(block_index >= self.init_block_index);
        let diff_block_index = block_index - self.init_block_index;
        let diff_block_index = u128::from(diff_block_index);
        self.init_coconut_balance + diff_block_index
    }
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    static SIGNER_NAME: &'static str = "bob_near";

    fn signer_name() -> String {
        SIGNER_NAME.to_string()
    }

    fn get_context(input: Vec<u8>, is_view: bool, block_index: u64) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: signer_name(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn no_citizen() {
        let context = get_context(vec![], false, 0);
        testing_env!(context);
        let contract = Coconuts::default();
        assert!(!contract.is_citizen(&signer_name()));
    }

    #[test]
    fn create_citizen() {
        let context = get_context(vec![], false, 0);
        testing_env!(context);
        let mut contract = Coconuts::default();
        contract.signer_create_citizen();
        assert!(contract.is_citizen(&signer_name()));
    }

    #[test]
    fn coconut_balance() {
        let context = get_context(vec![], false, 0);
        testing_env!(context);
        let mut contract = Coconuts::default();
        contract.signer_create_citizen();

        assert_eq!(contract.coconut_balance(&signer_name()).0, 0);

        let context = get_context(vec![], false, 1);
        testing_env!(context);

        assert_eq!(contract.coconut_balance(&signer_name()).0, 1);
    }

}
