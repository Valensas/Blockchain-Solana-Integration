use std::{sync::Arc, str::FromStr};

use rust_base58::FromBase58;
use rocket::{State, serde::json::Json};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Signature, transaction::Transaction, pubkey::Pubkey, signature::Keypair, commitment_config::CommitmentConfig};
use solana_transaction_status::{EncodedConfirmedBlock, UiTransactionEncoding};
use spl_token::instruction::transfer;
use solana_program::instruction::Instruction;
use crate::{errors::ResponseError, models::{DetailedTransaction, TransactionInfoConvertiable, SendTransactionRequest, SendTransactionResponse, SignTransactionRequest, SignTransactionResponse}, config::SOL_PRECISION};



#[post("/transactions/sign", data = "<transaction_parameters>")]
pub fn sign_transaction(
    transaction_parameters: Json<SignTransactionRequest>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<SignTransactionResponse>, ResponseError> {
    
    if transaction_parameters.from.is_empty(){
        return Err(ResponseError::EmptyError{code : "From part of the request is empty".to_string()});
    }

    if transaction_parameters.to.is_empty(){
        return Err(ResponseError::EmptyError{code : "To part of the request is empty".to_string()});
    }

    let sender_address = Pubkey::from_str(&transaction_parameters.from[0].adress)
    .map_err(|err| {
        log::error!("Error during creating the Pubkey object: {}", err);
        ResponseError::CreatePubkeyError{code : "Failed during creating the Pubkey object".to_string()}
    })?;

    let privkey = transaction_parameters.private_key.clone();

    let mut bytes_of_privatekey = privkey.from_base58().map_err(|err|{
        log::error!("Error during creating the byte array of private key: {}", err);
        ResponseError::CreateByteArrayError{code: "Failed during creating the byte array of private key".to_string()}
    })?;

    let mut bytes_of_publickey = transaction_parameters.from[0].adress.from_base58().map_err(|err|{
        log::error!("Error during creating the byte array of public key: {}", err);
        ResponseError::CreateByteArrayError{code: "Failed during creating the byte array of public key".to_string()}
    })?;

    bytes_of_privatekey.append(& mut bytes_of_publickey);

    let keypair: Keypair = Keypair::from_bytes(&bytes_of_privatekey)
    .map_err(|err|{
        log::error!("Error during creating the keypair: {}", err);
        ResponseError::CreateKeypairError{code: "Failed during creating the keypair".to_string()}
    })?;

    let blockhash = rpc_client.get_latest_blockhash_with_commitment(CommitmentConfig::confirmed()) // Here we use CommitmentConfig::confirmed() to avoid risk of expiring Blockhash
    .map_err(|err| {
        log::error!("Error while getting the latest confirmed blockhash: {}", err);
        ResponseError::GetBlockhashError{code: "Failed during getting the latest confirmed blockhash".to_string()}
    })?;

    let mut instructions: Vec<Instruction> = Vec::new();

    for transfer_param in &transaction_parameters.to{

        let to_address = Pubkey::from_str(&transfer_param.adress)
        .map_err(|err| {
            log::error!("Error during creating the Pubkey object: {}", err);
            ResponseError::CreatePubkeyError { code: "Failed during creating the Pubkey object".to_string() }
        })?;
        
        let amount = &transfer_param.amount;

        match transfer_param.contract {
            Some(_) => {
                let contract_str:&str = match transfer_param.contract.as_ref(){
                    Some(c) => c,
                    None => return Err(ResponseError::EmptyError { code: "Failed during getting the contract address".to_string() })
                };
    
                let contract = Pubkey::from_str(contract_str)
                .map_err(|err| {
                    log::error!("Error during creating the Pubkey object: {}", err);
                    ResponseError::CreatePubkeyError { code: "Failed during creating the Pubkey object".to_string() }
                })?;
                
                let instruction = transfer(&contract, &sender_address, // Instruction (contract adresi verilerek) oluşturulup vektöre pushlanıyor
                    &to_address, &sender_address, &[], *amount as u64)
                    .map_err(|err| {
                        log::error!("Error during creating the transaction instruction: {}", err);
                        ResponseError::CreateTransferError{code : "Failed during creating the transaction instruction".to_string()}
                    })?;
                instructions.push(instruction);
            }
            None => {
                instructions.push(solana_sdk::system_instruction::transfer(&sender_address, &to_address, *amount as u64))
            }
        }
    }

    let tx = Transaction::new_signed_with_payer( // Transaction objesi oluşturuluyor
        &instructions,
        Some(&sender_address),
        &[&keypair],
        blockhash.0
    );
    
    let signatures = &tx.signatures;
    let txn_hash = signatures[0].to_string();

    let signed_transaction = serde_json::to_string(&tx)
    .map_err(|err| {
        log::error!("Error getting latest block: {}", err);
        ResponseError::ConvertTransactionError { code: "Failed during converting Transaction object to String".to_string() }
    })?;
    let response: SignTransactionResponse = SignTransactionResponse{
        txn_hash: txn_hash,
        signed_transaction: signed_transaction
    };
    
    return Ok(Json(response));

}

#[get("/transactions/<txn_hash>/detail")]
pub fn get_transaction_details(
    rpc_client: &State<Arc<RpcClient>>,
    txn_hash: &str
) -> Result<Json<DetailedTransaction>, ResponseError> {

    let signature = match Signature::from_str(txn_hash){
        Ok(signature) => {
            signature
        },
        Err(_) => {
            return Err(ResponseError::StrToSignatureError { code: "Failed during converting txnHash (&str) to Signature".to_string() });
        },
    };

    let conf_transaction = match rpc_client.get_transaction(&signature, UiTransactionEncoding::Json) {
        Ok(conf_transaction) => {
            conf_transaction
        }
        Err(_) => {
            return Err(ResponseError::GetTransactionError { code: "Failed during getting the transaction with given hash".to_string() });
        }
    };

    let block_slot = conf_transaction.slot;
    let block: EncodedConfirmedBlock = match rpc_client.get_block(block_slot) {
        Ok(block) => {
            block
        },
        Err(_) => {
            return Err(ResponseError::GetBlockError { code: "Failed during getting the block with given slot".to_string() });
        }
    };

    let block_hash = block.blockhash;

    let transaction_meta = conf_transaction.transaction;

    /***** Fee haricindeki transaction bilgileri helper metoduyla burada alınıyor *****/
    let transaction = transaction_meta.to_transaction_info()?;

    match transaction_meta.meta {
        Some(meta) => {  // Transaction metası alındı
            /***** Transaction fee burada alınıyor *****/
            let transaction_fee = (meta.fee as f64) / (10_u32.pow(SOL_PRECISION) as f64); // Metanın içinden fee alındı, precisiona göre düzenlendi

            return Ok(Json(DetailedTransaction {
                from: transaction.from,
                to: transaction.to,
                hash: transaction.hash,
                status: transaction.status,
                fee: transaction_fee,
                block_hash: block_hash,
                block_height: block_slot
            }));
        },
        None => {
            return Err(ResponseError::TransactionMetaError { code: "Failed during getting the meta from given transaction".to_string() });
        }
    }
}

#[post("/transactions/send", data = "<transaction_parameters>")]
pub fn send_transaction(
    transaction_parameters: Json<SendTransactionRequest>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<SendTransactionResponse>, ResponseError> {

    let tx = serde_json::from_str::<Transaction>(&transaction_parameters.signed_transaction)
        .map_err(|err|{
            log::error!("Error while creating the transaction object: {}", err);
            ResponseError::CreateTransactionError { code: "Failed during creating the transaction object".to_string() }
        } )?;

    rpc_client
        .send_and_confirm_transaction(&tx)
        .map(|txn_hash|
            Json(SendTransactionResponse{
                txn_hash: txn_hash.to_string()
            })
        )
        .map_err(|err| {
            log::error!("Error while sending the transaction: {}", err);
            ResponseError::SendTransactionError { code: "Failed during sending the transaction".to_string() } 
        })
}

