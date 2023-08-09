use rocket::{Ignite, Rocket};
use blockchain_solana::server;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

mod mock;

pub async fn setup() -> Rocket<Ignite> {
    let mock_server = mock::start_mock_server().await;
    
    let rpc_url = mock_server.uri();
    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    server::start_server(rpc_client).await.unwrap()
}