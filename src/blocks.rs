use crate::{errors::ResponseError, models::{Block, TransactionInfo, TransactionInfoConvertiable}};

use std::sync::Arc;
use rocket::{State, serde::json::Json};
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::EncodedConfirmedBlock;


#[get("/blocks/latest")]
pub fn get_latest_block(
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<Block>, ResponseError> {

    let slot = rpc_client.get_slot()
    .map_err(|err| {
        log::error!("Error getting latest slot with the commmitment: {}", err); 
        ResponseError::LatestSlotError { code: "Failed during getting the latest slot".to_string()}})?;

    rpc_client.get_block(slot)
    .map(|block| Json(Block{
        height: slot,
        hash: block.blockhash,
        transactions: vec![]}))
    .map_err(|err| {
        log::error!("Error getting latest block: {}", err);
        ResponseError::GetBlockError { code: "Failed during getting the block with given slot".to_string()}})
}

#[get("/blocks/<slot>")]
pub fn scan_block_transactions_from_slot(
    rpc_client: &State<Arc<RpcClient>>,
    slot: u64
) -> Result<Json<Block>, ResponseError> {

    let block: EncodedConfirmedBlock = rpc_client.get_block(slot)
        .map_err(|err| {
            log::error!("Failed during getting the block with given slot: {}", err);
            ResponseError::GetBlockError { code: "Failed during getting the block with given slot".to_string() }
        })?;
    let hash = block.blockhash;

    let transactions_vec = block.transactions;
    let transactions: Vec<TransactionInfo> = transactions_vec.iter()
        .map(|transaction_meta| {
            transaction_meta.to_transaction_info(slot, &hash)
        }).collect::<Result<Vec<_>, _>>()?;

    Ok(Json(Block {
        /// We use slot instead of height in Solana
        height: slot, 
        hash,
        transactions
    }))
}
