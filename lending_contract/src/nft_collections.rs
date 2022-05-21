use crate::*;
use serde_json::Value;

#[near_bindgen]
impl LendingNftCollateral {

  pub fn get_nft_collection_interest(&mut self, nft_collection_id: AccountId) -> u128 {
    self.nft_collections.get(&nft_collection_id).unwrap_or(0)
  }

  pub fn set_nft_collection_interest(&mut self, nft_collection_id: AccountId, interest: U128) {
    self.nft_collections.insert(&nft_collection_id, &interest.0);
  }

  pub fn insert_new_nft_collection(&mut self, nft_collection_id: AccountId, interest: U128) {
    self.set_nft_collection_interest(nft_collection_id.clone(), interest);

    let mut lending_vector_id = nft_collection_id.clone();
    lending_vector_id.push_str("lending");
    let new_lending_vec = Vector::new(lending_vector_id.into_bytes().to_vec());
    self.lending_offers_vecs.insert(&nft_collection_id, &new_lending_vec);

    let mut borrowing_vector_id = nft_collection_id.clone();
    borrowing_vector_id.push_str("lending");
    let new_borrowing_vec = Vector::new(borrowing_vector_id.into_bytes().to_vec());
    self.borrowing_offers_vecs.insert(&nft_collection_id, &new_borrowing_vec);

    let lending_offer_map = LookupMap::new(b"lending_offer".to_vec());
    self.lending_offers.insert(&nft_collection_id, &lending_offer_map);

    let borrowing_offer_map = LookupMap::new(b"borrowing_offer".to_vec());
    self.borrowing_offers.insert(&nft_collection_id, &borrowing_offer_map);
  }

  // pub fn remove_nft_collection(&mut self, nft_collection_id: AccountId) {
  //   self.nft_collections.remove(&nft_collection_id);
  // }
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
  fn test_get_nft_collection_interest() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    contract.nft_collections.insert(&accounts(1).into(), &(10));
    let result = contract.get_nft_collection_interest(accounts(1).into());
    assert_eq!(result, 10);
  }

  #[test]
  fn test_set_nft_collection_interest() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    contract.set_nft_collection_interest(accounts(1).to_string(), U128(20));
    let result = contract.nft_collections.get(&accounts(1).to_string()).unwrap_or(0);
    assert_eq!(result, 20);
  }

  #[test]
  fn test_insert_new_nft_collection() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    contract.insert_new_nft_collection(accounts(1).into(), U128(10));
    let result = contract.nft_collections.get(&accounts(1).to_string()).unwrap_or(0);
    assert_eq!(result, 10);
  }

  // #[test]
  // fn test_remove_nft_collection() {
  //   let mut context = get_context(accounts(1));
  //   testing_env!(context.build());
  //   let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

  //   testing_env!(context
  //     .storage_usage(env::storage_usage())
  //     .attached_deposit(MINT_STORAGE_COST)
  //     .predecessor_account_id(accounts(0))
  //     .build());

  //   contract.remove_nft_collection(accounts(1).to_string());
  //   let result = contract.nft_collections.get(&accounts(1).to_string()).unwrap_or(0);
  //   assert_eq!(result, 0);
  // }
}