#![allow(non_snake_case)]
#[macro_use] extern crate rocket;

use std::sync::Arc;

use rocket::State;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use rocket::serde::json::Json;
use solana_sdk::signature::Keypair;
use bs58;
use solana_sdk::transaction::Transaction;

#[derive(Responder)]
enum ResponseErrors {
    #[response(status = 500, content_type = "json")]
    SendTransactionError {
        code: String
    },
    #[response(status = 500, content_type = "json")]
    CreateTransactionError{
        code: String
    }
}

#[rocket::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    let rocket = match rocket::build()
    .mount("/", routes![get_latest_block, send_transaction, create_wallet_address])
    .manage(rpc_client).ignite().await {
        Ok(rocket) => {
            log::info!("Server started gracefully");
            rocket
        },
        Err(err) => {
            log::error!("Server could not start gracefully: {}", err);
            return;
        },
    };

    // End State
    match rocket.launch().await {
        Ok(_) => {
            log::info!("Server closed gracefully");
        },
        Err(err) => {
            log::error!("Server could not close gracefully: {}", err);
        },
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    height: u64,
    hash: String
}

#[get("/blocks/latest")]
fn get_latest_block(
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<Block>, String> {
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
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct WalletResponse {
    address: String,
    privateKey: String
}

#[post("/address")]
fn create_wallet_address() -> Result<Json<WalletResponse>, ResponseErrors>{
    let keypair = Keypair::new();
    let byte_array = keypair.to_bytes();
    let key_length = 32;
    let address = bs58::encode(&byte_array[key_length..]).into_string();
    let private_key = bs58::encode(&byte_array[0..key_length]).into_string();

    let response = WalletResponse { address, privateKey: private_key};

    Ok(Json(response))
}
//********************************* Onur Create Wallet Address **************************************************
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct SendTransactionRequest {  // Request için obje oluşturuldu
    signedTransaction: String,
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct SendTransactionResponse { // Response için obje oluşturuldu
    txnHash: String,
}

#[post("/transactions/send", data = "<transaction_parameters>")]
fn send_transaction(
    transaction_parameters: Json<SendTransactionRequest>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<SendTransactionResponse>, ResponseErrors>{

    let tx: Transaction = match serde_json::from_str::<Transaction>(&transaction_parameters.signedTransaction){
        Ok(tx) =>{
            tx
        },
        Err(_) => {
            return Err(ResponseErrors::CreateTransactionError { code: "Failed during creating the transaction object".to_string() });
        }
    };

    
    match rpc_client.send_and_confirm_transaction(&tx) {
        Ok(txn_hash) => { // Signature'ı Stringe çevir
            let response: SendTransactionResponse = SendTransactionResponse{ // Response objesi oluşturuluyor
                txnHash: txn_hash.to_string()
            };
            
            return Ok(Json(response));
        },
        Err(_) => {
            return Err(ResponseErrors::SendTransactionError { code: "Failed during sending the transaction".to_string() });
        }
    }
}

