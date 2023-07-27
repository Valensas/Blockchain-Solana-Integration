#[macro_use] extern crate rocket;

pub mod errors;
pub mod blocks;
pub mod models;
pub mod config;
pub mod transactions;
pub mod wallets;
pub mod network;

use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

#[rocket::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    let rocket = match rocket::build()
    .mount("/", routes![
        blocks::get_latest_block,
        blocks::scan_block_transactions_from_slot,
        transactions::sign_transaction,
        transactions::send_transaction,
        transactions::get_transaction_details,
        transactions::get_confirmation_count,
        wallets::get_wallet_balance,
        wallets::create_wallet_address,
        network::get_calculated_fee
    ])
    .manage(rpc_client).ignite().await {
        Ok(rocket) => {
            log::info!("Server started gracefully");
            rocket
        },
        Err(err) => {
            log::error!("Server could not start gracefully: {}", err);
            return;
        },
    };

    // End State
    match rocket.launch().await {
        Ok(_) => {
            log::info!("Server closed gracefully");
        },
        Err(err) => {
            log::error!("Server could not close gracefully: {}", err);
        },
    };
}
