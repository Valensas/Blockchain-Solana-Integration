#[macro_use] extern crate rocket;

use std::sync::Arc;

use rocket::State;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use rocket::serde::json::Json;
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
    },
    #[response(status = 500, content_type = "json")]
    CreateByteArrayError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    GetBalanceError{
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
    .mount("/", routes![get_latest_block, send_transaction, get_wallet_balance])
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


#[derive(Debug, Serialize, Deserialize)]
struct Balance {
    balance: u64
}

#[get("/address/<address>/balance")]
fn get_wallet_balance(address: &str, rpc_client: &State<Arc<RpcClient>>) -> Result<Json<Balance>, ResponseErrors>{

    let mut pubkey_bytes_array: [u8;32] = [0;32]; // 32 elemanlı array oluşturmam gerekiyor çünkü Pubkey::new_from_array fonksiyonu kesin olarak bunu istiyor
    let pubkey_bytes_vec = match bs58::decode(address).into_vec(){
        Ok(pubkey_bytes_vec) => pubkey_bytes_vec,
        Err(_) => return Err(ResponseErrors::CreateByteArrayError { code: "Failed during creating the byte array of public key".to_string() })
    }; // Bu vektörün içindekileri arraye koyuyorum aşağıdaki döngüde


    for byte in 0..pubkey_bytes_vec.len(){
        pubkey_bytes_array[byte] = pubkey_bytes_vec[byte];
    }

    let pubkey = Pubkey::new_from_array(pubkey_bytes_array); // Pubkey objesi yaratılıyor

    let balance = match rpc_client.get_balance(&pubkey){
        Ok(balance) => balance,
        Err(_) => return Err(ResponseErrors::GetBalanceError { code : "Failed during getting the balance of a wallet".to_string() })
    }; // O pubkeyin balanceı alınıyor

    let response: Balance = Balance{balance};

    Ok(Json(response))
}
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

