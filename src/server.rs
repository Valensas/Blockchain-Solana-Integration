use crate::blocks;
use crate::transactions;
use crate::wallets;
use crate::network;
use crate::management;

use rocket::Ignite;
use rocket::Rocket;
use solana_client::rpc_client::RpcClient;
use crate::models::{PrometheusMetrics, ArcRwLockPrometheus};
use std::sync::{Arc, RwLock};

pub async fn start_server(rpc_client: Arc<RpcClient>) -> Result<Rocket<Ignite>, ()>{
    let prometheus = Arc::new(RwLock::new(PrometheusMetrics::new("blockchain_solana").unwrap()));
    let prometheus_fairing = ArcRwLockPrometheus::new(prometheus.clone());

    match rocket::build()
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
        .manage(prometheus_fairing)
        .manage(rpc_client)
        .ignite().await {
            Ok(rocket) => {
                log::info!("Server started gracefully");
                Ok(rocket)
            },
            Err(err) => {
                log::error!("Server could not start gracefully: {:#?}", err);
                Err(())
            },
        }
}