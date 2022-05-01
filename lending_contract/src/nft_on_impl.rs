use crate::*;
use serde_json::Value;

//structure of message:
/*
function: string,
args: {
    argName: String
}
*/

impl LendingNftCollateral {

    //nft on transfer should validate function call type and call appropriate function
    //must return false so that nft_resolve_transfer doesn't return token to original holder
    pub fn nft_on_transfer (
        &mut self,
        sender_id: String,
        previous_owner_id: String,
        token_id: String,
        msg: String) -> bool {

        let parsed_message: Value = serde_json::from_str(&msg).unwrap();

        if parsed_message["function"].as_str().unwrap() == "post_borrowing_offer" {
            self.post_borrowing_offer(env::predecessor_account_id(), U128(parsed_message["args"]["value_offered"].as_str().unwrap().parse().unwrap()), token_id, previous_owner_id);
        } else if parsed_message["function"].as_str().unwrap() == "pay_loan" {
            self.pay_loan(token_id, previous_owner_id);
        } else if parsed_message["function"].as_str().unwrap() == "transfer_warranty" {
            //needs to find a way to receive money
            self.transfer_warranty(token_id, sender_id);
        } else {
            panic!("msg could not be parsed");
        }

        false
    }

}