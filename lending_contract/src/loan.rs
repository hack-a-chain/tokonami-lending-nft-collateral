use crate::*;
use serde_json::Value;

const SECONDS_IN_A_YEAR: u128 = 31_536_000;


impl LendingNftCollateral {

  pub fn post_loan(&mut self, lender_account_id: AccountId, borrower_account_id: AccountId, warranty_collection: AccountId, warranty_token_id: TokenId, loan_value: U128) -> bool {
    let nft_collection_interest = self.get_nft_collection_interest(warranty_collection.clone());
    let expiration_time = env::block_timestamp() as u128 + self.loan_expiration_seconds_limit;
    let loan = Loan {
      value_before: loan_value.0,
      value_after: loan_value.0 * nft_collection_interest * (expiration_time / SECONDS_IN_A_YEAR),
      expiration_time: expiration_time,
      warranty_collection: warranty_collection.clone(),
      warranty_token_id: warranty_token_id.clone(),
    };

    self.loans.insert(&self.token_id_counter.to_string(), &loan);

    let token_metadata = TokenMetadata {
      title: Some("Loan".to_string()),
      // change this later
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
      warranty_collection: Some(warranty_collection.clone()),
      warranty_token_id: Some(warranty_token_id.clone())
    };

    // mint note
    ext_nft_contract::nft_mint(
      self.token_id_counter.to_string(),
      lender_account_id,
      token_metadata.clone(),
      &self.note_address,
      NO_DEPOSIT,
      BASE_GAS
    );
    // mint receipt
    ext_nft_contract::nft_mint(
      self.token_id_counter.to_string(),
      borrower_account_id,
      token_metadata.clone(),
      &self.receipt_address,
      NO_DEPOSIT,
      BASE_GAS
    );
    self.token_id_counter = self.token_id_counter + 1;
    true
  }

  pub fn pay_loan(&mut self, token_id: TokenId, note_owner_id: AccountId) {
    let initial_storage = U128(env::storage_usage() as u128);
    // only receipt contract can call this function
    assert!(env::predecessor_account_id() == self.receipt_address, "Only receipt contract can call this function");
    let loan = self.loans.get(&token_id).unwrap();
    
    self.reduce_and_withdraw_balance(U128(loan.value_after));

    ext_nft_contract::nft_transfer(
      env::current_account_id(), 
      loan.warranty_token_id,
      None,
      None,
      &loan.warranty_collection,
      NO_DEPOSIT,
      BASE_GAS
    );
    ext_nft_contract::nft_burn(
      token_id.clone(), 
      &self.note_address,
      NO_DEPOSIT,
      BASE_GAS
    );

    ext_nft_contract::nft_burn(
      token_id.clone(), 
      &self.receipt_address,
      NO_DEPOSIT,
      BASE_GAS
    );

    let final_storage = U128(env::storage_usage() as u128);
    self.set_storage(initial_storage, final_storage);
  }

  //function to call loan
  pub fn transfer_warranty_loan(&mut self, token_id: TokenId, sender_owner_id: AccountId) {
    let initial_storage = U128(env::storage_usage() as u128);

    assert!(env::predecessor_account_id() == self.note_address, "Only note contract can call this function");
    let loan = self.loans.get(&token_id).unwrap();
    assert!(loan.expiration_time < env::block_timestamp() as u128, "This loan hasn't expired yet");
    ext_nft_contract::nft_transfer(
      env::current_account_id(), 
      loan.warranty_token_id,
      None,
      None,
      &sender_owner_id.clone(),      
      NO_DEPOSIT,
      BASE_GAS
    );
    ext_nft_contract::nft_burn(
      token_id.clone(), 
      &self.note_address,
      NO_DEPOSIT,
      BASE_GAS
    );

    ext_nft_contract::nft_burn(
      token_id.clone(), 
      &self.receipt_address,
      NO_DEPOSIT,
      BASE_GAS
    );

    let final_storage = U128(env::storage_usage() as u128);
    self.set_storage(initial_storage, final_storage);
  }
}