use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, TreeMap, LazyOption};
use near_sdk::json_types::U128;

use crate::lending_nft_collateral::NftLending;

pub type TokenId = String;

#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Offer {
  pub owner: AccountId,
  pub value: LazyOption<U128>
}

#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Loan{
  pub note_id: TokenId,
  pub receipt_id: TokenId
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct LendingNftCollateral{

  pub owner_id: AccountId,

  pub notes: TreeMap<TokenId, AccountId>,
  pub receipts: TreeMap<TokenId, AccountId>,

  pub notes_per_owner: Option<LookupMap<AccountId, UnorderedSet<TokenId>>>,
  pub receipts_per_owner: Option<LookupMap<AccountId, UnorderedSet<TokenId>>>,
  pub notes_per_nft: LookupMap<AccountId, UnorderedMap<TokenId, Loan>>,

  pub borrow_offers: LookupMap<AccountId, UnorderedMap<TokenId, Offer>>,
  pub lending_offers: LookupMap<AccountId, UnorderedMap<U128, Offer>>,

}

impl NftLending for LendingNftCollateral{

}



