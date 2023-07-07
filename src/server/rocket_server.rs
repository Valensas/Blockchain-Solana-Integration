use mpc_rocket::client::client_service::Client as RocketClient;
use solana_sdk::client::Client as SolanaClient;

pub struct RocketServer{
    client: RocketClient,
    //solana_client: SolanaClient
}

impl RocketServer{
    pub fn new(client: RocketClient) -> Self{
        Self{
            client
        }
    }
}