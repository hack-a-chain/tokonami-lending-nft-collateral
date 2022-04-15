use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, TreeMap, LazyOption};
use near_sdk::json_types::U128;

use note_contract::Note;

pub type TokenId = String;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct LendingNftCollateral{

  pub owner_id: AccountId,
  pub borrow_offers: LookupMap<AccountId, UnorderedMap<TokenId, Offer>>,
  pub lending_offers: LookupMap<AccountId, UnorderedMap<U128, Offer>>,
  pub loans: LookupMap≤TokenId, ≤
}

impl NftLending for LendingNftCollateral{
  


}



