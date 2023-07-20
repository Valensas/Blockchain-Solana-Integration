#[macro_use] extern crate rocket;

use std::sync::Arc;

use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::transaction::Transaction;
use rocket::serde::json::Json;

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
    .mount("/", routes![hello, greet, greet_json, get_latest_block, send_transaction])
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
) -> String {
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

/*/************************************* DOĞA ****************************************************/

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
fn scan_block_transactions_from_slot(slot: u64,  rpc_client: &State<Arc<RpcClient>>) -> String {

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
fn scan_block_transactions_from_height(block_height: u64,  rpc_client: &State<Arc<RpcClient>>) -> String {

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
/************************************* END DOĞA ****************************************************/*/


//*************************************************ONUR SEND TRANSACTION********************************************************
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

//*************************************************ONUR SEND TRANSACTION********************************************************