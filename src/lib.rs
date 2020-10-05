#![allow(unused)]

use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, wee_alloc, AccountId, Balance, Promise, StorageUsage};
use near_sdk::BlockHeight;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub type CitizenId = u64;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Coconuts {
    accounts: LookupMap<AccountId, CitizenId>,
    citizens: LookupMap<CitizenId, Citizen>,
    next_citizen_id: u64,
}

impl Default for Coconuts {
    fn default() -> Coconuts {
        Coconuts {
            accounts: LookupMap::new(Vec::from(b"accounts".as_ref())),
            citizens: LookupMap::new(Vec::from(b"citizens".as_ref())),
            next_citizen_id: 0,
        }
    }
}

// Contract payable account management
#[near_bindgen]
impl Coconuts {
    pub fn signer_create_citizen(&mut self) {
        let account_id = env::signer_account_id();
        if self.is_citizen(&account_id) {
            env::panic(b"Account already exists");
        }
        let new_citizen_id = self.next_citizen_id;
        assert!(!self.citizens.contains_key(&new_citizen_id));
        let new_citizen = Citizen::default();
        self.citizens.insert(&new_citizen_id, &new_citizen);
        self.accounts.insert(&account_id, &new_citizen_id);
        assert!(self.next_citizen_id < u64::max_value());
        self.next_citizen_id += 1;
    }
}

// Contract view account management
#[near_bindgen]
impl Coconuts {
    pub fn is_citizen(&self, account_id: &AccountId) -> bool {
        if let Some(citizen_id) = self.accounts.get(account_id) {
            assert!(self.citizens.contains_key(&citizen_id));
            true
        } else {
            false
        }
    }
}

// Account management helpers
impl Coconuts {
    fn citizen(&self, account_id: &AccountId) -> Citizen {
        if let Some(citizen_id) = self.accounts.get(account_id) {
            if let Some(account) = self.citizens.get(&citizen_id) {
                account
            } else {
                env::panic(b"Account is not a citizen");
            }
        } else {
            env::panic(b"Account does not exist");
        }
    }

    fn set_citizen(&mut self, account_id: &AccountId, citizen: &Citizen) {
        if let Some(citizen_id) = self.accounts.get(account_id) {
            self.citizens.insert(&citizen_id, citizen);
        } else {
            env::panic(b"Account does not exist");
        }
    }
}

// Contract view citizen accessors
#[near_bindgen]
impl Coconuts {
    pub fn init_block_index(&self, account_id: &AccountId) -> U64 {
        U64(self.citizen(account_id).init_block_index)
    }

    pub fn young_coconut_balance(&self, account_id: &AccountId) -> U128 {
        U128(self.citizen(account_id).young_coconut_balance())
    }

    pub fn brown_coconut_balance(&self, account_id: &AccountId) -> U128 {
        U128(self.citizen(account_id).brown_coconut_balance())
    }

    pub fn citizen_state(&self, account_id: &AccountId) -> CitizenState {
        let citizen = self.citizen(account_id);
        let citizen_id = self.accounts.get(account_id).expect("citizen");
        let block_index = env::block_index();
        assert!(block_index >= citizen.init_block_index);
        CitizenState {
            account_id: account_id.clone(),
            citizen_id: U64(citizen_id),
            current_block_index: U64(block_index),
            init_block_index: U64(citizen.init_block_index),
            block_age: U64(block_index - citizen.init_block_index),
            young_coconut_balance: U128(citizen.young_coconut_balance()),
            brown_coconut_balance: U128(citizen.brown_coconut_balance()),
        }
    }
}

#[derive(Serialize)]
pub struct CitizenState {
    account_id: AccountId,
    citizen_id: U64,
    current_block_index: U64,
    init_block_index: U64,
    block_age: U64,
    young_coconut_balance: U128,
    brown_coconut_balance: U128,
}

// Contract payable asset transfers
#[near_bindgen]
impl Coconuts {
    pub fn signer_transfer_young_coconuts(&mut self, account_id_to: &AccountId, qty: U128) {
        let account_id_from = env::signer_account_id();
        self.transfer_young_coconuts(&account_id_from, account_id_to, qty.0)
    }
}

