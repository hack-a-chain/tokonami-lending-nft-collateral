use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk::PromiseOrValue;
use near_sdk::collections::{LazyOption, Vector};

pub type TokenId = String;

pub trait NftLending {

    //think about the offer's IDs, ideally we'd need something
    //unique that cannot overflow (ideia hash of account plus sequential number)

    //notes and receipts read functions
    fn get_user_notes(&self, account_id: AccountId) -> Vec<U128>;

    fn get_user_receipts(&self, account_id: AccountId) -> Vec<U128>;

    fn call_note(&mut self, note_id: TokenId) -> Option<U128>;

    fn pay_receipt(&mut self, receipt_id: TokenId) -> Option<U128>;

    //marketplace functions
    fn post_loan_offer(&mut self, nft_collection_id: AccountId, value_offered: U128) -> Option<U128>;

    fn loan_offer_at_market_rate(&mut self, nft_collection_id: AccountId) -> Option<U128>;

    fn loan_offer_to_specific_request(&mut self, nft_collection_id: AccountId, offer_id: TokenId) -> Option<U128>;

    fn post_borrow_offer(&mut self, nft_collection_id: AccountId, value_offered: U128, collateral_nft: TokenId) -> Option<U128>;

    fn borrow_offer_at_market_rate(&mut self, nft_collection_id: AccountId, collateral_nft: TokenId) -> Option<U128>;

    fn borrow_offer_to_specific_request(&mut self, nft_collection_id: AccountId, collateral_nft: TokenId, offer_id: U128) -> Option<U128>;

    //marketplace view functions
    fn get_loan_offers(&self, nft_collection_id: AccountId, start_index: U128, pagination: U128) -> Vec<>;

    fn get_borrow_offers(&self, nft_collection_id: AccountId, start_index: U128, pagination: U128) -> Vec<>;

    //governance functions
    fn add_collection(&mut self, nft_collection_id: AccountId, apy_rate: U128) -> bool;

    fn remove_collection(&mut self, nft_collection_id: AccountId, apy_rate: U128) -> bool;

    fn alter_collection(&mut self, nft_collection_id: AccountId, apy_rate: U128) -> bool;

    fn retrieve_funds(&mut self) -> bool;

    fn get_contract_params(&self) -> Struct;

    fn alter_contract_params(&mut self) -> bool;
}

//data structure
//cap on loan offers - if above cap remove from vector