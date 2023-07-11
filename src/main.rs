#[macro_use] extern crate rocket;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use serde_json::{Result, Value};

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
    rocket::build().mount("/", routes![hello, greet, greet_json, get_latest_block])
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    height: u64,
    hash: String
}

#[get("/blocks/latest")]
fn get_latest_block() -> String {
    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = RpcClient::new(rpc_url);
    let commitment_config = CommitmentConfig::finalized(); //TODO: İşlenmesi bitmiş block alınsın diye eklendi
    
    match rpc_client.get_slot_with_commitment(commitment_config) { // Son block slotunu alıp matchledik
        Ok(slot) => {
            let block_str = serde_json::to_string(&rpc_client.get_block(slot).unwrap()).unwrap(); // Son blockun bilgileri stringe dönüştürüldü
            let block_json: Value = serde_json::from_str(&block_str).unwrap(); // Son blockun bilgileri JSONa dönüştürüldü
            let block_height: u64 = block_json["blockHeight"].to_string().parse::<u64>().unwrap(); // BlockHeight alındı
            let block_hash = block_json["blockhash"].to_string(); // Blockhash alındı
            let block: Block = Block { // Block objesi yaratıldı
                height: block_height,
                hash: block_hash
            };
            serde_json::to_string(&block).unwrap() // Block objesi JSON stringine dönüştürüldü ve returnlendi
        }
        Err(_) => String::from("Slot not found")
    }
}