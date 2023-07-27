use crate::{errors::ResponseError, models::{Balance, WalletResponse}};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use rocket::{State, serde::json::Json};
use bs58;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;

#[get("/address/<address>/balance")]
pub fn get_wallet_balance(address: &str, rpc_client: &State<Arc<RpcClient>>) -> Result<Json<Balance>, Json<ResponseError>>{

    let pubkey = Pubkey::from_str(address)
    .map_err(|err| {
        log::error!("Error while getting the fee: {}", err);
        Json(ResponseError::GetFeeError{code: "Error while getting the fee".to_string()})
    })?;

    let balance = rpc_client.get_balance(&pubkey)
    .map_err(|err| {
        log::error!("Failed during getting the balance: {}", err);
        Json(ResponseError::GetBalanceError { code : "Failed during getting the balance of a wallet".to_string() })
    })?;

    let response: Balance = Balance{balance};

    Ok(Json(response))
}
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