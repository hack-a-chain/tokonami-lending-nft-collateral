use std::convert::TryInto;
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U128, U64};
pub use near_sdk::serde_json::{json, value::Value};
pub use near_sdk_sim::{call, view, deploy, init_simulator, to_yocto, UserAccount, 
    ContractAccount, DEFAULT_GAS, ViewResult, ExecutionResult};
pub use near_sdk::AccountId;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    LENDING_CONTRACT => "./target/wasm32-unknown-unknown/release/classy_kangaroo_coin_flip.wasm",
    NFT_CONTRACT => "./target/wasm32-unknown-unknown/release/classy_kangaroo_coin_flip.wasm"
}

use std::convert::TryFrom;
pub use rand::Rng;


const NFT_FEE: u128 = 4_000;
const OWNER_FEE: u128 = 500;
const HOUSE_FEE: u128 = 500;
const PARTNER_FEE: u128 = 100;
const FRACTIONAL_BASE: u128 = 100_000;
const MIN_BALANCE_FRACTION: u128 = 100;

const GAS_ATTACHMENT: u64 = 300_000_000_000_000;

mod xpto
#[test]
fn simulate_full_flow() {
    //Test full flow from deploying app
    //deploys coin contract
    //3 different partnered games are created
    //users deposit and play in one game
    //asserts that deposit play and withdraw functions are working as expected
    //asserts no state spill over from one game to another
    //gets fee balances and withdraw to owners

    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = 300_000_000_000_000;
    genesis.gas_price = 0;

    let root = init_simulator(Some(genesis));

    let owner_account_lending = root.create_user("owner_account".to_string(), to_yocto("100"));
    let owner_account_nft_collection = root.create_user("owner_account".to_string(), to_yocto("100"));

   //deploy contracts
    let lending_account = root.deploy(
        &LENDING_CONTRACT,
        "leding_contract".to_string(),
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
            "metadata": metadata
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    owner_account_lending.call(
        nft_receipt_account.account_id(), 
        "new", 
        &json!({
            "owner_id": lending_account.account_id(), 
            "metadata": metadata
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

    owner_account_lending.call(
        nft_collection_account.account_id(), 
        "new", 
        &json!({
            "owner_id": owner_account_nft_collection.account_id(), 
            "metadata": metadata
        }).to_string().into_bytes(),
        GAS_ATTACHMENT, 
        0
    ).assert_success();

}
