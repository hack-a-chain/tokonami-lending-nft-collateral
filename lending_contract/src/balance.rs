use crate::*;
use serde_json::Value;

#[near_bindgen]
impl LendingNftCollateral {

  pub fn get_balance_value(&mut self, owner_id: AccountId) -> u128 {
    self.balances.get(&owner_id).unwrap_or(0)
  }

  #[payable]
  pub fn deposit_balance(&mut self, value_to_deposit: U128) {
    let current_value = self.get_balance_value(env::predecessor_account_id());
    self.balances.insert(&env::predecessor_account_id(), &(current_value + value_to_deposit.0));
  }

  pub fn remove_balance(&mut self, value_to_remove: U128) {
    let current_value = self.get_balance_value(env::predecessor_account_id());
    assert!(value_to_remove.0 <= current_value, "You don't have enough credit to remove");
    self.balances.insert(&env::predecessor_account_id(), &(current_value - value_to_remove.0));
    Promise::new(env::predecessor_account_id()).transfer(value_to_remove.0);
  }

  let initial_storage = env::storage_usage();

  let final_storage = env::storage_usage();

  pub fn measure_storage(account_id: AccountId, initial_storage, final_storage) {

    if final_storage > initial_storage {
      let value_to_remove = (final_storage - initial_storage) * env::storage_byte_cost;
      let current_value = self.get_balance_value(account_id);
      assert!(value_to_remove.0 <= current_value, "You don't have enough credit to remove");
      self.balances.insert(&env::predecessor_account_id(), &(current_value - value_to_remove.0));
    } else {
      let current_value = self.get_balance_value(account_id);
      self.balances.insert(&account_id, &( current_value + (initial_storage - final_storage) * env::storage_byte_cost));
    }

  }

}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
  use near_sdk::test_utils::{accounts, VMContextBuilder};
  use near_sdk::testing_env;
  use near_sdk::MockedBlockchain;

  use super::*;

  const MINT_STORAGE_COST: u128 = 5920000000000000000000;

  fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(accounts(0))
        .signer_account_id(predecessor_account_id.clone())
        .predecessor_account_id(predecessor_account_id);
    builder
  }

  #[test]
  fn test_get_balance_value() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    contract.balances.insert(&accounts(1).into(), &(10));
    let result = contract.get_balance_value(accounts(1).into());
    assert_eq!(result, 10);
  }

  #[test]
  fn test_deposit_balance() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    contract.deposit_balance(U128(20));
    let result = contract.balances.get(&accounts(0).to_string()).unwrap_or(0);
    assert_eq!(result, 20);
  }

  #[test]
  fn test_remove_balance() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    contract.balances.insert(&accounts(0).into(), &(50));
    contract.remove_balance(U128(20));
    let result = contract.balances.get(&accounts(0).to_string()).unwrap_or(0);
    assert_eq!(result, 30);
  }
}