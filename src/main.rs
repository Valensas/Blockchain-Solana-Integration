#[macro_use] extern crate rocket;

use rust_base58::FromBase58;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use rocket::serde::json::Json;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use spl_token::instruction::transfer;
use solana_program::instruction::Instruction;
use solana_sdk::hash::Hash;


use std::sync::Arc;

use rocket::State;

#[derive(Responder)]
enum ResponseErrors {
    #[response(status = 500, content_type = "json")]
    SendTransactionError {
        code: String
    },
    #[response(status = 500, content_type = "json")]
    CreateTransactionError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    CreatePubkeyError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    EmptyError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    CreateByteArrayError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    CreateKeypairError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    GetBlockhashError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    ConvertTransactionError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    CreateTransferError{
        code: String
    }
}

#[rocket::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    let rocket = match rocket::build()
    .mount("/", routes![get_latest_block, send_transaction, sign_transaction])
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

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    height: u64,
    hash: String
}

#[get("/blocks/latest")]
fn get_latest_block(
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<Block>, String> {
    let commitment_config = CommitmentConfig::finalized(); //TODO: İşlenmesi bitmiş block alınsın diye eklendi

    match rpc_client.get_slot_with_commitment(commitment_config) { // Son block slotunu alıp matchledik
        Ok(slot) => {
            let block = rpc_client.get_block(slot).unwrap();

            let block: Block = Block { // Block objesi yaratıldı
                height: block.block_height.unwrap(),
                hash: block.blockhash
            };

            Ok(Json(block))
            //serde_json::from_str(&block) // Block objesi JSON stringine dönüştürüldü ve returnlendi
        }
        Err(_) => Err(String::from("Slot not found"))
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct TransactionInfo { // Transaction bilgilerini içeren obje
    adress: String,
    amount: u64,
    contract: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionRequest { // Requestteki parametrelerle obje
    from: Vec<TransactionInfo>,
    to: Vec<TransactionInfo>,
    private_key: String
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct TransactionResponse { // Response için obje
    signedTransaction: String,
    txnHash: String
}

#[post("/transactions/sign", data = "<transaction_parameters>")]
fn sign_transaction(
    transaction_parameters: Json<TransactionRequest>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<TransactionResponse>, ResponseErrors> {
    
    if transaction_parameters.from.is_empty(){
        return Err(ResponseErrors::EmptyError{code : "From part of the request is empty".to_string()});
    }

    if transaction_parameters.to.is_empty(){
        return Err(ResponseErrors::EmptyError{code : "To part of the request is empty".to_string()});
    }

    let sender_address = match Pubkey::from_str(&transaction_parameters.from[0].adress){
        Ok(address) => address,
        Err(_) => return Err(ResponseErrors::CreatePubkeyError{code : "Failed during creating the Pubkey object".to_string()})
    }; // Gönderici adresi alıyor


    let privkey = transaction_parameters.private_key.clone(); // Private Key alınıyor
    let mut bytes_of_privatekey = match privkey.from_base58(){
        Ok(bytes) => bytes,
        Err(_) => return Err(ResponseErrors::CreateByteArrayError{code: "Failed during creating the byte array of private key".to_string()})
    }; // Private Key byte arraye dönüştürülüyor

    let mut bytes_of_publickey = match transaction_parameters.from[0].adress.from_base58(){
        Ok(bytes) => bytes,
        Err(_) => return Err(ResponseErrors::CreateByteArrayError{code: "Failed during creating the byte array of public key".to_string()})
    };
    bytes_of_privatekey.append(& mut bytes_of_publickey);

    let keypair: Keypair = match Keypair::from_bytes(&bytes_of_privatekey){
        Ok(key) => key,
        Err(_) => return Err(ResponseErrors::CreateKeypairError{code: "Failed during creating the keypair".to_string()})
    }; // Bu byte array ile Keypair objesi oluşturuluyor

    let blockhash: Hash = match rpc_client.get_latest_blockhash(){
        Ok(hash) => hash,
        Err(_) => return Err(ResponseErrors::GetBlockhashError{code: "Failed during getting the latest blockhash".to_string()})
    }; // Recent blockhash alınıyor

    let mut instructions: Vec<Instruction> = Vec::new(); // Instructions vektörü oluşturuluyor

    for transfer_param in &transaction_parameters.to{
        let to_address = match Pubkey::from_str(&transfer_param.adress){
            Ok(address) => address,
            Err(_) => return Err(ResponseErrors::CreatePubkeyError { code: "Failed during creating the Pubkey object".to_string() })
        };
        let amount = &transfer_param.amount;
        if transfer_param.contract.is_none(){ // Contract adresi yok ise
            instructions.push(solana_sdk::system_instruction::transfer(&sender_address, &to_address, *amount)) // Instruction oluşturulup vektöre pushlanıyor
        }
        else{ // Contract adresi var ise
            let contract_str:&str = match transfer_param.contract.as_ref(){
                Some(c) => c,
                None => return Err(ResponseErrors::EmptyError { code: "Failed during getting the contract address".to_string() })
            };

            let contract = match Pubkey::from_str(contract_str){
                Ok(contract) =>contract,
                Err(_) => return Err(ResponseErrors::CreatePubkeyError { code: "Failed during creating the Pubkey object".to_string() })
            };
            
            let instruction = match transfer(&contract, &sender_address, // Instruction (contract adresi verilerek) oluşturulup vektöre pushlanıyor
                &to_address, &sender_address, &[], *amount){
                    Ok(instruction) => instruction,
                    Err(_) => return Err(ResponseErrors::CreateTransferError{code : "Failed during creating the transaction instruction".to_string()})
                };
            instructions.push(instruction);
        }
    }

    let tx = Transaction::new_signed_with_payer( // Transaction objesi oluşturuluyor
        &instructions,
        Some(&sender_address),
        &[&keypair],
        blockhash
    );
    
    let signatures = &tx.signatures; // İçindeki imza alınıyor
    let txn_hash = signatures[0].to_string(); // İmza stringe çevriliyor

    let signed_transaction = match serde_json::to_string(&tx){
        Ok(transaction) => transaction,
        Err(_) => return Err(ResponseErrors::ConvertTransactionError { code: "Failed during converting Transaction object to String".to_string() })
    }; // tx objesi stringe çevriliyor
    let response: TransactionResponse = TransactionResponse{ // Response objesi oluşturuluyor
        txnHash: txn_hash,
        signedTransaction: signed_transaction
    };
    
    return Ok(Json(response)); // Returnleniyor


}



#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct SendTransactionRequest {  // Request için obje oluşturuldu
    signedTransaction: String,
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct SendTransactionResponse { // Response için obje oluşturuldu
    txnHash: String,
}

#[post("/transactions/send", data = "<transaction_parameters>")]
fn send_transaction(
    transaction_parameters: Json<SendTransactionRequest>,
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<SendTransactionResponse>, ResponseErrors>{

    let tx: Transaction = match serde_json::from_str::<Transaction>(&transaction_parameters.signedTransaction){
        Ok(tx) =>{
            tx
        },
        Err(_) => {
            return Err(ResponseErrors::CreateTransactionError { code: "Failed during creating the transaction object".to_string() });
        }
    };

    
    match rpc_client.send_and_confirm_transaction(&tx) {
        Ok(txn_hash) => { // Signature'ı Stringe çevir
            let response: SendTransactionResponse = SendTransactionResponse{ // Response objesi oluşturuluyor
                txnHash: txn_hash.to_string()
            };
            
            return Ok(Json(response));
        },
        Err(_) => {
            return Err(ResponseErrors::SendTransactionError { code: "Failed during sending the transaction".to_string() });
        }
    }
}

