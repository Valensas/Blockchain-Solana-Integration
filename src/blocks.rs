use crate::{errors::ResponseError, models::{Block, TransactionInfo, TransactionInfoConvertiable}};

use std::sync::Arc;
use rocket::{State, serde::json::Json};
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::EncodedConfirmedBlock;

#[get("/blocks/latest")]
pub fn get_latest_block(
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<Block>, ResponseError> {

    let slot = match rpc_client.get_slot() { // Solana latest slot alındı
        Ok(slot) => {
            slot
        },
        Err(err) => {
            log::error!("Error getting latest block: {}", err);
            return Err(ResponseError::LatestSlotError { code: "Failed during getting the latest slot".to_string() });
        }
    };

    match rpc_client.get_block(slot) {  // Solana blocku verilen slota göre alındı
        Ok(block) => {
            Ok(Json(Block { // Block objesi yaratıldı ve Json olarak returnlendi
                height: slot, // Solana için height yerine slot kullanıyoruz
                hash: block.blockhash, // Solana blocku içinden hash bilgisi alındı
                transactions: vec![]
            }))
        },
        Err(_) => {
            return Err(ResponseError::GetBlockError { code: "Failed during getting the block with given slot".to_string() });
        }
    }
}

#[get("/blocks/<slot>")] // Örnek slot: 205126242
pub fn scan_block_transactions_from_slot(
    rpc_client: &State<Arc<RpcClient>>,
    slot: u64
) -> Result<Json<Block>, ResponseError> {

    let block: EncodedConfirmedBlock = match rpc_client.get_block(slot) {  // Solana blocku verilen slota göre alındı
        Ok(block) => {
            block
        },
        Err(_) => {
            return Err(ResponseError::GetBlockError { code: "Failed during getting the block with given slot".to_string() });
        }
    };

    let mut transactions: Vec<TransactionInfo> = Vec::new(); // BlockTransaction'ın içine konulacak transactions vektörü açıldı

    let transactions_vec = block.transactions; // Solana block'undan transactions bilgisi vektör şeklinde alındı
    for transaction_meta in transactions_vec.iter() { // Alınan vektördeki transactionlar iterate edildi
        let transaction =  transaction_meta.to_transaction_info()?;
        transactions.push(transaction);
    }

    Ok(Json(Block { // BlockTransactions objesi yaratıldı ve Json olarak returnlendi
        height: slot, // Solana için height yerine slot kullanıyoruz
        hash: block.blockhash, // Solana blocku içinden hash bilgisi alındı
        transactions
    }))
}
