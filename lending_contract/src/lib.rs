use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::env;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::init;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
// use serde_json::Value;
use near_sdk::callback;
use near_sdk::ext_contract;
use near_sdk::serde_json::{self, Value};
use near_sdk::{Balance, Gas, Promise, PromiseOrValue};

// use crate::lending_contract_interface::NftLending;

pub type NftCollection = AccountId;
const NO_DEPOSIT: Balance = 0;
const BASE_GAS: Gas = 5_000_000_000_000;

mod lending_contract_interface;
pub mod nft_on_impl;
pub mod loan;
pub mod balance;
pub mod controller;
pub mod nft_collections;

#[ext_contract(ext_nft_contract)]
trait NftContract {
    fn nft_token(&self, 
      token_id: TokenId) -> Token;

    fn nft_mint(&self, 
      token_id: TokenId, 
      receiver_id: AccountId, 
      token_metadata: TokenMetadata) -> Token;

    fn nft_burn(&self, 
      token_id: TokenId) -> bool;

    fn nft_transfer(&self,
      receiver_id: String,
      token_id: String,
      approval_id: Option<u128>,
      memo: Option<String>
    );
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Offer {
  pub offer_id: String,
  pub owner_id: AccountId,
  pub value: u128,
  pub token_id: Option<TokenId>
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LendingNftCollateral {
  //define later
  pub lending_offers_quantity_limit: u64,
  pub borrowing_offers_quantity_limit: u64,
  pub owner_id: AccountId,
  pub lending_offers: LookupMap<NftCollection, LookupMap<String, Offer>>,
  pub borrowing_offers: LookupMap<NftCollection, LookupMap<String, Offer>>,
  // change this later
  pub current_lending_offer_id: LookupMap<NftCollection, u128>,
  pub current_borrowing_offer_id: LookupMap<NftCollection, u128>,

  //ordered offers
  //lower(0) to higher(len-1)
  pub lending_offers_vecs: LookupMap<NftCollection, Vector<Offer>>,
  //higher(0)  to lower(len-1)
  pub borrowing_offers_vecs: LookupMap<NftCollection, Vector<Offer>>,

  pub token_id_counter: u128,
  pub loans: LookupMap<TokenId, Loan>,
  pub loan_expiration_seconds_limit: u128,
  pub note_address: AccountId,
  pub receipt_address: AccountId,

  pub balances: LookupMap<AccountId, u128>,
  pub nft_collections: LookupMap<AccountId, u128>
}

impl Default for LendingNftCollateral {
  fn default() -> Self {
      panic!("Should be initialized before usage")
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Loan {
  pub value_before: u128,
  pub value_after: u128,
  pub expiration_time: u128,
  pub warranty_collection: AccountId,
  pub warranty_token_id: String
}

// impl NftLending for LendingNftCollateral{
#[near_bindgen]
impl LendingNftCollateral {

  #[init]
  fn new(owner_id: AccountId, note_address: AccountId, receipt_address: AccountId) -> Self {
    Self {
      token_id_counter: 0,
      // mudar depois: colocar validação no código
      lending_offers_quantity_limit: 20,
      borrowing_offers_quantity_limit: 20,
      owner_id: owner_id,
      borrowing_offers: LookupMap::new(b"borrowing_offers".to_vec()),
      lending_offers: LookupMap::new(b"lending_offers".to_vec()),
      current_lending_offer_id: LookupMap::new(b"current_lending_offer_id".to_vec()),
      current_borrowing_offer_id: LookupMap::new(b"current_lending_offer_id".to_vec()),
      lending_offers_vecs: LookupMap::new(b"lending_offers_vecs".to_vec()),
      borrowing_offers_vecs: LookupMap::new(b"borrowing_offers_vecs".to_vec()),
      loans: LookupMap::new(b"loans".to_vec()),
      // 15 days in seconds
      loan_expiration_seconds_limit: 1296000,
      note_address: note_address,
      receipt_address: receipt_address,
      balances: LookupMap::new(b"balances".to_vec()),
      nft_collections: LookupMap::new(b"nft_collections".to_vec()),
    }
  }

  fn get_best_lending_offer(&mut self, nft_collection_id: NftCollection) -> Option<Offer> 
  {
    let lending_offer_vec = self.get_lending_offers_vec_from_nft_collection(nft_collection_id.to_string());
    let best_offer_index = if lending_offer_vec.len() == 0 {0} else {lending_offer_vec.len() - 1};
    lending_offer_vec.get(best_offer_index)
  }

  fn get_best_borrowing_offer(&mut self, nft_collection_id: NftCollection) -> Option<Offer> {
    let borrowing_offer_vec = self.get_borrowing_offers_vec_from_nft_collection(nft_collection_id.to_string());
    let best_offer_index = if borrowing_offer_vec.len() == 0 {0} else {borrowing_offer_vec.len() - 1};
    borrowing_offer_vec.get(best_offer_index)
  }

  fn cancel_specific_lending_offer(&mut self, offer_id: String, nft_collection_id: NftCollection) {
    let initial_storage = U128(env::storage_usage() as u128);

    let nft_collection_lending_offers = self.lending_offers.get(&nft_collection_id);
    let mut nft_collection_lending_offer_vec = self.lending_offers_vecs.get(&nft_collection_id).unwrap();
    let specific_lending_offer = nft_collection_lending_offers.unwrap().get(&offer_id).unwrap();
    assert!(env::predecessor_account_id() == specific_lending_offer.owner_id, "You are not the owner of this offer");
    // reorder and remove from vecs
    self.reorder_vec_without_specific_offer(&mut nft_collection_lending_offer_vec, specific_lending_offer.clone());
    self.lending_offers.get(&nft_collection_id.clone()).unwrap().remove(&offer_id);

    let final_storage = U128(env::storage_usage() as u128);
    self.set_storage(initial_storage, final_storage);
  }

  fn cancel_specific_borrowing_offer(&mut self, offer_id: String, nft_collection_id: NftCollection) {
    let initial_storage = U128(env::storage_usage() as u128);

    let nft_collection_borrowing_offers = self.borrowing_offers.get(&nft_collection_id);
    let mut nft_collection_borrowing_offer_vec = self.borrowing_offers_vecs.get(&nft_collection_id).unwrap();
    let specific_borrowing_offer = nft_collection_borrowing_offers.unwrap().get(&offer_id).unwrap();
    assert!(env::predecessor_account_id() == specific_borrowing_offer.owner_id, "You are not the owner of this offer");
    // REORDER AND REMOVE FROM VECS
    self.reorder_vec_without_specific_offer(&mut nft_collection_borrowing_offer_vec, specific_borrowing_offer.clone());
    self.borrowing_offers.get(&nft_collection_id.clone()).unwrap().remove(&offer_id);
      
    //transfer nft back
    ext_nft_contract::nft_transfer(
      env::current_account_id(), 
      specific_borrowing_offer.token_id.unwrap(),
      None,
      None,
      &specific_borrowing_offer.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );

    let final_storage = U128(env::storage_usage() as u128);
    self.set_storage(initial_storage, final_storage);
  }

  fn choose_specific_lending_offer(&mut self, nft_collection_id: NftCollection, offer_id: String, token_id: TokenId) -> bool {
    let initial_storage = U128(env::storage_usage() as u128);

    let nft_collection_lending_offers = self.lending_offers.get(&nft_collection_id);
    let mut nft_collection_lending_offer_vec = self.lending_offers_vecs.get(&nft_collection_id).unwrap();
    let specific_lending_offer = nft_collection_lending_offers.unwrap().get(&offer_id).unwrap();
    self.post_loan(specific_lending_offer.clone().owner_id, env::predecessor_account_id(), nft_collection_id.clone(), token_id, U128(specific_lending_offer.clone().value));
    // REORDER AND REMOVE FROM VECS
    self.reorder_vec_without_specific_offer(&mut nft_collection_lending_offer_vec, specific_lending_offer.clone());
    self.lending_offers.get(&nft_collection_id.clone()).unwrap().remove(&offer_id);

    let final_storage = U128(env::storage_usage() as u128);
    self.set_storage(initial_storage, final_storage);
    true
  }

  fn choose_specific_borrowing_offer(&mut self, nft_collection_id: NftCollection, offer_id: String) -> bool {
    let initial_storage = U128(env::storage_usage() as u128);

    let nft_collection_borrowing_offers = self.borrowing_offers.get(&nft_collection_id);
    let mut nft_collection_borrowing_offer_vec = self.borrowing_offers_vecs.get(&nft_collection_id).unwrap();
    let specific_borrowing_offer = nft_collection_borrowing_offers.unwrap().get(&offer_id).unwrap();
    self.post_loan(env::predecessor_account_id(), specific_borrowing_offer.clone().owner_id, nft_collection_id.clone(), specific_borrowing_offer.clone().token_id.unwrap(), U128(specific_borrowing_offer.clone().value));
    // REORDER AND REMOVE FROM VECS
    self.reorder_vec_without_specific_offer(&mut nft_collection_borrowing_offer_vec, specific_borrowing_offer.clone());
    self.borrowing_offers.get(&nft_collection_id.clone()).unwrap().remove(&offer_id);

    let final_storage = U128(env::storage_usage() as u128);
    self.set_storage(initial_storage, final_storage);

    true
  }

  #[payable]
  fn post_lending_offer(&mut self, nft_collection_id: AccountId, value_offered: U128) -> bool {
    let initial_storage = U128(env::storage_usage() as u128);

    let mut lending_offers_vec = self.get_lending_offers_vec_from_nft_collection(nft_collection_id.clone());
    assert!(lending_offers_vec.len() < self.lending_offers_quantity_limit, "There are too many offers already");

    if self.evaluate_lending_offer_possible_match(&nft_collection_id, value_offered) {
      let best_borrowing_offer = self.get_best_borrowing_offer(nft_collection_id.clone()).unwrap();
      self.post_loan(env::predecessor_account_id(), best_borrowing_offer.owner_id, nft_collection_id.clone(), best_borrowing_offer.token_id.unwrap(), value_offered);
      self.borrowing_offers_vecs.get(&nft_collection_id.clone()).unwrap().pop();

      let final_storage = U128(env::storage_usage() as u128);
      self.set_storage(initial_storage, final_storage);

      false
    }
    else {
      let offer_id = self.current_lending_offer_id.get(&nft_collection_id).unwrap_or(0);
      let offer = Offer{offer_id: offer_id.to_string(), owner_id: env::predecessor_account_id(), value: value_offered.0, token_id: None};
      let ordered_lending_offer_vec = self.sort_order_lending_offer_vec(lending_offers_vec, offer.clone());
      self.lending_offers_vecs.insert(&nft_collection_id.clone(), &ordered_lending_offer_vec);
      let mut offer_map = self.lending_offers.get(&nft_collection_id.clone()).unwrap();
      offer_map.insert(&offer_id.to_string(), &offer);
      self.lending_offers.insert(&nft_collection_id.clone(), &offer_map);
      self.current_lending_offer_id.insert(&nft_collection_id.clone(), &(offer_id + 1));

      let final_storage = U128(env::storage_usage() as u128);
      self.set_storage(initial_storage, final_storage);

      true
    }
  }

  #[payable]
  fn post_borrowing_offer(&mut self, nft_collection_id: NftCollection, value_offered: U128, collateral_nft: TokenId, nft_owner_id: AccountId) -> bool {
    let initial_storage = U128(env::storage_usage() as u128);

    let mut borrowing_offers_vec = self.get_borrowing_offers_vec_from_nft_collection(nft_collection_id.clone());
    assert!(borrowing_offers_vec.len() < self.borrowing_offers_quantity_limit, "There are too many offers already");

    //check if there is a match
    if self.evaluate_borrowing_offer_possible_match(&nft_collection_id, value_offered) {
      let best_lending_offer = self.get_best_lending_offer(nft_collection_id.clone()).unwrap();
      self.post_loan(best_lending_offer.owner_id, nft_owner_id, nft_collection_id.clone(), collateral_nft, value_offered);
      self.lending_offers_vecs.get(&nft_collection_id.clone()).unwrap().pop();

      let final_storage = U128(env::storage_usage() as u128);
      self.set_storage(initial_storage, final_storage);

      false
    }
    else {
      let offer_id = self.current_borrowing_offer_id.get(&nft_collection_id).unwrap_or(0);
      let offer = Offer{offer_id: offer_id.to_string(), owner_id: nft_owner_id, value: value_offered.0, token_id: Some(collateral_nft)};
      let ordered_borrowing_offer_vec = self.sort_order_lending_offer_vec(borrowing_offers_vec, offer.clone());
      self.borrowing_offers_vecs.insert(&nft_collection_id.clone(), &ordered_borrowing_offer_vec);
      let mut offer_map = self.lending_offers.get(&nft_collection_id.clone()).unwrap();
      offer_map.insert(&offer_id.to_string(), &offer);
      self.borrowing_offers.insert(&nft_collection_id.clone(), &offer_map);
      self.current_borrowing_offer_id.insert(&nft_collection_id.clone(), &(offer_id + 1));

      let final_storage = U128(env::storage_usage() as u128);
      self.set_storage(initial_storage, final_storage);

      true
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
  fn test_new() {
      let mut context = get_context(accounts(1));
      testing_env!(context.build());
      let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());
      testing_env!(context.is_view(true).build());
      assert_eq!(contract.owner_id, accounts(1).to_string());
  }

  #[test]
  fn test_get_best_lending_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let mut vector_id = nft_collection_id.clone();
    vector_id.push_str("lending");
    let mut new_vec = Vector::new(vector_id.into_bytes().to_vec());
    let lending_offer1 = Offer{offer_id: "offer_id_test1".to_string(), owner_id: accounts(1).into(), value: 10, token_id: None};
    let lending_offer2 = Offer{offer_id: "offer_id_test2".to_string(), owner_id: accounts(1).into(), value: 20, token_id: None};
    new_vec.push(&lending_offer1);
    new_vec.push(&lending_offer2);
    contract.lending_offers_vecs.insert(&nft_collection_id, &new_vec);
    let best_offer = contract.get_best_lending_offer(nft_collection_id.clone()).unwrap();
    assert_eq!(best_offer.value, 20);
    assert_eq!(best_offer.offer_id, "offer_id_test2".to_string());
  }

  #[test]
  fn test_get_best_borrowing_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let mut vector_id = nft_collection_id.clone();
    vector_id.push_str("borrowing");
    let mut new_vec = Vector::new(vector_id.into_bytes().to_vec());
    let borrowing_offer1 = Offer{offer_id: "offer_id_test1".to_string(), owner_id: accounts(1).into(), value: 20, token_id: Some("token_id_test1".to_string())};
    let borrowing_offer2 = Offer{offer_id: "offer_id_test2".to_string(), owner_id: accounts(1).into(), value: 10, token_id: Some("token_id_test2".to_string())};
    new_vec.push(&borrowing_offer1);
    new_vec.push(&borrowing_offer2);
    contract.borrowing_offers_vecs.insert(&nft_collection_id, &new_vec);
    let best_offer = contract.get_best_borrowing_offer(nft_collection_id.clone()).unwrap();
    assert_eq!(best_offer.value, 10);
    assert_eq!(best_offer.offer_id, "offer_id_test2".to_string());
    assert_eq!(best_offer.token_id.unwrap(), "token_id_test2".to_string());
  }

  #[test] 
  fn test_cancel_specific_lending_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let mut vector_id = nft_collection_id.clone();
    vector_id.push_str("lending");
    let mut new_vec = Vector::new(vector_id.into_bytes().to_vec());
    let lending_offer1 = Offer{offer_id: "offer_id_test1".to_string(), owner_id: accounts(0).into(), value: 10, token_id: None};
    let lending_offer2 = Offer{offer_id: "offer_id_test2".to_string(), owner_id: accounts(1).into(), value: 20, token_id: None};
    new_vec.push(&lending_offer1);
    new_vec.push(&lending_offer2);
    contract.lending_offers_vecs.insert(&nft_collection_id, &new_vec);
    let mut offer_map = LookupMap::new(b"lending_offer".to_vec());
    offer_map.insert(&("offer_id_test1".to_string()), &lending_offer1);
    offer_map.insert(&("offer_id_test2".to_string()), &lending_offer2);
    contract.lending_offers.insert(&nft_collection_id.clone(), &offer_map);

    contract.cancel_specific_lending_offer("offer_id_test1".to_string(), nft_collection_id.clone());
    let lending_offer_vec = contract.lending_offers_vecs.get(&nft_collection_id).unwrap();
    // ta certo isso? é pra ser 2 mesmo?
    assert_eq!(lending_offer_vec.len(), 2);
    assert_eq!(lending_offer_vec.get(0).unwrap().offer_id, "offer_id_test2".to_string());
  }

  #[test] 
  fn test_cancel_specific_borrowing_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let mut vector_id = nft_collection_id.clone();
    vector_id.push_str("borrowing");
    let mut new_vec = Vector::new(vector_id.into_bytes().to_vec());
    let borrowing_offer1 = Offer{offer_id: "offer_id_test1".to_string(), owner_id: accounts(0).into(), value: 20, token_id: Some("token_id1".to_string())};
    let borrowing_offer2 = Offer{offer_id: "offer_id_test2".to_string(), owner_id: accounts(1).into(), value: 10, token_id: Some("token_id2".to_string())};
    new_vec.push(&borrowing_offer1);
    new_vec.push(&borrowing_offer2);
    contract.borrowing_offers_vecs.insert(&nft_collection_id, &new_vec);
    let mut offer_map = LookupMap::new(b"borrowing_offer".to_vec());
    offer_map.insert(&("offer_id_test1".to_string()), &borrowing_offer1);
    offer_map.insert(&("offer_id_test2".to_string()), &borrowing_offer2);
    contract.borrowing_offers.insert(&nft_collection_id.clone(), &offer_map);

    contract.cancel_specific_borrowing_offer("offer_id_test1".to_string(), nft_collection_id.clone());
    let borrowing_offer_vec = contract.borrowing_offers_vecs.get(&nft_collection_id).unwrap();
    // ta certo isso? é pra ser 2 mesmo?
    assert_eq!(borrowing_offer_vec.len(), 2);
    assert_eq!(borrowing_offer_vec.get(0).unwrap().offer_id, "offer_id_test2".to_string());
  }

  #[test]
  fn test_choose_specific_lending_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let mut vector_id = nft_collection_id.clone();
    vector_id.push_str("lending");
    let mut new_vec = Vector::new(vector_id.into_bytes().to_vec());
    let lending_offer1 = Offer{offer_id: "offer_id_test1".to_string(), owner_id: accounts(0).into(), value: 10, token_id: None};
    let lending_offer2 = Offer{offer_id: "offer_id_test2".to_string(), owner_id: accounts(1).into(), value: 20, token_id: None};
    new_vec.push(&lending_offer1);
    new_vec.push(&lending_offer2);
    contract.lending_offers_vecs.insert(&nft_collection_id, &new_vec);
    let mut offer_map = LookupMap::new(b"lending_offer".to_vec());
    offer_map.insert(&("offer_id_test1".to_string()), &lending_offer1);
    offer_map.insert(&("offer_id_test2".to_string()), &lending_offer2);
    contract.lending_offers.insert(&nft_collection_id.clone(), &offer_map);

    let success = contract.choose_specific_lending_offer(nft_collection_id.clone(), "offer_id_test1".to_string(), "token_id1".to_string());
    assert_eq!(success, true);
    assert_eq!(contract.lending_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().offer_id, "offer_id_test2".to_string());

  }

