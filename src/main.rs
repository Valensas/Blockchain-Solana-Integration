#[macro_use] extern crate rocket;
//use rocket::futures::executor::block_on;

use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use serde_json::Value;
//use bs58::decode;
use solana_sdk::signature::Keypair;
//use solana_sdk::instruction::Instruction;
//use rust_decimal::prelude::*;
//use solana_sdk::system_instruction::SystemInstruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_instruction::transfer_many;
//use solana_sdk::pubkey::ParsePubkeyError;
use std::str::FromStr;


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
    rocket::build().mount("/", routes![hello, greet, greet_json, get_latest_block])
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
struct TransactionInfo {
    adress: String,
    //#[serde(with = "rust_decimal::serde::str")]
    amount: u64
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionRequest { // Request Objesi
    from: Vec<TransactionInfo>,
    to: Vec<TransactionInfo>,
    contract: String,
    private_key: String
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionResponse { // Response için obje
    signedTransaction: String,
    txnHash: String
}

#[post("/transactions/sign", data = "<request>")]
/*
Örnek requestte birden fazla gönderici hesap olamayacağı ve oradaki private key gönderici adresin private keyi olarak varsayarak yazdım. 
Ayrıca amountları u64 olarak assume ettim solanadaki çoğu fonksiyonda öyle aldığı için. 
Sanırım amountların bir de contracttaki precision ile çarpmak gerekiyormuş ama emin değilim
*/
fn sign_transaction(request: &str) -> String {

    let rpc_url = "https://api.devnet.solana.com".to_string(); // Linki ekledik
    let rpc_client = RpcClient::new(rpc_url);
    let mut receiving_amount = 0; // Tutar kontrolü için kullanılacak değişken

    let transaction_parameters: TransactionRequest = serde_json::from_str(request).unwrap(); // requesti objeye çevir
    let mut to_and_amount: Vec<(Pubkey, u64)> = Vec::new(); // Kripto para alacak hesapların listesi
    //let mut instruction: [Vec<Instruction>; 1];
    
    if transaction_parameters.from.len() as i32 !=  1{ // Kripto para gönderen adres sayısı bir değilse hata returnlüyor
        return String::from("Inappropriate number of from address");
    }


    for transfer_param in &transaction_parameters.to{ // Kripto para alacak hesaplar ekleniyor listeye
        let to_address = Pubkey::from_str(&transfer_param.adress).unwrap();
        let amount = &transfer_param.amount;
        receiving_amount += amount;
        to_and_amount.push((to_address, *amount));
    }

    // let array_to_and_amount = to_and_amount.try_into().unwrap();

    let sender_address = Pubkey::from_str(&transaction_parameters.from[0].adress).unwrap(); // Gönderici adresi alıyor
    let amount = &transaction_parameters.from[0].amount; // Gönderilecek tutarı alıyor

    if *amount != receiving_amount{ // Gönderilen tutar ile alınacak tutar eşit değilse hata returnlüyor
        return String::from("Trying to send and receive different amounts");
    }

    // KULLANILMAYACAK instruction[0] = transfer_many(&sender_address, to_and_amount.try_into().unwrap());

    /*
    KULLANILMAYACAK
    for transfer_param in &transaction_parameters.from{
        let sender_address = Pubkey::from_str(&transfer_param.adress).unwrap();
        let amount = &transfer_param.amount;
        let instruction = transfer_many(&sender_address, &array_to_and_amount);
        instructions.push(instruction);
    }*/
    
    let blockhash = rpc_client.get_latest_blockhash().unwrap();

    let private_key = bs58::decode(&transaction_parameters.private_key).into_vec().unwrap(); // 

    let keypair = Keypair::from_bytes(&private_key).unwrap(); // new_signed_with_payer için gönderici adresin private key'i alındı ve keypair oluşturuldu

    let mut tx = Transaction::new_signed_with_payer(
        transfer_many(&sender_address, to_and_amount.try_into().unwrap()).try_into().unwrap(), // Temel sıkıntı burada
        Some(&sender_address),
        &[&keypair],
        blockhash
    );
    let signature = rpc_client.send_and_confirm_transaction(&tx).unwrap(); // İmza alınıyor
    let transaction_hash = Transaction::verify_and_hash_message(&tx).unwrap();// Transaction hash alınıyor
    let response: TransactionResponse = TransactionResponse{ // Response objesi oluşturuluyor hash ve signature ile
        txnHash: transaction_hash.to_string(),
        signedTransaction: signature.to_string()
    };
    return serde_json::to_string(&response).unwrap(); // Returnleniyor


}