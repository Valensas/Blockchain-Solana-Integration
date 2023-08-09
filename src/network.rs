use std::sync::Arc;
use rocket::{State, serde::json::Json};
use crate::{errors::{ResponseError, Code}, models::ContractResponse};
use spl_token::instruction::transfer;
use solana_program::instruction::Instruction;
use solana_sdk::{message::Message, pubkey::Pubkey};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;

fn calculate_fee(instruction: Instruction, rpc_client: Arc<RpcClient>, sender_address: Option<&Pubkey>) -> Result<Json<ContractResponse>, ResponseError>{
    let instructions: [Instruction; 1] = [instruction];
    let message: Message = Message::new_with_blockhash(&instructions, sender_address, &rpc_client.get_latest_blockhash().unwrap());
    let calculated_fee = rpc_client.get_fee_for_message(&message)
    .map_err(|err| {
        log::error!("Error while getting the fee: {}", err);
        ResponseError::GetFeeError(Json(Code {code: "Error while getting the fee".to_string()}))
    })?;
    Ok(Json(ContractResponse { calculated_fee }))
}

#[get("/fee/estimate?<contract>")]
pub fn get_calculated_fee(
    contract: Option<String>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<ContractResponse>, ResponseError> {
    let sender_address: Pubkey = Pubkey::new_unique();
    let to_address: Pubkey = Pubkey::new_unique();

    match contract{
        Some(contract_str) => {
            let contract_address: Pubkey = Pubkey::from_str(&contract_str)
            .map_err(|err| {
                log::error!("Error while creating the Pubkey object from the contract address: {}", err);
                ResponseError::CreatePubkeyError (Json(Code{code: "Error while creating the Pubkey object from the contract address".to_string()}))
            })?;


            let instruction: Instruction = transfer(
            &spl_token::id(),
            &contract_address,
            &to_address,
            &sender_address,
            &[&sender_address],
            1)
            .map_err(|err| {
                log::error!("Error while creating the transfer: {}", err);
                ResponseError::CreateTransferError(Json(Code{code: "Error while creating the transfer".to_string()}))
            })?;
            calculate_fee(instruction, rpc_client.inner().clone(), Some(&sender_address))
        },
        None => {
            let instruction: Instruction = solana_sdk::system_instruction::transfer(&sender_address, &to_address, 100000000);
            calculate_fee(instruction, rpc_client.inner().clone(), None)
        }
    }
}
