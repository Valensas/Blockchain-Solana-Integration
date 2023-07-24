use crate::models::WalletResponse;
use rocket::serde::json::Json;
use solana_sdk::signature::Keypair;

#[post("/address")]
pub fn create_wallet_address() -> Json<WalletResponse>{
    let keypair = Keypair::new();
    let byte_array = keypair.to_bytes();
    let key_length = 32;
    let address = bs58::encode(&byte_array[key_length..]).into_string();
    let private_key = bs58::encode(&byte_array[0..key_length]).into_string();

    let response = WalletResponse { address, private_key };

    Json(response)
}