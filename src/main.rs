#![allow(non_snake_case)]
#[macro_use] extern crate rocket;
//use rocket::futures::executor::block_on;

use rust_base58::{ToBase58, FromBase58};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use serde_json::Value;
use rocket::serde::json::Json;
//use bs58::decode;
use solana_sdk::signature::Keypair;
//use solana_sdk::instruction::Instruction;
//use rust_decimal::prelude::*;
//use solana_sdk::system_instruction::SystemInstruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::pubkey::Pubkey;
//use solana_sdk::system_instruction::transfer_many;
//use solana_sdk::system_instruction::transfer;
use solana_sdk::system_instruction::transfer_many;
//use std::arch::aarch64::LD;
//use solana_sdk::pubkey::ParsePubkeyError;
use std::str::FromStr;
use spl_token::instruction::transfer;
use solana_program::instruction::Instruction;
use solana_sdk::signature::Signature;
use base64::encode;


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

#[post("/transactions/sign", data = "<request>")]
fn sign_transaction(request: &str) -> Result<Json<TransactionResponse>, String> {

    let rpc_url = "https://api.devnet.solana.com".to_string();
    let rpc_client = RpcClient::new(rpc_url); // RPC Client oluşturuldu
    let transaction_parameters: TransactionRequest = serde_json::from_str(request).unwrap(); // Request objeye çevrildi
    

    let sender_address = Pubkey::from_str(&transaction_parameters.from[0].adress).unwrap(); // Gönderici adresi alıyor


    let privkey_string = transaction_parameters.private_key.clone(); // Private Key alınıyor
    let mut byte_array = privkey_string.from_base58().unwrap(); // Private Key byte arraye dönüştürülüyor

    for i in &transaction_parameters.from[0].adress.from_base58().unwrap(){ // Oluşturulan byte arraye public keyin de byte şekli ekleniyor
        byte_array.push(*i);
    }

    let keypair: Keypair = Keypair::from_bytes(&byte_array).unwrap(); // Bu byte array ile Keypair objesi oluşturuluyor
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

    /*if transaction_parameters.contract.is_none(){

        let mut to_and_amount: Vec<(Pubkey, u64)> = Vec::new(); // Kripto para alacak hesapların listesi
        for transfer_param in &transaction_parameters.to{ // Kripto para alacak hesaplar ekleniyor listeye
            let to_address = Pubkey::from_str(&transfer_param.adress).unwrap();
            let amount = &transfer_param.amount;
            to_and_amount.push((to_address, *amount));
        }

        
        
        let instruction_array = transfer_many(&sender_address, to_and_amount.as_slice());

        let tx = Transaction::new_signed_with_payer(
            &instruction_array,
            Some(&sender_address),
            &[&keypair],
            blockhash
        );

        rpc_client.send_and_confirm_transaction(&tx).unwrap();
        let txnHash = Transaction::verify_and_hash_message(&tx).unwrap().to_string();// Transaction hash alınıyor
        let signedTransaction = serde_json::to_string(&tx).unwrap(); // Deserialize this with serde_json::from_str::<Transaction>("Transaction String");

        let response: TransactionResponse = TransactionResponse{ // Response objesi oluşturuluyor hash ve signature ile
            txnHash,
            signedTransaction
        };
        
        return Ok(Json(response)); // Returnleniyor
    }
    
    else{
        
        let mut instruction: Vec<Instruction> = Vec::new();
        let contract = Pubkey::from_str(&transaction_parameters.contract.unwrap()).unwrap();

        for transfer_param in &transaction_parameters.to{ 

            let to_address = Pubkey::from_str(&transfer_param.adress).unwrap();
            let amount = &transfer_param.amount;

            let ix = transfer(&contract, &sender_address,
                    &to_address, &sender_address, &[], *amount).unwrap();
            instruction.push(ix);
            
        }

        let tx = Transaction::new_signed_with_payer(
            &instruction,
            Some(&sender_address),
            &[&keypair],
            blockhash
        );

        let txnHash = Transaction::verify_and_hash_message(&tx).unwrap().to_string();// Transaction hash alınıyor
        let signedTransaction = serde_json::to_string(&tx).unwrap(); // Deserialize this with serde_json::from_str::<Transaction>("Transaction String");

        let response: TransactionResponse = TransactionResponse{ // Response objesi oluşturuluyor hash ve signature ile
            txnHash,
            signedTransaction
        };
        
        return Ok(Json(response)); // Returnleniyor

    }*/

}



