use std::convert::TryInto;
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U128, U64};
pub use near_sdk::serde_json::{json, value::Value};
pub use near_sdk_sim::{call, view, deploy, init_simulator, to_yocto, UserAccount, 
    ContractAccount, DEFAULT_GAS, ViewResult, ExecutionResult};
pub use near_sdk::AccountId;
use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};


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

    let rachel_account = root.create_user("rachel".to_string(), to_yocto("100"));
    let monica_account = root.create_user("monica".to_string(), to_yocto("100"));
    let phoebe_account = root.create_user("phoebe".to_string(), to_yocto("100"));
    let chandler_account = root.create_user("chandler".to_string(), to_yocto("100"));
    let joey_account = root.create_user("joey".to_string(), to_yocto("100"));
    let ross_account = root.create_user("ross".to_string(), to_yocto("100"));

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
    
    rachel_account.call(
        lending_account.account_id(),
        "deposit_balance",
        &json!({
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        to_yocto("10")
    ).assert_success();

    monica_account.call(
        lending_account.account_id(),
        "deposit_balance",
        &json!({
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        to_yocto("10")
    ).assert_success();

    phoebe_account.call(
        lending_account.account_id(),
        "deposit_balance",
        &json!({
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        to_yocto("10")
    ).assert_success();

    chandler_account.call(
        lending_account.account_id(),
        "deposit_balance",
        &json!({
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        to_yocto("9")
    ).assert_success();

    ross_account.call(
        lending_account.account_id(),
        "deposit_balance",
        &json!({
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        to_yocto("9")
    ).assert_success();

    joey_account.call(
        lending_account.account_id(),
        "deposit_balance",
        &json!({
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        to_yocto("9")
    ).assert_success();

    let result_rachel: U128 = root.view(
        lending_account.account_id(),
        "get_balance_value",
        &json!({
            "owner_id": rachel_account.account_id()
        }).to_string().into_bytes()
    ).unwrap_json();

    let result_chandler: U128 = root.view(
        lending_account.account_id(),
        "get_balance_value",
        &json!({
            "owner_id": chandler_account.account_id()
        }).to_string().into_bytes()
    ).unwrap_json();

    assert_eq!(result_rachel,  U128(to_yocto("10")));
    assert_eq!(result_chandler,  U128(to_yocto("9")));

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

    // offer type

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, BorshDeserialize, BorshSerialize)]
    #[serde(crate = "near_sdk::serde")]
    pub struct Offer {
        pub offer_id: String,
        pub owner_id: AccountId,
        pub value: u128,
        pub token_id: Option<TokenId>
      }

    //post lending offer  

    rachel_account.call(
        lending_account.account_id(),
        "post_lending_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id(), 
            "value_offered": U128(100)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    monica_account.call(
        lending_account.account_id(),
        "post_lending_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id(), 
            "value_offered": U128(110)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    let best_lending_offer1: Option<Offer> = root.call(
        lending_account.account_id(),
        "get_best_lending_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).unwrap_json();

    assert_eq!(best_lending_offer1.unwrap().value,  110);

    phoebe_account.call(
        lending_account.account_id(),
        "post_lending_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id(), 
            "value_offered": U128(120)
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

      let best_lending_offer2: Option<Offer> = root.call(
        lending_account.account_id(),
        "get_best_lending_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).unwrap_json();

    assert_eq!(best_lending_offer2.unwrap().value,  120);

    // post borrowing offer

    // without match

    chandler_account.call(
        lending_account.account_id(),
        "post_borrowing_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id(), 
            "value_offered": U128(150),
            "collateral_nft": "token_id".to_string(),
            "nft_owner_id": monica_account.account_id(),
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    joey_account.call(
        lending_account.account_id(),
        "post_borrowing_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id(), 
            "value_offered": U128(160),
            "collateral_nft": "token_id".to_string(),
            "nft_owner_id": monica_account.account_id(),
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    let best_borrowing_offer1: Option<Offer> = root.call(
        lending_account.account_id(),
        "get_best_borrowing_offer",
        &json!({
            "nft_collection_id": nft_collection_account.account_id()
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).unwrap_json();

    assert_eq!(best_borrowing_offer1.unwrap().value,  150);

    
    //with match

    // ross_account.call(
    //     lending_account.account_id(),
    //     "post_borrowing_offer",
    //     &json!({
    //         "nft_collection_id": nft_collection_account.account_id(), 
    //         "value_offered": U128(100),
    //         "collateral_nft": "token_id".to_string(),
    //         "nft_owner_id": ross_account.account_id(),
    //     }).to_string().into_bytes(),
    //     GAS_ATTACHMENT, 
    //     0
    // ).assert_success();

    // let best_borrowing_offer2: Option<Offer> = root.call(
    //     lending_account.account_id(),
    //     "get_best_borrowing_offer",
    //     &json!({
    //         "nft_collection_id": nft_collection_account.account_id()
    //     }).to_string().into_bytes(),
    //     GAS_ATTACHMENT, 
    //     0
    // ).unwrap_json();

    // assert_eq!(best_borrowing_offer2.unwrap().value,  150);

    // let best_lending_offer3: Option<Offer> = root.call(
    //     lending_account.account_id(),
    //     "get_best_lending_offer",
    //     &json!({
    //         "nft_collection_id": nft_collection_account.account_id()
    //     }).to_string().into_bytes(),
    //     GAS_ATTACHMENT, 
    //     0
    // ).unwrap_json();

    // assert_eq!(best_lending_offer3.unwrap().value,  110);

    // choose specific lending offer



    // choose specific borrowing offer


    // get best lending offer

    // get best borrowing offer

    // cancel specific lending offer

    // cancel specific borrowing offer

}