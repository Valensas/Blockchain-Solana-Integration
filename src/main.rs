#![allow(non_snake_case)]
#[macro_use] extern crate rocket;
//use rocket::futures::executor::block_on;

use rust_base58::FromBase58;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use serde_json::Value;
use rocket::serde::json::Json;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use spl_token::instruction::transfer;
use solana_program::instruction::Instruction;


#[get("/hello")]
fn hello() -> &'static str {
    "Hello world\n"
}

#[get("/greet?<name>")] // Name'e göre printliyor
fn greet(name: &str) -> String {
    format!("Hello {}\n", name)
}

#[derive(Debug, Deserialize)]
struct GreetingRequest {
    name: String
}

#[post("/greet", data = "<request>")] // JSONu alıyor, GreetingRequest'e dönüştürüyor, sonra ismi basıyor
fn greet_json(request: &str) -> String {
    let g: GreetingRequest = serde_json::from_str(request).unwrap();
    format!("Hello {}\n", g.name)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello, greet, greet_json, get_latest_block, sign_transaction])
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    height: u64,
    hash: String
}

#[get("/blocks/latest")]
fn get_latest_block() -> String {
    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = RpcClient::new(rpc_url);
    let commitment_config = CommitmentConfig::finalized(); //TODO: İşlenmesi bitmiş block alınsın diye eklendi
    
    match rpc_client.get_slot_with_commitment(commitment_config) { // Son block slotunu alıp matchledik
        Ok(slot) => {
            let block_str = serde_json::to_string(&rpc_client.get_block(slot).unwrap()).unwrap(); // Son blockun bilgileri stringe dönüştürüldü
            let block_json: Value = serde_json::from_str(&block_str).unwrap(); // Son blockun bilgileri JSONa dönüştürüldü
            let block_height: u64 = block_json["blockHeight"].to_string().parse::<u64>().unwrap(); // BlockHeight alındı
            let block_hash = block_json["blockhash"].to_string(); // Blockhash alındı
            let block: Block = Block { // Block objesi yaratıldı
                height: block_height,
                hash: block_hash
            };
            serde_json::to_string(&block).unwrap() // Block objesi JSON stringine dönüştürüldü ve returnlendi
        }
        Err(_) => String::from("Slot not found")
    }
}

// **************************************** Onur Sign Transaction *************************************************************
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

#[derive(Debug, Serialize, Deserialize)]
struct TransactionResponse { // Response için obje
    signedTransaction: String,
    txnHash: String
}

#[post("/transactions/sign", data = "<transaction_parameters>")]
fn sign_transaction(transaction_parameters: Json<TransactionRequest>) -> Result<Json<TransactionResponse>, String> {

    let rpc_url = "https://api.devnet.solana.com".to_string();
    let rpc_client = RpcClient::new(rpc_url); // RPC Client oluşturuldu
    
    if transaction_parameters.from.is_empty(){
        return Err(String::from("No item in from part"));
    }

    if transaction_parameters.to.is_empty(){
        return Err(String::from("No item in to part"));
    }

    let sender_address = Pubkey::from_str(&transaction_parameters.from[0].adress).unwrap(); // Gönderici adresi alıyor


    let privkey = transaction_parameters.private_key.clone(); // Private Key alınıyor
    let mut bytes_of_privatekey = privkey.from_base58().unwrap(); // Private Key byte arraye dönüştürülüyor

    bytes_of_privatekey.append(& mut transaction_parameters.from[0].adress.from_base58().unwrap());

    let keypair: Keypair = Keypair::from_bytes(&bytes_of_privatekey).unwrap(); // Bu byte array ile Keypair objesi oluşturuluyor
    let blockhash = rpc_client.get_latest_blockhash().unwrap(); // Recent blockhash alınıyor

    let mut instructions: Vec<Instruction> = Vec::new(); // Instructions vektörü oluşturuluyor

    for transfer_param in &transaction_parameters.to{
        let to_address = Pubkey::from_str(&transfer_param.adress).unwrap();
        let amount = &transfer_param.amount;
        if transfer_param.contract.is_none(){ // Contract adresi yok ise
            instructions.push(solana_sdk::system_instruction::transfer(&sender_address, &to_address, *amount)) // Instruction oluşturulup vektöre pushlanıyor
        }
        else{ // Contract adresi var ise
            let contract = Pubkey::from_str(transfer_param.contract.as_ref().unwrap()).unwrap();
            let ix = transfer(&contract, &sender_address, // Instruction (contract adresi verilerek) oluşturulup vektöre pushlanıyor
                &to_address, &sender_address, &[], *amount).unwrap();
            instructions.push(ix);
        }
    }

    let tx = Transaction::new_signed_with_payer( // Transaction objesi oluşturuluyor
        &instructions,
        Some(&sender_address),
        &[&keypair],
        blockhash
    );
    
    let signatures = &tx.signatures; // İçindeki imza alınıyor
    let txnHash = signatures[0].to_string(); // İmza stringe çevriliyor

    let signedTransaction = serde_json::to_string(&tx).unwrap(); // tx objesi stringe çevriliyor
    let response: TransactionResponse = TransactionResponse{ // Response objesi oluşturuluyor
        txnHash,
        signedTransaction
    };
    
    return Ok(Json(response)); // Returnleniyor


}


// **************************************** Onur Sign Transaction *************************************************************
