use crate::*;
use serde_json::Value;

impl LendingNftCollateral {

  pub fn increase_balance(&mut self, value: U128) {
    let current_value = self.get_balance_value(env::predecessor_account_id());
    self.balances.insert(&env::predecessor_account_id(), &(current_value.0 + value.0));
  }
}

#[near_bindgen]
impl LendingNftCollateral {

  pub fn get_balance_value(&self, owner_id: AccountId) -> U128 {
    U128(self.balances.get(&owner_id).unwrap_or(0))
  }

  #[payable]
  pub fn deposit_balance(&mut self) {
    self.increase_balance(U128(env::attached_deposit()));
  }

  pub fn reduce_and_withdraw_balance(&mut self, value_to_reduce: U128) {
    self.reduce_balance(value_to_reduce);
    Promise::new(env::predecessor_account_id()).transfer(value_to_reduce.0);
  }

  pub fn reduce_balance(&mut self, value_to_reduce: U128) {
    let current_value = self.get_balance_value(env::predecessor_account_id());
    assert!(value_to_reduce.0 <= current_value.0, "You don't have enough credit to reduce");
    self.balances.insert(&env::predecessor_account_id(), &(current_value.0 - value_to_reduce.0));
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
    assert_eq!(result, U128(10));
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

    contract.deposit_balance();
    let result = contract.balances.get(&accounts(0).to_string()).unwrap_or(0);
    assert_eq!(result, MINT_STORAGE_COST);
  }

  #[test]
  fn test_reduce_balance() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    contract.balances.insert(&accounts(0).into(), &(50));
    contract.reduce_balance(U128(20));
    let result = contract.balances.get(&accounts(0).to_string()).unwrap_or(0);
    assert_eq!(result, 30);
  }
}