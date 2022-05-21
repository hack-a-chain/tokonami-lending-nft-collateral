use std::convert::TryInto;
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U128, U64};
pub use near_sdk::serde_json::{json, value::Value};
pub use near_sdk_sim::{call, view, deploy, init_simulator, to_yocto, UserAccount, 
    ContractAccount, DEFAULT_GAS, ViewResult, ExecutionResult};
pub use near_sdk::AccountId;
use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    LENDING_CONTRACT => "../lending_contract/target/wasm32-unknown-unknown/release/contract.wasm",
    NFT_CONTRACT => "../nft_contract/target/wasm32-unknown-unknown/release/non_fungible_token.wasm"
}

use std::convert::TryFrom;


const NFT_FEE: u128 = 4_000;
const OWNER_FEE: u128 = 500;
const HOUSE_FEE: u128 = 500;
const PARTNER_FEE: u128 = 100;
const FRACTIONAL_BASE: u128 = 100_000;
const MIN_BALANCE_FRACTION: u128 = 100;

const GAS_ATTACHMENT: u64 = 300_000_000_000_000;

#[test]
fn simulate_full_flow() {
    //Test full flow from deploying app
    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = 300_000_000_000_000;
    genesis.gas_price = 0;

    let root = init_simulator(Some(genesis));

    let owner_account_lending = root.create_user("owner_account1".to_string(), to_yocto("100"));
    let owner_account_nft_collection = root.create_user("owner_account2".to_string(), to_yocto("100"));

    let alice_account = root.create_user("alice".to_string(), to_yocto("100"));
    let bob_account = root.create_user("bob".to_string(), to_yocto("100"));

   //deploy contracts
    let lending_account = root.deploy(
        &LENDING_CONTRACT,
        "lending_contract".to_string(),
        to_yocto("100")
    );

    let nft_note_account = root.deploy(
        &NFT_CONTRACT,
        "note_contract".to_string(),
        to_yocto("100")
    );

    let nft_receit_account = root.deploy(
        &NFT_CONTRACT,
        "receipt_contract".to_string(),
        to_yocto("100")
    );

    let nft_collection_account = root.deploy(
        &NFT_CONTRACT,
        "nft_collection_contract".to_string(),
        to_yocto("100")
    );

        //initialize contracts
    owner_account_lending.call(
        lending_account.account_id(), 
        "new", 
        &json!({
            "owner_id": owner_account_lending.account_id(), 
            "note_address": nft_note_account.account_id(), 
            "receipt_address": nft_receit_account.account_id()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    owner_account_lending.call(
        nft_note_account.account_id(), 
        "new", 
        &json!({
            "owner_id": lending_account.account_id(), 
            "metadata": NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "Example NEAR non-fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some("EXAMPLE".to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            }
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();


    owner_account_lending.call(
        nft_receit_account.account_id(), 
        "new", 
        &json!({
            "owner_id": lending_account.account_id(), 
            "metadata": NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "Example NEAR non-fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some("EXAMPLE".to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            }
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();


    owner_account_lending.call(
        nft_collection_account.account_id(), 
        "new", 
        &json!({
            "owner_id": lending_account.account_id(), 
            "metadata": NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "Example NEAR non-fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some("EXAMPLE".to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            }
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    // deposit balance
    
    alice_account.call(
        lending_account.account_id(),
        "deposit_balance",
        &json!({
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        to_yocto("10")
    ).assert_success();

    let result: U128 = root.view(
        lending_account.account_id(),
        "get_balance_value",
        &json!({
            "owner_id": alice_account.account_id()
        }).to_string().into_bytes()
    ).unwrap_json();


    // assert_eq!(result,  to_yocto("10"));

    // insert new nft collection

    owner_account_lending.call(
        lending_account.account_id(),
        "insert_new_nft_collection",
        &json!({
            "nft_collection_id": nft_collection_account.account_id(),
            "interest": U128(1)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    //post lending offer  

    alice_account.call(
        lending_account.account_id(),
        "post_lending_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id(), 
            "value_offered": U128(100)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();


    // post borrowing offer (without match)  

    // choose specific lending offer

    // choose specific borrowing offer


    // get best lending offer

    // get best borrowing offer

    // cancel specific lending offer

    // cancel specific borrowing offer

}