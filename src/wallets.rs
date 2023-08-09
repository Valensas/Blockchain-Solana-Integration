use crate::{errors::{ResponseError, Code}, models::{Balance, WalletResponse}};
use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use std::sync::Arc;
use rocket::{State, serde::json::Json};
use bs58;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;
use solana_account_decoder::UiAccountData;

#[get("/address/<address>/balance?<contract>")]
pub fn get_wallet_balance(address: &str, contract: Option<String>,rpc_client: &State<Arc<RpcClient>>) -> Result<Json<Balance>, ResponseError>{

    let pubkey = Pubkey::from_str(address)
            .map_err(|err| {
            log::error!("Error while creating the Pubkey object from owner address: {}", err);
            ResponseError::CreatePubkeyError(Json(Code{code: "Error while creating the Pubkey object from owner address".to_string()}))
    })?;

    let balance: f64 = match contract {
        Some(c_address) =>{

            let contract_address = Pubkey::from_str(&c_address)
                .map_err(|err| {
                log::error!("Error while creating the Pubkey object from contract address: {}", err);
                ResponseError::CreatePubkeyError(Json(Code{code: "Error while creating the Pubkey object from contract address".to_string()}))       
            })?;
            
            
            let rpc_account = rpc_client.get_token_accounts_by_owner(&pubkey, TokenAccountsFilter::Mint(contract_address))
                        .map_err(|err| {
                        log::error!("Failed during getting the balance: {}", err);
                        ResponseError::GetBalanceError (Json(Code{ code : "Failed during getting the balance of a wallet".to_string() }))
                    })?;

            if rpc_account.is_empty(){
                0 as f64
            }
            
            else{
                log::info!("len vec: {}", rpc_account.len());

                let parsed_account = match rpc_account[0].account.data.clone(){
                    UiAccountData::Binary(_, _) => {
                        return Err(ResponseError::UiAccountDataTypeError(Json(Code{ code: "UiAccountData type Binary not implemented".to_string() })));
                    },
                    UiAccountData::LegacyBinary(_) => {
                        return Err(ResponseError::UiAccountDataTypeError(Json(Code{ code: "UiAccountData type LegacyBinary not implemented".to_string() })));
                    },
                    UiAccountData::Json(parsed_account) => parsed_account
                };

                match parsed_account.parsed.get("info"){
                    Some(info) => {
                        match info.get("tokenAmount"){
                            Some(token_amount) => {
                                match token_amount.get("uiAmount"){
                                    Some(ui_amount) => {
                                        match ui_amount.as_f64(){
                                            Some(amount) => amount,
                                            None => {
                                                log::error!("Error while converting UiAmount to f64");
                                                return Err(ResponseError::ConvertUiAmountError(Json(Code{code: "Error while converting UiAmount to f64".to_string()})));    
                                            }
                                        }
                                    },
                                    None => {
                                        log::error!("Error: couldn't get the uiAmount from the parsed account.");
                                        return Err(ResponseError::EmptyError(Json(Code{code: "Error: couldn't get the uiAmount from the parsed account.".to_string()})));
                                    }
                                }
                            },
                            None => {
                                log::error!("Error: couldn't get the tokenAmount from the parsed account.");
                                return Err(ResponseError::EmptyError(Json(Code{code: "Error while converting tokenAmount to f64".to_string()})));
                            }
                        }
                    },
                    None => {
                        log::error!("Error: couldn't get the info from the parsed account.");
                        return Err(ResponseError::EmptyError(Json(Code{code: "Error: couldn't get the info from the parsed account.".to_string()})));
                    }
                }
            }
        },
        None => {
            rpc_client.get_balance(&pubkey)
                        .map_err(|err| {
                        log::error!("Failed during getting the balance: {}", err);
                        ResponseError::GetBalanceError (Json(Code{ code : "Failed during getting the balance of a wallet".to_string() }))
            })? as f64
        }
    };

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