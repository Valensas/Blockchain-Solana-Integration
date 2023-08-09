use solana_client::rpc_client::RpcClient;
use blockchain_solana::server;
use std::sync::Arc;

#[rocket::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let rpc_url = "https://api.devnet.solana.com".to_string();
    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    let rocket = server::start_server(rpc_client).await.unwrap();

    match rocket.launch().await {
        Ok(_) => {
            log::info!("Server closed gracefully");
        },
        Err(err) => {
            log::error!("Server could not close gracefully: {}", err);
        },
    };
}
