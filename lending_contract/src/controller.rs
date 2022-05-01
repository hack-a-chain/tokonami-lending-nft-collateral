use crate::*;
use serde_json::Value;

impl LendingNftCollateral {
  pub fn evaluate_lending_offer_possible_match(&mut self, nft_collection_id: &NftCollection, lending_offer_value: U128) -> bool {
    match self.get_best_borrowing_offer(nft_collection_id.to_string()) {
      Some(offer) => lending_offer_value.0 >= offer.value,
      None => false
    }
  }

  pub fn evaluate_borrowing_offer_possible_match(&mut self, nft_collection_id: &NftCollection, borrowing_offer_value: U128) -> bool {
    match self.get_best_lending_offer(nft_collection_id.to_string()) {
      Some(offer) => borrowing_offer_value.0 <= offer.value,
      None => false
    }
  }

  pub fn reorder_vec_without_specific_offer(&mut self, offers_vec: &mut Vector<Offer> , offer_to_remove: Offer) {
    let mut append_vec = Vec::new();
    let mut counter = 0;
    loop {
      let offer = offers_vec.get(counter).unwrap();
      if offer.offer_id != offer_to_remove.offer_id {
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
  }

  pub fn get_lending_offers_vec_from_nft_collection(&mut self, nft_collection_id: NftCollection) -> Vector<Offer> {
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

  pub fn get_borrowing_offers_vec_from_nft_collection(&mut self, nft_collection_id: NftCollection) -> Vector<Offer> {
    let mut borrowing_offers_vec = match self.borrowing_offers_vecs.get(&nft_collection_id) {
      Some(value) => value,
      None => {
        let mut vector_id = nft_collection_id.clone();
        vector_id.push_str("borrowing");
        let new_vec = Vector::new(vector_id.into_bytes().to_vec());
        self.borrowing_offers_vecs.insert(&nft_collection_id, &new_vec);
        new_vec
      }
    };
    borrowing_offers_vec
  }

  pub fn sort_order_lending_offer_vec(&mut self, lending_offers_vec: Vector<Offer> , new_offer: Offer) -> Vector<Offer> {
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

  pub fn sort_order_borrowing_offer_vec(&mut self, borrowing_offers_vec: Vector<Offer> , new_offer: Offer) -> Vector<Offer> {
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
  fn test_evaluate_lending_offer_possible_match() {
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

    let result_true = contract.evaluate_lending_offer_possible_match(&nft_collection_id.clone(), U128(10));
    let result_true2 = contract.evaluate_lending_offer_possible_match(&nft_collection_id.clone(), U128(15));
    let result_false = contract.evaluate_lending_offer_possible_match(&nft_collection_id.clone(), U128(5));
    assert_eq!(result_true, true);
    assert_eq!(result_true2, true);
    assert_eq!(result_false, false);
  }

  #[test]
  fn test_evaluate_borrowing_offer_possible_match() {
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
    let lending_offer1 = Offer{offer_id: "offer_id_test1".to_string(), owner_id: accounts(1).into(), value: 20, token_id: None};
    let lending_offer2 = Offer{offer_id: "offer_id_test2".to_string(), owner_id: accounts(1).into(), value: 10, token_id: None};
    new_vec.push(&lending_offer1);
    new_vec.push(&lending_offer2);
    contract.lending_offers_vecs.insert(&nft_collection_id, &new_vec);

    let result_true = contract.evaluate_borrowing_offer_possible_match(&nft_collection_id.clone(), U128(10));
    let result_false = contract.evaluate_borrowing_offer_possible_match(&nft_collection_id.clone(), U128(15));
    let result_true2 = contract.evaluate_borrowing_offer_possible_match(&nft_collection_id.clone(), U128(5));
    assert_eq!(result_true, true);
    assert_eq!(result_true2, true);
    assert_eq!(result_false, false);
  }

  #[test]
  fn test_get_lending_offers_vec_from_nft_collection() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    let mut contract = LendingNftCollateral::new(accounts(1).into(), accounts(2).into(), accounts(3).into());
            
    testing_env!(context
      .storage_usage(env::storage_usage())
      .attached_deposit(MINT_STORAGE_COST)
      .predecessor_account_id(accounts(0))
      .build());

    let nft_collection_id = "nft_collection_test".to_string();
    let mut lending_offers_empty_vec = contract.get_lending_offers_vec_from_nft_collection(nft_collection_id.clone());

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

}  