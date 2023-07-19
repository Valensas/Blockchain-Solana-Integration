#[macro_use] extern crate rocket;

use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use rocket::serde::json::Json;
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
    rocket::build().mount("/", routes![hello, greet, greet_json, get_latest_block, get_wallet_balance])
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


#[derive(Debug, Serialize, Deserialize)]
struct Balance {
    balance: u64
}

#[get("/address/<address>/balance")]
fn get_wallet_balance(address: &str) -> Result<Json<Balance>, String>{
    let rpc_url: String = "https://api.devnet.solana.com".to_string();
    let rpc_client: RpcClient = RpcClient::new(rpc_url);

    let mut pubkey_bytes_array: [u8;32] = [0;32]; // 32 elemanlı array oluşturmam gerekiyor çünkü Pubkey::new_from_array fonksiyonu kesin olarak bunu istiyor
    let pubkey_bytes_vec = bs58::decode(address).into_vec().unwrap(); // Bu vektörün içindekileri arraye koyuyorum aşağıdaki döngüde

    for byte in 0..pubkey_bytes_vec.len(){
        pubkey_bytes_array[byte] = pubkey_bytes_vec[byte];
    }

    let pubkey = Pubkey::new_from_array(pubkey_bytes_array); // Pubkey objesi yaratılıyor

    let balance = rpc_client.get_balance(&pubkey).unwrap(); // O pubkeyin balanceı alınıyor

    let response: Balance = Balance{balance};

    Ok(Json(response))
}
