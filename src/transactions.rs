use std::{sync::Arc, str::FromStr};

use rust_base58::FromBase58;
use rocket::{State, serde::json::Json};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_sdk::{signature::Signature, transaction::Transaction, pubkey::Pubkey, signature::Keypair, commitment_config::CommitmentConfig};
use solana_transaction_status::{UiTransactionEncoding, TransactionDetails};
use spl_token::instruction::transfer;
use solana_program::instruction::Instruction;
use crate::{errors::{ResponseError, Code}, models::{TransactionInfoConvertiable, SendTransactionRequest, SendTransactionResponse, SignTransactionRequest, SignTransactionResponse, TransactionInfo, ConfirmationCount}};

#[post("/transactions/sign", data = "<transaction_parameters>")]
pub fn sign_transaction(
    transaction_parameters: Json<SignTransactionRequest>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<SignTransactionResponse>, ResponseError> {
    
    if transaction_parameters.from.is_empty(){
        return Err(ResponseError::EmptyError(Json(Code{code : "From part of the request is empty".to_string()})));
    }

    if transaction_parameters.to.is_empty(){
        return Err(ResponseError::EmptyError(Json(Code{code : "To part of the request is empty".to_string()})));
    }

    let sender_address = Pubkey::from_str(&transaction_parameters.from[0].adress)
    .map_err(|err| {
        log::error!("Error during creating the Pubkey object from the sender address: {}", err);
        ResponseError::CreatePubkeyError(Json(Code{code : "Failed during creating the Pubkey object from the sender address".to_string()}))
    })?;

    let privkey = transaction_parameters.private_key.clone();

    let mut bytes_of_privatekey = privkey.from_base58().map_err(|err|{
        log::error!("Error during creating the byte array of private key: {}", err);
        ResponseError::CreateByteArrayError(Json(Code{code: "Failed during creating the byte array of private key".to_string()}))
    })?;

    let mut bytes_of_publickey = transaction_parameters.from[0].adress.from_base58().map_err(|err|{
        log::error!("Error during creating the byte array of public key: {}", err);
        ResponseError::CreateByteArrayError(Json(Code{code: "Failed during creating the byte array of public key".to_string()}))
    })?;

    bytes_of_privatekey.append(& mut bytes_of_publickey);

    let keypair: Keypair = Keypair::from_bytes(&bytes_of_privatekey)
    .map_err(|err|{
        log::error!("Error during creating the keypair object: {}", err);
        ResponseError::CreateKeypairError(Json(Code{code: "Failed during creating the keypair object".to_string()}))
    })?;

    let blockhash = rpc_client.get_latest_blockhash()
    .map_err(|err| {
        log::error!("Error while getting the latest confirmed blockhash: {}", err);
        ResponseError::GetBlockhashError(Json(Code{code: "Failed during getting the latest confirmed blockhash".to_string()}))
    })?;

    let instructions: Vec<Instruction> = transaction_parameters.to.iter()
        .map(|transfer_param| {
            let to_address = Pubkey::from_str(&transfer_param.adress)
                .map_err(|err| {
                    log::error!("Error during creating the Pubkey object from the receiver address: {}", err);
                    ResponseError::CreatePubkeyError (Json( Code {code: "Failed during creating the Pubkey object from the receiver address".to_string() }))
                })?;
                
            
            let amount = &transfer_param.amount;

            match transfer_param.contract.as_ref() {
                Some(contract_str) => {

                    let contract = Pubkey::from_str(contract_str)
                        .map_err(|err| {
                            log::error!("Error during creating the Pubkey object from the contract address: {}", err);
                            ResponseError::CreatePubkeyError (Json(Code{ code: "Failed during creating the Pubkey object from the contract address".to_string() }))
                        })?;
                    


                    let instruction = transfer(&spl_token::id(), 
                    &contract, 
                    &to_address, 
                    &sender_address,
                    &[&sender_address],
                    *amount as u64)
                        .map_err(|err| {
                            log::error!("Error during creating the transaction instruction: {}", err);
                            ResponseError::CreateTransferError (Json(Code{ code: "Failed during creating the transaction instruction".to_string() }))
                        })?;
                    Ok(instruction)
                }
                None => Ok(solana_sdk::system_instruction::transfer(&sender_address, &to_address, *amount as u64))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
        
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&sender_address),
            &[&keypair],
            blockhash
        );
        
        let signatures = &tx.signatures;
        let txn_hash = signatures[0].to_string();
    
        let signed_transaction = serde_json::to_string(&tx)
        .map_err(|err| {
            log::error!("Error during converting the Transaction object to String: {}", err);
            ResponseError::ConvertTransactionError (Json(Code{ code: "Failed during converting Transaction object to String".to_string() }))
        })?;
        let response: SignTransactionResponse = SignTransactionResponse{
            txn_hash,
            signed_transaction
        };
        
        Ok(Json(response))

}

#[get("/transactions/<txn_hash>/detail")]
pub fn get_transaction_details(
    rpc_client: &State<Arc<RpcClient>>,
    txn_hash: &str
) -> Result<Json<TransactionInfo>, ResponseError> {

    let signature = Signature::from_str(txn_hash)
        .map_err(|err| {
            log::error!("Failed during converting txnHash (&str) to Signature: {}", err);
            ResponseError::StrToSignatureError (Json(Code{ code: "Failed during parsing signature".to_string() }))
        })?;
    
    let conf_transaction = rpc_client.get_transaction(&signature, UiTransactionEncoding::Json)
        .map_err(|err| {
            log::error!("Failed during getting the transaction with given hash: {}", err);
            ResponseError::GetTransactionError (Json(Code{ code: "Failed during getting the transaction with given hash".to_string() }))
        })?;

    let block_slot = conf_transaction.slot;
    let block_hash = rpc_client.get_block_with_config(block_slot, RpcBlockConfig {encoding: Some(UiTransactionEncoding::Json), transaction_details: Some(TransactionDetails::Full), rewards: Some(false), commitment: Some(CommitmentConfig::finalized()), max_supported_transaction_version: Some(0)})
        .map(|block| block.blockhash)
        .map_err(|err| {
            log::error!("Failed during getting the block with given slot: {}", err);
            ResponseError::GetBlockError (Json(Code{ code: "Failed during getting the block with given slot".to_string() }))
        })?;

    let transaction_meta = conf_transaction.transaction;

    let transaction = transaction_meta.to_transaction_info(block_slot, &block_hash)?;
    Ok(Json(transaction))
}

#[post("/transactions/send", data = "<transaction_parameters>")]
pub fn send_transaction(
    transaction_parameters: Json<SendTransactionRequest>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<SendTransactionResponse>, ResponseError> {

    let tx = serde_json::from_str::<Transaction>(&transaction_parameters.signed_transaction)
        .map_err(|err|{
            log::error!("Error while creating the transaction object: {}", err);
            ResponseError::CreateTransactionError (Json(Code{ code: "Failed during creating the transaction object".to_string() }))
        })?;

    rpc_client
        .send_and_confirm_transaction(&tx)
        .map(|txn_hash|
            Json(SendTransactionResponse{
                txn_hash: txn_hash.to_string()
            })
        )
        .map_err(|err| {
            log::error!("Error while sending the transaction: {}", err);
            ResponseError::SendTransactionError (Json(Code{ code: "Failed during sending the transaction".to_string() }))
        })
}

#[get("/transactions/<txn_hash>/confirmations")]
pub fn get_confirmation_count(    
    rpc_client: &State<Arc<RpcClient>>,
    txn_hash: &str
) -> Result<Json<ConfirmationCount>, ResponseError> {
    let signature = Signature::from_str(txn_hash)
        .map_err(|err| {
            log::error!("Failed during converting txnHash (&str) to Signature: {}", err);
            ResponseError::StrToSignatureError (Json(Code{ code: "Failed during parsing signature".to_string() }))
        })?;
    
    let block_slot = rpc_client.get_transaction(&signature, UiTransactionEncoding::Json)
        .map(|transaction| transaction.slot)
        .map_err(|err| {
            log::error!("Failed during getting the transaction with given hash: {}", err);
            ResponseError::GetTransactionError (Json(Code{ code: "Failed during getting the transaction with given hash".to_string() }))
        })?;
    
    let block = rpc_client.get_block_with_config(block_slot, RpcBlockConfig {encoding: Some(UiTransactionEncoding::Json), transaction_details: Some(TransactionDetails::Full), rewards: Some(false), commitment: Some(CommitmentConfig::finalized()), max_supported_transaction_version: Some(0)})
        .map_err(|err| {
            log::error!("Failed during getting the block with given slot: {}", err);
            ResponseError::GetBlockError (Json(Code{ code: "Failed during getting the block with given slot".to_string() }))
        })?;

    let block_height = match block.block_height {
        Some(height) => {
            height
        },
        None => {
            return Err(ResponseError::GetBlockHeightError (Json(Code{ code: "Failed during getting the height of the given block".to_string() })));
        }
    };

    let latest_block_height = rpc_client.get_block_height()
        .map_err(|err| {
            log::error!("Failed during getting the latest block height: {}", err);
            ResponseError::GetBlockHeightError (Json(Code{ code: "Failed during getting the latest block height".to_string() }))
        })?;

    Ok(Json(ConfirmationCount { 
        confirmations_count: latest_block_height - block_height
    }))

}