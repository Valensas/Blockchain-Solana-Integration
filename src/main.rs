#![allow(non_snake_case)]
#[macro_use] extern crate rocket;

use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use rocket::serde::json::Json;
use solana_sdk::signature::Keypair;
use bs58;
/*use solana_transaction_status::{
    EncodedConfirmedBlock, EncodedTransactionWithStatusMeta, EncodedConfirmedTransactionWithStatusMeta, TransactionStatus,
    UiConfirmedBlock, UiTransactionEncoding,
};*/

#[get("/hello")]
fn hello() -> &'static str {
    "Hello world\n"
}

#[get("/greet?<name>")] // Name'e göre printliyor
fn greet(name: &str) -> String {
    format!("Hello {}\n", name)
}

#[derive(Debug, Deserialize)]
struct GreetingRequest {
    name: String
}

#[post("/greet", data = "<request>")] // JSONu alıyor, GreetingRequest'e dönüştürüyor, sonra ismi basıyor
fn greet_json(request: &str) -> String {
    let g: GreetingRequest = serde_json::from_str(request).unwrap();
    format!("Hello {}\n", g.name)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, greet, greet_json, get_latest_block, create_wallet_address])
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    height: u64,
    hash: String
}

#[get("/blocks/latest")]
fn get_latest_block() -> Result<Json<Block>, String> {
    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = RpcClient::new(rpc_url);
    let commitment_config = CommitmentConfig::finalized(); //TODO: İşlenmesi bitmiş block alınsın diye eklendi

    match rpc_client.get_slot_with_commitment(commitment_config) { // Son block slotunu alıp matchledik
        Ok(slot) => {
            let block = rpc_client.get_block(slot).unwrap();

            let block: Block = Block { // Block objesi yaratıldı
                height: block.block_height.unwrap(),
                hash: block.blockhash
            };

            Ok(Json(block))
            //serde_json::from_str(&block) // Block objesi JSON stringine dönüştürüldü ve returnlendi
        }
        Err(_) => Err(String::from("Slot not found"))
    }
}

//********************************* Onur Create Wallet Address **************************************************
#[derive(Debug, Serialize, Deserialize)]
struct WalletResponse {
    address: String,
    privateKey: String
}

#[post("/address")]
fn create_wallet_address() -> Result<Json<WalletResponse>, String>{
    let keypair = Keypair::new();
    let byte_array = keypair.to_bytes();
    let address = bs58::encode(&byte_array[32..64]).into_string();
    let privateKey = bs58::encode(&byte_array[0..32]).into_string();

    let response = WalletResponse { address, privateKey};

    Ok(Json(response))
}
//********************************* Onur Create Wallet Address **************************************************