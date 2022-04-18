use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::env;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::init;

mod lending_contract_interface;
// use crate::lending_contract_interface::NftLending;

pub type TokenId = String;
pub type NftCollection = AccountId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Offer {
  pub offer_id: String,
  pub owner_id: AccountId,
  pub value: u128
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LendingNftCollateral{

  pub owner_id: ValidAccountId,
  pub borrowing_offers: LookupMap<NftCollection, LookupMap<TokenId, Offer>>,
  pub lending_offers: LookupMap<NftCollection, LookupMap<String, Offer>>,
  pub current_lending_offer_id: LookupMap<NftCollection, u128>,
  pub borrowing_offers_vecs: LookupMap<NftCollection, Vector<Offer>>,
  pub lending_offers_vecs: LookupMap<NftCollection, Vector<Offer>>
}
// criar a lógica do match
// criar a lógica de selecionar oferta específica
// criar testes unitários

impl Default for LendingNftCollateral {
  fn default() -> Self {
      panic!("Should be initialized before usage")
  }
}

// impl NftLending for LendingNftCollateral{
#[near_bindgen]
impl LendingNftCollateral{

  #[init]
  fn new(owner_id: ValidAccountId) -> Self {
    Self {
      owner_id: owner_id,
      borrowing_offers: LookupMap::new(b"borrowing_offers".to_vec()),
      lending_offers: LookupMap::new(b"lending_offers".to_vec()),
      current_lending_offer_id: LookupMap::new(b"current_lending_offer_id".to_vec()),
      borrowing_offers_vecs: LookupMap::new(b"borrowing_offers_vecs".to_vec()),
      lending_offers_vecs: LookupMap::new(b"lending_offers_vecs".to_vec())
    }
  }

  fn evaluate_lending_offer_possible_match(&mut self, nft_collection_id: NftCollection, lending_offer_value: U128) -> bool {
    let borrowing_vec = self.borrowing_offers_vecs.get(&nft_collection_id).unwrap();
    let best_borrowing_offer_value = borrowing_vec.get(borrowing_vec.len() - 1).unwrap();
    lending_offer_value.0 >= best_borrowing_offer_value.value
  }
 
  fn post_lending_offer(&mut self, nft_collection_id: NftCollection, value_offered: U128) -> bool {
    let mut offer_map = self.lending_offers.get(&nft_collection_id).expect("NftCollection invalid");
    let offer_id = self.current_lending_offer_id.get(&nft_collection_id).unwrap_or(0);
    let offer = Offer{offer_id: offer_id.to_string(), owner_id: env::predecessor_account_id(), value: value_offered.0};
    offer_map.insert(&offer_id.to_string(), &offer);
    self.current_lending_offer_id.insert(&nft_collection_id, &(offer_id + 1));

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
    let mut append_vec = Vec::new();
    let mut counter = if lending_offers_vec.len() == 0 {lending_offers_vec.len()} else {lending_offers_vec.len() - 1};
    loop {
      match lending_offers_vec.get(counter) {
        Some(offer) => {
          if offer.value >= value_offered.0 {
            append_vec.push(lending_offers_vec.pop().unwrap());
            counter = counter - 1;
          } else {
            lending_offers_vec.push(&offer);
            break
          }
        },
        None => {
          lending_offers_vec.push(&offer);
          break
        }
      };
    }
    let mut reverse_vec = append_vec;
    reverse_vec.reverse();
    lending_offers_vec.extend(reverse_vec.into_iter());
    true
  }
  
  fn post_borrowing_offer(&mut self, nft_collection_id: NftCollection, value_offered: U128, collateral_nft: TokenId) -> bool {
    let mut offer_map = self.borrowing_offers.get(&nft_collection_id).expect("NftCollection invalid");
    let offer_id = self.current_lending_offer_id.get(&nft_collection_id).unwrap_or(0);
    let offer = Offer{offer_id: offer_id.to_string(), owner_id: env::predecessor_account_id(), value: value_offered.0};
    offer_map.insert(&collateral_nft, &offer);

    let mut borrowing_offers_vec = match self.borrowing_offers_vecs.get(&nft_collection_id) {
      Some(value) => value,
      None => {
        let mut vector_id = nft_collection_id.clone();
        vector_id.push_str("borrow");
        let new_vec = Vector::new(vector_id.into_bytes().to_vec());
        self.borrowing_offers_vecs.insert(&nft_collection_id, &new_vec);
        new_vec
      }
    };
    let mut append_vec = Vec::new();
    let mut counter = if borrowing_offers_vec.len() == 0 {borrowing_offers_vec.len()} else {borrowing_offers_vec.len() - 1};
    loop {
      match borrowing_offers_vec.get(counter) {
        Some(offer) => {
          if offer.value >= value_offered.0 {
            append_vec.push(borrowing_offers_vec.pop().unwrap());
            counter = counter - 1;
          } else {
            borrowing_offers_vec.push(&offer);
            break
          }
        },
        None => {
          borrowing_offers_vec.push(&offer);
          break
        }
      };
    }
    let mut reverse_vec = append_vec;
    reverse_vec.reverse();
    borrowing_offers_vec.extend(reverse_vec.into_iter());
    true
  }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
  use near_sdk::test_utils::{accounts, VMContextBuilder};
  use near_sdk::testing_env;
  use near_sdk::MockedBlockchain;

  use super::*;

  fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
      let mut builder = VMContextBuilder::new();
      builder
          .current_account_id(accounts(0))
          .signer_account_id(predecessor_account_id.clone())
          .predecessor_account_id(predecessor_account_id);
      builder
  }

  // #[test]
  // fn test_new() {
  //     let mut context = get_context(accounts(1));
  //     testing_env!(context.build());
  //     let contract = LendingNftCollateral::new(accounts(1).into());
  //     testing_env!(context.is_view(true).build());
  //     assert_eq!(contract.nft_token("1".to_string()), None);
  // }
}


