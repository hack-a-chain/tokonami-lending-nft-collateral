use crate::*;
use serde_json::Value;

#[near_bindgen]
impl LendingNftCollateral {

  pub fn post_loan(&mut self, lender_account_id: AccountId, borrower_account_id: AccountId, warranty_collection: AccountId, warranty_token_id: TokenId, loan_value: U128) -> bool {
    let loan = Loan {
      value: loan_value.0,
      expiration_time: env::block_timestamp() as u128 + self.loan_expiration_seconds_limit,
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

  #[payable]
  pub fn pay_loan(&mut self, token_id: TokenId, note_owner_id: AccountId) -> Promise {
    // only receipt contract can call this function
    assert!(env::predecessor_account_id() == self.receipt_address, "Only receipt contract can call this function");
    let loan = self.loans.get(&token_id).unwrap();
    
    let borrower_balance = self.balances.get(&env::predecessor_account_id()).unwrap_or(0);
    assert!(borrower_balance >= loan.value, "You don't have enough credit for this transaction");
    self.balances.insert(&env::predecessor_account_id(), &(borrower_balance - loan.value));
    Promise::new(note_owner_id.clone()).transfer(loan.value);
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
    )
  }

  //function to call loan
  #[payable]
  pub fn transfer_warranty_loan(&mut self, token_id: TokenId, sender_owner_id: AccountId) -> Promise {
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
    )
  }
}