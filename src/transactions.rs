use std::{sync::Arc, str::FromStr};

use rocket::{State, serde::json::Json};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Signature, transaction::Transaction};
use solana_transaction_status::{EncodedConfirmedBlock, UiTransactionEncoding};

use crate::{errors::ResponseError, models::{DetailedTransaction, TransactionInfoConvertiable, SendTransactionRequest, SendTransactionResponse}, config::SOL_PRECISION};


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
        .map_err(|_| ResponseError::CreateTransactionError { code: "Failed during creating the transaction object".to_string() } )?;

    rpc_client
        .send_and_confirm_transaction(&tx)
        .map(|txn_hash|
            Json(SendTransactionResponse{ // Response objesi oluşturuluyor
                txn_hash: txn_hash.to_string()
            })
        )
        .map_err(|_| ResponseError::SendTransactionError { code: "Failed during sending the transaction".to_string() } )
}

