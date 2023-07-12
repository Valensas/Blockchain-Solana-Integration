#[macro_use] extern crate rocket;

use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use rust_decimal::prelude::*;
use rocket::serde::json::Json;
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
    rocket::build().mount("/", routes![hello, greet, greet_json, get_latest_block, scan_block_transactions_from_slot])
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

/************************************* DOĞA ****************************************************/

#[derive(Debug, Serialize, Deserialize)]
struct TransactionInfo { // Onur ile ortak
    adress: String,
    //#[serde(with = "rust_decimal::serde::str")]
    amount: Decimal
}

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    from: TransactionInfo,
    to: TransactionInfo,
    hash: String,
    status: String,
    contract: String
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockTransactions {
    height: u64,
    hash: String,
    transactions: Vec<Transaction>
}

#[get("/blocks/<slot>")]
fn scan_block_transactions_from_slot(slot: u64) -> String {
    let rpc_url = "https://api.devnet.solana.com".to_string(); // Solana url
    let rpc_client = RpcClient::new(rpc_url);

    //let commitment_config = CommitmentConfig::finalized(); //TODO: burada gerek var mı? kullanmadım şimdilik

    match rpc_client.get_block(slot) { //Verilen slot'a göre chainden block bilgileri alınıyor 
        Ok(block) => {
            let block_hash = block.blockhash;
            let block_height = block.block_height.unwrap();
            //let transactions: Vec<EncodedTransactionWithStatusMeta> = block.transactions;

            "hey".to_string()
            //transactions.into_iter().to_string().collect::<String>()

            /*let block_str = serde_json::to_string(&block).unwrap(); // Son blockun bilgileri stringe dönüştürüldü
            let block_json: Value = serde_json::from_str(&block_str).unwrap(); // Son blockun bilgileri JSONa dönüştürüldü

            let block_height: u64 = block_json["blockHeight"].to_string().parse::<u64>().unwrap(); // BlockHeight alındı
            let block_hash = block_json["blockhash"].to_string(); // Blockhash alındı

            block_json["transactions"].to_string()*/
        }
        Err(_) => String::from("Block with given slot not found!\n")
    }
}

/*#[get("/blocks/<block_height>", rank = 1)] // rank = 1: önce u64'e cast edebiliyor mu baksın (block height), edemezse string alsın (block hash)
fn scan_block_transactions_from_height(block_height: u64) -> String {
    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = RpcClient::new(rpc_url);
    let commitment_config = CommitmentConfig::finalized(); //TODO: burada gerek var mı?
    
    "hey".to_string()

    match rpc_client.get_confirmed_blocks_with_commitment(block_height, commitment_config) { // Son block slotunu alıp matchledik
        Ok(block) => {
            let block_str = serde_json::to_string(&block).unwrap(); // Son blockun bilgileri stringe dönüştürüldü
            let block_json: Value = serde_json::from_str(&block_str).unwrap(); // Son blockun bilgileri JSONa dönüştürüldü

            block_json.to_string()
        }
        Err(_) => String::from("Slot not found")
    }
}*/

/*#[get("/blocks/<block_hash>", rank = 2)]
fn scan_block_transactions_from_hash(block_hash: String) -> String {
    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = RpcClient::new(rpc_url);
    let commitment_config = CommitmentConfig::finalized(); //TODO: burada gerek var mı?
    
    "hey".to_string()

    match rpc_client.get_confirmed_blocks_with_commitment(&block_hash, commitment_config) { // Son block slotunu alıp matchledik
        Ok(block) => {
            let block_str = serde_json::to_string(&block).unwrap(); // Son blockun bilgileri stringe dönüştürüldü
            let block_json: Value = serde_json::from_str(&block_str).unwrap(); // Son blockun bilgileri JSONa dönüştürüldü

            block_json.to_string()
        }
        Err(_) => String::from("Slot not found")
    }
}*/
/************************************* END DOĞA ****************************************************/