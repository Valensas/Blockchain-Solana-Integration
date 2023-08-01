#[macro_use] extern crate rocket;

pub mod errors;
pub mod blocks;
pub mod models;
pub mod config;
pub mod transactions;
pub mod wallets;
pub mod network;
pub mod management;

use solana_client::rpc_client::RpcClient;
use std::{sync::{Arc, RwLock}, net::Ipv4Addr,};
use crate::models::{PrometheusMetrics, ArcRwLockPrometheus};

#[rocket::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    let config = rocket::Config {
         address: Ipv4Addr::new(0, 0, 0, 0).into(),
         ..rocket::Config::debug_default()
     };

    let prometheus = Arc::new(RwLock::new(PrometheusMetrics::new("blockchain_solana").unwrap()));
    let prometheus_fairing = ArcRwLockPrometheus::new(prometheus.clone());

    let rocket = match rocket::custom(config)
        .mount("/", routes![
        blocks::get_latest_block,
        blocks::scan_block_transactions_from_slot,
        transactions::sign_transaction,
        transactions::send_transaction,
        transactions::get_transaction_details,
        transactions::get_confirmation_count,
        wallets::get_wallet_balance,
        wallets::create_wallet_address,
        network::get_calculated_fee,
        management::metrics
    ])
    .attach(prometheus_fairing.clone())
    .manage(rpc_client)
    .manage(prometheus_fairing)
    .ignite().await {
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