  #[test]
  fn test_choose_specific_borrowing_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let mut vector_id = nft_collection_id.clone();
    vector_id.push_str("borrowing");
    let mut new_vec = Vector::new(vector_id.into_bytes().to_vec());
    let borrowing_offer1 = Offer{offer_id: "offer_id_test1".to_string(), owner_id: accounts(0).into(), value: 20, token_id: Some("token_id1".to_string())};
    let borrowing_offer2 = Offer{offer_id: "offer_id_test2".to_string(), owner_id: accounts(1).into(), value: 10, token_id: Some("token_id2".to_string())};
    new_vec.push(&borrowing_offer1);
    new_vec.push(&borrowing_offer2);
    contract.borrowing_offers_vecs.insert(&nft_collection_id, &new_vec);
    let mut offer_map = LookupMap::new(b"borrowing_offer".to_vec());
    offer_map.insert(&("offer_id_test1".to_string()), &borrowing_offer1);
    offer_map.insert(&("offer_id_test2".to_string()), &borrowing_offer2);
    contract.borrowing_offers.insert(&nft_collection_id.clone(), &offer_map);

    let success = contract.choose_specific_borrowing_offer(nft_collection_id.clone(), "offer_id_test1".to_string());
    assert_eq!(success, true);
    assert_eq!(contract.borrowing_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().offer_id, "offer_id_test2".to_string());
  }

  #[test]
  fn test_post_lending_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());
            
    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());
    contract.deposit_balance(U128(MINT_STORAGE_COST*10000));
    let nft_collection_id = "nft_collection_test".to_string();
    contract.insert_new_nft_collection(nft_collection_id.clone(), U128(1));
    let success = contract.post_lending_offer(nft_collection_id.clone(), U128(10));
    assert_eq!(success, true);
    assert_eq!(contract.lending_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().value, 10);
    let offer_id = contract.lending_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().offer_id;
    assert_eq!(contract.lending_offers.get(&nft_collection_id).unwrap().get(&offer_id).unwrap().value, 10);
    }

    #[test]
    fn test_post_borrowing_offer() {
      let mut context = get_context(accounts(1));
      testing_env!(context.build());
      let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());
            
      testing_env!(context
        .storage_usage(env::storage_usage())
        .attached_deposit(MINT_STORAGE_COST)
        .predecessor_account_id(accounts(0))
        .build());

      contract.deposit_balance(U128(MINT_STORAGE_COST*10000));
      let nft_collection_id = "nft_collection_test".to_string();
      contract.insert_new_nft_collection(nft_collection_id.clone(), U128(1));
      let success = contract.post_borrowing_offer(nft_collection_id.clone(), U128(10), "token_id".to_string(), accounts(0).into());
      assert_eq!(success, true);
      assert_eq!(contract.borrowing_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().value, 10);
      let offer_id = contract.borrowing_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().offer_id;
      assert_eq!(contract.borrowing_offers.get(&nft_collection_id).unwrap().get(&offer_id).unwrap().value, 10);
    }
}