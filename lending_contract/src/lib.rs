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
use near_sdk::{Balance, Gas, Promise};
mod lending_contract_interface;
// use crate::lending_contract_interface::NftLending;

pub type NftCollection = AccountId;
const NO_DEPOSIT: Balance = 0;
const BASE_GAS: Gas = 5_000_000_000_000;

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn callback_promise_result() -> bool;
    fn callback_pay_loan(#[callback] token: Token, loan: Loan) -> bool;
    fn callback_transfer_warrancy(#[callback] token: Token, loan: Loan) -> bool;
}

#[ext_contract(ext_nft_contract)]
trait NftContract {
    fn nft_token(&self, 
      token_id: TokenId) -> Token;

    fn nft_mint(&self, 
      token_id: TokenId, 
      receiver_id: AccountId, 
      token_metadata: TokenMetadata) -> Token;

    fn nft_burn(&self, 
      token_id: TokenId, 
      owner_id: AccountId) -> bool;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Loan {
  pub value: u128,
  pub expiration_time: Option<u128>,
  pub warranty_collection: AccountId,
  pub warranty_token_id: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LendingNftCollateral {
  pub token_id_counter: u128,
  //define later
  pub offer_limit: u128,
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

  pub loans: LookupMap<TokenId, Loan>
}

impl Default for LendingNftCollateral {
  fn default() -> Self {
      panic!("Should be initialized before usage")
  }
}

// impl NftLending for LendingNftCollateral{
#[near_bindgen]
impl LendingNftCollateral {

  #[init]
  fn new(owner_id: AccountId) -> Self {
    Self {
      token_id_counter: 0,
      offer_limit: 20,
      owner_id: owner_id,
      borrowing_offers: LookupMap::new(b"borrowing_offers".to_vec()),
      lending_offers: LookupMap::new(b"lending_offers".to_vec()),
      current_lending_offer_id: LookupMap::new(b"current_lending_offer_id".to_vec()),
      current_borrowing_offer_id: LookupMap::new(b"current_lending_offer_id".to_vec()),
      lending_offers_vecs: LookupMap::new(b"lending_offers_vecs".to_vec()),
      borrowing_offers_vecs: LookupMap::new(b"borrowing_offers_vecs".to_vec()),
      loans: LookupMap::new(b"lending_offers_vecs".to_vec())
    }
  }

  fn get_best_lending_offer(&mut self, nft_collection_id: NftCollection) -> Offer 
  {
    let lending_offer_vec = self.get_lending_offers_vec_from_nft_collection(nft_collection_id.to_string());
    let best_offer_index = if lending_offer_vec.len() == 0 {lending_offer_vec.len()} else {lending_offer_vec.len() - 1};
    let best_lending_offer = lending_offer_vec.get(best_offer_index).unwrap_or(
      Offer{offer_id: "empty_offer".to_string(), owner_id: env::predecessor_account_id(), value: 0, token_id: None}
    );
    best_lending_offer
  }

  fn get_best_borrowing_offer(&mut self, nft_collection_id: NftCollection) -> Offer {
    let borrowing_offer_vec = self.get_borrowing_offers_vec_from_nft_collection(nft_collection_id.to_string());
    let best_offer_index = if borrowing_offer_vec.len() == 0 {borrowing_offer_vec.len()} else {borrowing_offer_vec.len() - 1};
    let best_borrowing_offer = borrowing_offer_vec.get(best_offer_index).unwrap_or(
      Offer{offer_id: "empty_offer".to_string(), owner_id: env::predecessor_account_id(), value: u128::MAX, token_id: None}
    );
    best_borrowing_offer
  }

  fn evaluate_lending_offer_possible_match(&mut self, nft_collection_id: &NftCollection, lending_offer_value: U128) -> bool {
    let best_borrowing_offer = self.get_best_borrowing_offer(nft_collection_id.to_string());
    lending_offer_value.0 >= best_borrowing_offer.value
  }

  fn evaluate_borrowing_offer_possible_match(&mut self, nft_collection_id: &NftCollection, borrowing_offer_value: U128) -> bool {
    let best_lending_offer = self.get_best_lending_offer(nft_collection_id.to_string());
    borrowing_offer_value.0 <= best_lending_offer.value
  }

  fn reorder_vec_without_specific_offer(&mut self, offers_vec: Vector<Offer> , offer_to_remove: Offer) -> Vector<Offer> {
    let mut offers_vec = offers_vec;
    let mut append_vec = Vec::new();
    let mut counter = 0;
    loop {
      let offer = offers_vec.get(counter).unwrap();
      if offer.value != offer_to_remove.value {
        append_vec.push(offers_vec.pop().unwrap());
        counter = counter + 1;
      } else {
        offers_vec.pop();
        break
      }
    }
    let mut reverse_vec = append_vec;
    reverse_vec.reverse();
    offers_vec.extend(reverse_vec.into_iter());
    offers_vec
  }

  fn choose_specific_lending_offer(&mut self, nft_collection_id: NftCollection, offer_id: String, token_id: TokenId) -> bool {
    let cloned_nft_collection_id = nft_collection_id.clone();
    let nft_collection_lending_offers = self.lending_offers.get(&nft_collection_id);
    let nft_collection_lending_offer_vec = self.lending_offers_vecs.get(&nft_collection_id);
    let specific_lending_offer = nft_collection_lending_offers.unwrap().get(&offer_id).unwrap();
    let cloned_specific_lending_offer = specific_lending_offer.clone();
    let cloned_nft_collection_id2 = cloned_nft_collection_id.clone();
    self.post_loan(specific_lending_offer.owner_id, env::predecessor_account_id(), cloned_nft_collection_id, token_id, U128(specific_lending_offer.value));
    // REORDER AND REMOVE FROM VECS
    let ordered_lending_offer_vec = self.reorder_vec_without_specific_offer(nft_collection_lending_offer_vec.unwrap(), cloned_specific_lending_offer);
    let cloned_nft_collection_id3 = cloned_nft_collection_id2.clone();
    self.lending_offers_vecs.insert(&cloned_nft_collection_id2, &ordered_lending_offer_vec);
    self.lending_offers.get(&cloned_nft_collection_id3).unwrap().remove(&offer_id).unwrap();
    true
  }

  fn choose_specific_borrowing_offer(&mut self, nft_collection_id: NftCollection, offer_id: String) -> bool {
    let cloned_nft_collection_id = nft_collection_id.clone();
    let nft_collection_borrowing_offers = self.borrowing_offers.get(&nft_collection_id);
    let nft_collection_borrowing_offer_vec = self.borrowing_offers_vecs.get(&nft_collection_id);
    let specific_borrowing_offer = nft_collection_borrowing_offers.unwrap().get(&offer_id).unwrap();
    let cloned_specific_borrowing_offer = specific_borrowing_offer.clone();
    let cloned_nft_collection_id2 = cloned_nft_collection_id.clone();
    self.post_loan(env::predecessor_account_id(), specific_borrowing_offer.owner_id, cloned_nft_collection_id, specific_borrowing_offer.token_id.unwrap(), U128(specific_borrowing_offer.value));
    // REORDER AND REMOVE FROM VECS
    let ordered_borrowing_offer_vec = self.reorder_vec_without_specific_offer(nft_collection_borrowing_offer_vec.unwrap(), cloned_specific_borrowing_offer);
    let cloned_nft_collection_id3 = cloned_nft_collection_id2.clone();
    self.borrowing_offers_vecs.insert(&cloned_nft_collection_id2, &ordered_borrowing_offer_vec);
    self.borrowing_offers.get(&cloned_nft_collection_id3).unwrap().remove(&offer_id);
    true
  }

  fn get_lending_offers_vec_from_nft_collection(&mut self, nft_collection_id: NftCollection) -> Vector<Offer> {
    let mut lending_offers_vec = match self.lending_offers_vecs.get(&nft_collection_id) {
      Some(value) => value,
      None => {
        let mut vector_id = nft_collection_id.clone();
        vector_id.push_str("lending");
        let new_vec = Vector::new(vector_id.into_bytes().to_vec());
        self.lending_offers_vecs.insert(&nft_collection_id, &new_vec);
        new_vec
      }
    };
    lending_offers_vec
  }

  fn get_borrowing_offers_vec_from_nft_collection(&mut self, nft_collection_id: NftCollection) -> Vector<Offer> {
    let mut borrowing_offers_vec = match self.borrowing_offers_vecs.get(&nft_collection_id) {
      Some(value) => value,
      None => {
        let mut vector_id = nft_collection_id.clone();
        vector_id.push_str("lending");
        let new_vec = Vector::new(vector_id.into_bytes().to_vec());
        self.borrowing_offers_vecs.insert(&nft_collection_id, &new_vec);
        new_vec
      }
    };
    borrowing_offers_vec
  }

  fn sort_order_lending_offer_vec(&mut self, lending_offers_vec: Vector<Offer> , new_offer: Offer) -> Vector<Offer> {
    let mut lending_offers_vec = lending_offers_vec;
    let mut append_vec = Vec::new();
    let mut counter = if lending_offers_vec.len() == 0 {lending_offers_vec.len()} else {lending_offers_vec.len() - 1};
    loop {
      match lending_offers_vec.get(counter) {
        Some(offer) => {
          if offer.value >= new_offer.value {
            append_vec.push(lending_offers_vec.pop().unwrap());
            if counter > 0 {
              counter = counter - 1;
            }
          } else {
            lending_offers_vec.push(&new_offer);
            break
          }
        },
        None => {
          lending_offers_vec.push(&new_offer);
          break
        }
      };
    }
    let mut reverse_vec = append_vec;
    reverse_vec.reverse();
    lending_offers_vec.extend(reverse_vec.into_iter());
    lending_offers_vec
  }

  fn sort_order_borrowing_offer_vec(&mut self, borrowing_offers_vec: Vector<Offer> , new_offer: Offer) -> Vector<Offer> {
    let mut borrowing_offers_vec = borrowing_offers_vec;
    let mut append_vec = Vec::new();
    let mut counter = if borrowing_offers_vec.len() == 0 {borrowing_offers_vec.len()} else {borrowing_offers_vec.len() - 1};
    loop {
      match borrowing_offers_vec.get(counter) {
        Some(offer) => {
          if offer.value <= new_offer.value {
            append_vec.push(borrowing_offers_vec.pop().unwrap());
            if counter > 0 {
              counter = counter - 1;
            }
          } else {
            borrowing_offers_vec.push(&new_offer);
            break
          }
        },
        None => {
          borrowing_offers_vec.push(&new_offer);
          break
        }
      };
    }
    let mut reverse_vec = append_vec;
    reverse_vec.reverse();
    borrowing_offers_vec.extend(reverse_vec.into_iter());
    borrowing_offers_vec
  }

  fn lock_warranty(&mut self, warranty_collection: AccountId, warranty_token_id: TokenId) -> bool {
    true
  }

  fn post_loan(&mut self, lender_account_id: AccountId, borrower_account_id: AccountId, warranty_collection: AccountId, warranty_token_id: TokenId, loan_value: U128) -> bool {
    // self.lock_warranty(warranty_collection, warranty_token_id);
    // lock nft
    let cloned_warranty_collection = warranty_collection.clone();
    let cloned_warranty_token_id = warranty_token_id.clone();
    let loan = Loan {
      value: loan_value.0,
      expiration_time: None,
      warranty_collection: warranty_collection,
      warranty_token_id: warranty_token_id,
    };

    self.loans.insert(&self.token_id_counter.to_string(), &loan);

    let token_metadata = TokenMetadata {
      title: Some("Loan".to_string()),
      description: Some("fwiefjdadger".to_string()),
      media: None,
      media_hash: None,
      copies: Some(1u64),
      issued_at: None,
      expires_at: None,
      starts_at: None,
      updated_at: None,
      extra: None,
      reference: None,
      reference_hash: None,
      loan_value: Some(loan_value.0),
      loan_expiration_time: None,
      warranty_collection: Some(cloned_warranty_collection),
      warranty_token_id: Some(cloned_warranty_token_id)
    };
    let cloned_token_metadata = token_metadata.clone();

    // mint note
    ext_nft_contract::nft_mint(
      self.token_id_counter.to_string(),
      lender_account_id,
      token_metadata,
      // CHANGE THIS LATER
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );
    // mint receipt
    ext_nft_contract::nft_mint(
      self.token_id_counter.to_string(),
      borrower_account_id,
      cloned_token_metadata,
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );
    self.token_id_counter = self.token_id_counter + 1;
    true
  }

  #[private]
  pub fn callback_pay_loan(&mut self, #[callback] token: Token, loan: Loan) -> bool {
    let cloned_token_id = token.token_id.clone();
    let cloned_owner_id = token.owner_id.clone();
    Promise::new(token.owner_id).transfer(loan.value);
    // UNLOCK NFT
    ext_nft_contract::nft_transfer(
      env::predecessor_account_id(), 
      loan.warranty_token_id,
      None,
      None,
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );
    ext_nft_contract::nft_burn(
      token.token_id, 
      env::predecessor_account_id(),
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );

    ext_nft_contract::nft_burn(
      cloned_token_id, 
      cloned_owner_id,
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );
    true
  }

  fn pay_loan(&mut self, token_id: TokenId) ->  bool {
    let loan = self.loans.get(&token_id).unwrap();
    // get lender note
    ext_nft_contract::nft_token(
      token_id,
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS

    ).then(
      ext_self::callback_pay_loan(
        loan,
        &self.owner_id,
        NO_DEPOSIT,
        BASE_GAS
      ));
    true
  }

  #[private]
  pub fn callback_transfer_warrancy(&mut self, #[callback] token: Token, loan: Loan) -> bool {
    let cloned_token_id = token.token_id.clone();
    let cloned_owner_id = token.owner_id.clone();
    // UNLOCK NFT
    ext_nft_contract::nft_transfer(
      env::predecessor_account_id(), 
      loan.warranty_token_id,
      None,
      None,
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );
    ext_nft_contract::nft_burn(
      token.token_id, 
      env::predecessor_account_id(),
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );

    ext_nft_contract::nft_burn(
      cloned_token_id, 
      cloned_owner_id,
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS
    );
    true
  }

  fn transfer_warranty(&mut self, token_id: TokenId) -> bool{
    let loan = self.loans.get(&token_id).unwrap();
    // get borrower receipt
    ext_nft_contract::nft_token(
      token_id,
      &self.owner_id,
      NO_DEPOSIT,
      BASE_GAS

    ).then(
      ext_self::callback_transfer_warrancy(
        loan,
        &self.owner_id,
        NO_DEPOSIT,
        BASE_GAS
      ));
    true
  }


  fn nft_on_transfer(
    &mut self,
    sender_id: String,
    previous_owner_id: String,
    token_id: String,
    msg: String,
  ) -> bool {
    let pay_loan_args: Value = serde_json::from_str(&msg).unwrap();
    self.pay_loan(pay_loan_args["token_id"].to_string())
  }

  #[payable]
  fn post_lending_offer(&mut self, nft_collection_id: AccountId, value_offered: U128) -> bool {
    let cloned_nft_collection_id = nft_collection_id.clone();
    let cloned_nft_collection_id2 = cloned_nft_collection_id.clone();
    if self.evaluate_lending_offer_possible_match(&nft_collection_id, value_offered) {
      let best_borrowing_offer = self.get_best_borrowing_offer(nft_collection_id);
      self.post_loan(env::predecessor_account_id(), best_borrowing_offer.owner_id, cloned_nft_collection_id2, best_borrowing_offer.token_id.unwrap(), value_offered);
      self.borrowing_offers_vecs.get(&cloned_nft_collection_id).unwrap().pop();
      // lock nft
      false
    }
    else {
      let offer_id = self.current_lending_offer_id.get(&nft_collection_id).unwrap_or(0);
      let offer = Offer{offer_id: offer_id.to_string(), owner_id: env::predecessor_account_id(), value: value_offered.0, token_id: None};
      let cloned_nft_collection_id_2 = cloned_nft_collection_id.clone();
      let mut lending_offers_vec = self.get_lending_offers_vec_from_nft_collection(cloned_nft_collection_id);
      let ordered_lending_offer_vec = self.sort_order_lending_offer_vec(lending_offers_vec, offer);
      self.lending_offers_vecs.insert(&cloned_nft_collection_id_2, &ordered_lending_offer_vec);
      self.current_lending_offer_id.insert(&cloned_nft_collection_id_2, &(offer_id + 1));
      true
    }
  }

  
  fn post_borrowing_offer(&mut self, nft_collection_id: NftCollection, value_offered: U128, collateral_nft: TokenId) -> bool {
    let cloned_nft_collection_id = nft_collection_id.clone();
    let cloned_nft_collection_id2 = cloned_nft_collection_id.clone();
    if self.evaluate_borrowing_offer_possible_match(&nft_collection_id, value_offered) {
      let best_lending_offer = self.get_best_lending_offer(nft_collection_id);
      self.post_loan(best_lending_offer.owner_id, env::predecessor_account_id(), cloned_nft_collection_id2, collateral_nft, value_offered);
      self.lending_offers_vecs.get(&cloned_nft_collection_id).unwrap().pop();
      // lock nft
      false
    }
    else {
      let offer_id = self.current_borrowing_offer_id.get(&nft_collection_id).unwrap_or(0);
      let offer = Offer{offer_id: offer_id.to_string(), owner_id: env::predecessor_account_id(), value: value_offered.0, token_id: Some(collateral_nft)};
      let cloned_nft_collection_id_2 = cloned_nft_collection_id.clone();
      let mut borrowing_offers_vec = self.get_borrowing_offers_vec_from_nft_collection(cloned_nft_collection_id);
      let ordered_borrowing_offer_vec = self.sort_order_lending_offer_vec(borrowing_offers_vec, offer);
      self.borrowing_offers_vecs.insert(&cloned_nft_collection_id_2, &ordered_borrowing_offer_vec);
      self.current_borrowing_offer_id.insert(&cloned_nft_collection_id_2, &(offer_id + 1));
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
      let contract = LendingNftCollateral::new(accounts(1).into());
      testing_env!(context.is_view(true).build());
      assert_eq!(contract.owner_id, accounts(1).to_string());
  }

  #[test]
  fn test_get_best_lending_offer() {

  }

  #[test]
  fn test_get_best_borrowing_offer() {
    
  }

  #[test]
  fn test_evaluate_lending_offer_possible_match() {
    
  }

  #[test]
  fn test_evaluate_borrowing_offer_possible_match() {
    
  }

  #[test]
  fn test_get_specific_lending_offer() {
    
  }

  #[test]
  fn test_get_specific_borrowing_offer() {
    
  }

  #[test]
  fn test_get_lending_offers_vec_from_nft_collection() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let cloned_nft_collection_id = nft_collection_id.clone();
    let mut lending_offers_empty_vec = contract.get_lending_offers_vec_from_nft_collection(cloned_nft_collection_id);

    let offer = Offer{offer_id: "offer_id_test".to_string(), owner_id: accounts(1).into(), value: 10, token_id: None};
    lending_offers_empty_vec.push(&offer);
    contract.lending_offers_vecs.insert(&nft_collection_id, &lending_offers_empty_vec);
    assert_eq!(contract.lending_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().offer_id, "offer_id_test");
    assert_eq!(contract.lending_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().owner_id, accounts(1).to_string());
    assert_eq!(contract.lending_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().value, 10);
  }

  #[test]
  fn test_get_borrowing_offers_vec_from_nft_collection() {

  }


  #[test]
  fn test_sort_order_lending_offer_vec() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

      let nft_collection_id = "nft_collection_test".to_string();
      let cloned_nft_collection_id = nft_collection_id.clone();
      let mut vector_id = nft_collection_id.clone();
      vector_id.push_str("lending");
      let mut new_vec = Vector::new(vector_id.into_bytes().to_vec());
      let offer = Offer{offer_id: "offer_id_test".to_string(), owner_id: accounts(1).into(), value: 10, token_id: None};

      // test with empty vector
      let ordered_offer_vec = contract.sort_order_lending_offer_vec(new_vec, offer);
      assert_eq!(ordered_offer_vec.get(0).unwrap().value, 10);
      
      // test with a lower value
      let offer2 = Offer{offer_id: "offer_id_test".to_string(), owner_id: accounts(1).into(), value: 5, token_id: None};
      let ordered_offer_vec2 = contract.sort_order_lending_offer_vec(ordered_offer_vec, offer2);
      assert_eq!(ordered_offer_vec2.get(0).unwrap().value, 5);
      assert_eq!(ordered_offer_vec2.get(1).unwrap().value, 10);

      //test with a higher value
      let offer3 = Offer{offer_id: "offer_id_test".to_string(), owner_id: accounts(1).into(), value: 20, token_id: None};
      let ordered_offer_vec3 = contract.sort_order_lending_offer_vec(ordered_offer_vec2, offer3);
      assert_eq!(ordered_offer_vec3.get(0).unwrap().value, 5);
      assert_eq!(ordered_offer_vec3.get(1).unwrap().value, 10);
      assert_eq!(ordered_offer_vec3.get(2).unwrap().value, 20);
  } 

  #[test]
  fn test_sort_order_borrowing_offer_vec() {

  }

  #[test]
  fn test_post_loan() {

  }

  #[test]
  fn test_post_lending_offer() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into());

    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

      let nft_collection_id = "nft_collection_test".to_string();
      let cloned_nft_collection_id = nft_collection_id.clone();
      let success = contract.post_lending_offer(cloned_nft_collection_id, U128(10));
      assert_eq!(success, true);
      assert_eq!(contract.lending_offers_vecs.get(&nft_collection_id).unwrap().get(0).unwrap().value, 10);
    }

    #[test]
    fn test_post_borrowing_offer() {
  
    }
}