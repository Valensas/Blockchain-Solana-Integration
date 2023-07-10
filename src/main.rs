#[macro_use] extern crate rocket;
use rocket::http::Status;
use serde::Deserialize;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;

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

#[post("/greet", data = "<request>")] //JSONu alıyor, GreetingRequest'e dönüştürüyor, sonra ismi basıyor
fn greet_json(request: &str) -> String {
    let g: GreetingRequest = serde_json::from_str(request).unwrap();
    format!("Hello {}\n", g.name)
}

#[derive(Debug,Serialize,Deserialize)]
struct Block{
    height: u64,
    hash: String
}
impl Block {
    fn new() -> Self {
        Self {
            height: 0,
            hash: String::from("0"),
        }
    }
}
#[get("/blocks/latest")]
fn get_latest_block() -> String{
    let rpc_url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(rpc_url.to_string());
    let block_height = rpc_client.get_block_height().unwrap();
    let mut block: Block = Block::new();

    block.height = block_height;
    
    match rpc_client.get_slot() {
        Ok(slot) => {
            let block_hash: Result<solana_transaction_status::EncodedConfirmedBlock, String> = match rpc_client.get_block(slot) {
                Ok(hash) => hash.to_string(),
                Err(_) => return String::from("Error"),
            };
            block.hash = block_hash;

            let json= serde_json::to_string(&block).unwrap();
            json
        },
        Err(_) => String::from("Error"),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, greet, greet_json, get_latest_block])
}