// Asset transfer helpers
impl Coconuts {
    fn transfer_young_coconuts(&mut self, account_id_from: &AccountId, account_id_to: &AccountId, qty: u128) {
        if !self.is_citizen(&account_id_from) {
            env::panic(b"Signer account is not a citizen");
        }
        if !self.is_citizen(&account_id_to) {
            env::panic(b"Destination account is not a citizen");
        }

        let mut citizen_from = self.citizen(&account_id_from);
        let mut citizen_to = self.citizen(&account_id_to);

        let balance_from = citizen_from.young_coconut_balance();
        let balance_to = citizen_to.young_coconut_balance();

        if balance_from.checked_sub(qty).is_none() {
            env::panic(b"Transfer quantity less than balance");
        }

        if balance_to.checked_add(qty).is_none()  {
            env::panic(b"Transfer overflows receiver");
        }

        citizen_from.young_coconut_adjustments.sent +=
            citizen_from.young_coconut_adjustments.sent.checked_add(qty).expect("overflow");
        citizen_to.young_coconut_adjustments.received +=
            citizen_to.young_coconut_adjustments.received.checked_add(qty).expect("overflow");

        self.set_citizen(&account_id_from, &citizen_from);
        self.set_citizen(&account_id_to, &citizen_to);
    }
}



#[derive(BorshDeserialize, BorshSerialize)]
pub struct Citizen {
    init_block_index: BlockHeight,
    coconut_tree_count: u128,
    young_coconut_adjustments: Adjustments,
}

impl Default for Citizen {
    fn default() -> Citizen {
        Citizen {
            init_block_index: env::block_index(),
            coconut_tree_count: 1,
            young_coconut_adjustments: Adjustments::default(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Default)]
pub struct Adjustments {
    pub sent: u128,
    pub received: u128,
}

const INITIAL_COCONUTS: u128 = 0;
/// Young coconuts generated each block, per tree
const COCONUTS_PER_BLOCK: u128 = 1;
/// Blocks until a young coconut becomes a brown coconut
const COCONUT_MATURATION_BLOCKS: u128 = 10;

impl Citizen {
    fn baseline_coconuts(&self) -> Balance {
        let block_index = env::block_index();
        assert!(block_index >= self.init_block_index);
        let diff_block_index = block_index - self.init_block_index;
        let diff_block_index = u128::from(diff_block_index);
        let coconuts_since_init = diff_block_index
            .checked_mul(self.coconut_tree_count).expect("overflow")
            .checked_mul(COCONUTS_PER_BLOCK).expect("overflow");
        INITIAL_COCONUTS.checked_add(coconuts_since_init).expect("overflow")
    }

    fn young_coconut_balance(&self) -> Balance {
        let baseline_coconuts = self.baseline_coconuts();
        assert!(self.brown_coconut_balance() <= baseline_coconuts);
        baseline_coconuts.checked_sub(self.brown_coconut_balance()).expect("overflow")
    }

    fn brown_coconut_balance(&self) -> Balance {
        let baseline_coconuts = self.baseline_coconuts();
        baseline_coconuts.saturating_sub(COCONUT_MATURATION_BLOCKS).checked_mul(COCONUTS_PER_BLOCK).expect("overflow")
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
    fn young_coconut_balance() {
        let context = get_context(vec![], false, 0);
        testing_env!(context);
        let mut contract = Coconuts::default();
        contract.signer_create_citizen();

        assert_eq!(contract.young_coconut_balance(&signer_name()).0, 0);

        let context = get_context(vec![], false, 1);
        testing_env!(context);

        assert_eq!(contract.young_coconut_balance(&signer_name()).0, 1);
    }

    #[test]
    fn coconut_balance_after_maturation() {
        let context = get_context(vec![], false, 0);
        testing_env!(context);
        let mut contract = Coconuts::default();
        contract.signer_create_citizen();

        assert_eq!(contract.young_coconut_balance(&signer_name()).0, 0);
        assert_eq!(contract.brown_coconut_balance(&signer_name()).0, 0);

        let context = get_context(vec![], false, 11);
        testing_env!(context);

        assert_eq!(contract.young_coconut_balance(&signer_name()).0, 10);
        assert_eq!(contract.brown_coconut_balance(&signer_name()).0, 1);

        let context = get_context(vec![], false, 20);
        testing_env!(context);

        assert_eq!(contract.young_coconut_balance(&signer_name()).0, 10);
        assert_eq!(contract.brown_coconut_balance(&signer_name()).0, 10);
    }
}
