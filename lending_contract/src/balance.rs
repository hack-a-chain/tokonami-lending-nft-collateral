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
}