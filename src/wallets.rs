use crate::{errors::ResponseError, models::Balance};
use rocket::serde::json::Json;
use bs58;
use solana_sdk::pubkey::Pubkey;

#[get("/address/<address>/balance")]
fn get_wallet_balance(address: &str, rpc_client: &State<Arc<RpcClient>>) -> Result<Json<Balance>, ResponseErrors>{

    let mut pubkey_bytes_array: [u8;32] = [0;32];
    let pubkey_bytes_vec = bs58::decode(address).into_vec()
    .map_err(|err| {
        log::error!("Failed during decoding the address: {}", err);
        ResponseErrors::CreateByteArrayError { code: "Failed during creating the byte array of public key".to_string() }
    })?;


    for byte in 0..pubkey_bytes_vec.len(){
        pubkey_bytes_array[byte] = pubkey_bytes_vec[byte];
    }

    let pubkey = Pubkey::new_from_array(pubkey_bytes_array);

    let balance = rpc_client.get_balance(&pubkey)
    .map_err(|err| {
        log::error!("Failed during getting the balance: {}", err);
        ResponseErrors::GetBalanceError { code : "Failed during getting the balance of a wallet".to_string() }
    })?;

    let response: Balance = Balance{balance};

    Ok(Json(response))
}