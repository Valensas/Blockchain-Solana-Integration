use crate::{errors::ResponseError, models::Balance};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use rocket::{State, serde::json::Json};
use bs58;
use solana_sdk::pubkey::Pubkey;

#[get("/address/<address>/balance")]
pub fn get_wallet_balance(address: &str, rpc_client: &State<Arc<RpcClient>>) -> Result<Json<Balance>, ResponseError>{

    let pubkey_bytes_vec = bs58::decode(address).into_vec()
    .map_err(|err| {
        log::error!("Failed during creating pubkey bytes vector: {}", err);
        ResponseError::CreateByteArrayError { code: "Failed during creating the byte vector of public key".to_string() }
    })?;

    let pubkey_bytes_array: [u8;32] = pubkey_bytes_vec.try_into()
    .map_err(|err| {
        log::error!("Failed during creating pubkey bytes array: {:?}", err);
        ResponseError::CreateByteArrayError { code: "Failed during creating the byte array of public key".to_string() }
    })?;


    let pubkey = Pubkey::new_from_array(pubkey_bytes_array);

    let balance = rpc_client.get_balance(&pubkey)
    .map_err(|err| {
        log::error!("Failed during getting the balance: {}", err);
        ResponseError::GetBalanceError { code : "Failed during getting the balance of a wallet".to_string() }
    })?;

    let response: Balance = Balance{balance};

    Ok(Json(response))
}