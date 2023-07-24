#[macro_use] extern crate rocket;

use std::str::FromStr;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{*, option_serializer::OptionSerializer};
use std::sync::Arc;
use rocket::State;
use solana_sdk::transaction::Transaction;

const SOL_PRECISION: u32 = 9;

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
    LatestSlotError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    GetBlockError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    GetTransactionError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    StrToSignatureError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    TransactionMetaError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    EncodedTransactionTypeError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    TransactionMessageTypeError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    BalanceAmountError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    IndexError{
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
    .mount("/", routes![get_latest_block, send_transaction, scan_block_transactions_from_slot, get_transaction_details])
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
    height: u64, // Solana için height yerine slot kullanıyoruz
    hash: String
}

#[get("/blocks/latest")]
fn get_latest_block(
    rpc_client: &State<Arc<RpcClient>>
) -> Result<Json<Block>, ResponseErrors> {
    
    let slot = match rpc_client.get_slot_with_commitment(CommitmentConfig::finalized()) { // Solana latest slot alındı
        Ok(slot) => {
            slot
        },
        Err(_) => {
            return Err(ResponseErrors::LatestSlotError { code: "Failed during getting the latest slot".to_string() });
        }
    };

    match rpc_client.get_block(slot) {  // Solana blocku verilen slota göre alındı
        Ok(block) => {
            Ok(Json(Block { // Block objesi yaratıldı ve Json olarak returnlendi
                height: slot, // Solana için height yerine slot kullanıyoruz
                hash: block.blockhash // Solana blocku içinden hash bilgisi alındı
            }))
        },
        Err(_) => {
            return Err(ResponseErrors::GetBlockError { code: "Failed during getting the block with given slot".to_string() });
        }
    }
}

/************************************* DOĞA ****************************************************/

#[derive(Debug, Serialize, Deserialize)]
struct AccountInfo {
    adress: String,
    amount: f64,
    contract: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionInfo {
    from: Vec<AccountInfo>,
    to: Vec<AccountInfo>,
    hash: String,
    status: String
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockTransactions {
    height: u64, // Solana için height yerine slot kullanıyoruz
    hash: String,
    transactions: Vec<TransactionInfo>
}

#[get("/blocks/<slot>")] // Örnek slot: 205126242
fn scan_block_transactions_from_slot(
    rpc_client: &State<Arc<RpcClient>>,
    slot: u64
) -> Result<Json<BlockTransactions>, ResponseErrors> {

    let block: EncodedConfirmedBlock = match rpc_client.get_block(slot) {  // Solana blocku verilen slota göre alındı
        Ok(block) => {
            block
        },
        Err(_) => {
            return Err(ResponseErrors::GetBlockError { code: "Failed during getting the block with given slot".to_string() });
        }
    };

    let mut transactions: Vec<TransactionInfo> = Vec::new(); // BlockTransaction'ın içine konulacak transactions vektörü açıldı
    
    let transactions_vec = block.transactions; // Solana block'undan transactions bilgisi vektör şeklinde alındı
    for transaction_meta in transactions_vec.iter() { // Alınan vektördeki transactionlar iterate edildi
        let transaction = match transaction_info_helper(transaction_meta.clone()) {
            Ok(transaction) => {
                transaction
            },
            Err(error) => {
                return Err(error);
            }
        };
        transactions.push(transaction);
    }

    Ok(Json(BlockTransactions { // BlockTransactions objesi yaratıldı ve Json olarak returnlendi
        height: slot, // Solana için height yerine slot kullanıyoruz
        hash: block.blockhash, // Solana blocku içinden hash bilgisi alındı
        transactions
    }))
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct DetailedTransaction {
    from: Vec<AccountInfo>,
    to: Vec<AccountInfo>,
    hash: String,
    status: String,
    fee: f64,
    blockHash: String,
    blockHeight: u64 // Solana için height yerine slot kullanıyoruz
}

#[allow(non_snake_case)]
#[get("/transactions/<txnHash>/detail")]
fn get_transaction_details(
    rpc_client: &State<Arc<RpcClient>>,
    txnHash: &str
) -> Result<Json<DetailedTransaction>, ResponseErrors> {

    let signature = match Signature::from_str(txnHash){
        Ok(signature) => {
            signature
        },
        Err(_) => {
            return Err(ResponseErrors::StrToSignatureError { code: "Failed during converting txnHash (&str) to Signature".to_string() });
        },
    };
    
    let conf_transaction = match rpc_client.get_transaction(&signature, UiTransactionEncoding::Json) {
        Ok(conf_transaction) => {
            conf_transaction
        }
        Err(_) => {
            return Err(ResponseErrors::GetTransactionError { code: "Failed during getting the transaction with given hash".to_string() });
        }
    };

    /***** Block bilgileri burada alınıyor *****/
    let block_slot = conf_transaction.slot;
    let block: EncodedConfirmedBlock = match rpc_client.get_block(block_slot) {  // Solana blocku verilen slota göre alındı
        Ok(block) => {
            block
        },
        Err(_) => {
            return Err(ResponseErrors::GetBlockError { code: "Failed during getting the block with given slot".to_string() });
        }
    };

    let block_hash = block.blockhash;
    
    let transaction_meta = conf_transaction.transaction;
    
    /***** Fee haricindeki transaction bilgileri helper metoduyla burada alınıyor *****/
    let transaction = match transaction_info_helper(transaction_meta.clone()) {
        Ok(transaction) => {
            transaction
        },
        Err(error) => {
            return Err(error);
        }
    };
    
    match transaction_meta.meta {
        Some(meta) => {  // Transaction metası alındı
            /***** Transaction fee burada alınıyor *****/
            let transaction_fee = (meta.fee as f64) / (10_u32.pow(SOL_PRECISION) as f64); // Metanın içinden fee alındı, precisiona göre düzenlendi

            return Ok(Json(DetailedTransaction {
                from: transaction.from,
                to: transaction.to,
                hash: transaction.hash,
                status: transaction.status,
                fee: transaction_fee,
                blockHash: block_hash,
                blockHeight: block_slot
            }));
        },
        None => {
            return Err(ResponseErrors::TransactionMetaError { code: "Failed during getting the meta from given transaction".to_string() });
        }
    }
}

fn transaction_info_helper(
    transaction_meta: EncodedTransactionWithStatusMeta
) -> Result<TransactionInfo, ResponseErrors> {
    
    let meta = match transaction_meta.meta {  // Transaction metası alındı
        Some(meta) => {
            meta
        }, 
        None => {
            return Err(ResponseErrors::TransactionMetaError { code: "Failed during getting the meta from given transaction".to_string() });
        }
    };

    /***** Transaction status burada alınıyor *****/
    let transaction_status = meta.status.is_ok().to_string();

    /* SOL balances */
    let pre_balances = meta.pre_balances;
    let post_balances = meta.post_balances;

    /* Token balances */
    let pre_token_balances = match meta.pre_token_balances {
        OptionSerializer::Some(balances) => balances,
        OptionSerializer::None => Vec::new(),
        OptionSerializer::Skip => Vec::new(),
    };
    let post_token_balances = match meta.post_token_balances {
        OptionSerializer::Some(balances) => balances,
        OptionSerializer::None => Vec::new(),
        OptionSerializer::Skip => Vec::new(),
    };

    let transaction: UiTransaction = match transaction_meta.transaction {
        EncodedTransaction::LegacyBinary(_legacy_binary) => {
            return Err(ResponseErrors::EncodedTransactionTypeError { code: "Encoded transaction type LegacyBinary not implemented".to_string() });
        },
        EncodedTransaction::Binary(_binary, _encoding) => {
            return Err(ResponseErrors::EncodedTransactionTypeError { code: "Encoded transaction type Binary not implemented".to_string() });
        },
        EncodedTransaction::Accounts(_ui_accounts_list) => {
            return Err(ResponseErrors::EncodedTransactionTypeError { code: "Encoded transaction type Accounts not implemented".to_string() });
        },
        EncodedTransaction::Json(ui_transaction) => ui_transaction
    };

    /***** Transaction hash burada alınıyor *****/
    let transaction_hash = (&transaction.signatures[0]).to_string();
                    
    let mut transaction_from: Vec<AccountInfo> = Vec::new(); // Transaction objesinin içine konulacak from vektörü açıldı
    let mut transaction_to: Vec<AccountInfo> = Vec::new(); // Transaction objesinin içine konulacak to vektörü açıldı

    let message = match transaction.message {
        UiMessage::Parsed(_ui_parsed_message) => {
            return Err(ResponseErrors::TransactionMessageTypeError { code: "Transaction message type Parsed not implemented".to_string() });
        },
        UiMessage::Raw(ui_raw_message) => ui_raw_message
    };

    /***** Account adresleri burada alınıyor *****/
    let account_keys = &message.account_keys;
                                
    /***** SOL transactionları yazılıyor *****/
    for (i, account_key) in account_keys.iter().enumerate() { // Tüm accountların balance değişimini hesaplamak için
        /***** Amountlar burada hesaplanıyor *****/
        let amount: f64 = (post_balances[i] as f64 - pre_balances[i] as f64) / (10_u32.pow(SOL_PRECISION) as f64);
            
        /***** AccountInfolar burada to/from vektörlerine pushlanıyor *****/
        if amount < 0.0 { // Balance değişimi 0'dan küçükse from'a, 0'dan büyükse to'ya yazılıyor - 0 ise yazılmıyor
            transaction_from.push(AccountInfo {
                adress: account_key.clone(),
                amount: -amount,
                contract: None // SOL için contract None konuluyor
            })
        } else if amount > 0.0 {
            transaction_to.push(AccountInfo {
                adress: account_key.clone(),
                amount: amount,
                contract: None // SOL için contract None konuluyor
            })
        }
    }

    /***** Token transactionları yazılıyor *****/
    for post_token_balance in post_token_balances.iter() {
        let account_index = post_token_balance.account_index as usize;
        let mint = post_token_balance.clone().mint;
        let mut amount = match post_token_balance.ui_token_amount.ui_amount {
            Some(ui_amount) => {
                ui_amount
            },
            None => {
                return Err(ResponseErrors::BalanceAmountError { code: "Failed during getting the amount from token balance".to_string() });
            }
        };

        for pre_token_balance in pre_token_balances.iter() {
            if pre_token_balance.account_index as usize == account_index && pre_token_balance.mint == mint {
                amount -= match pre_token_balance.ui_token_amount.ui_amount {
                    Some(ui_amount) => {
                        ui_amount
                    },
                    None => {
                        return Err(ResponseErrors::BalanceAmountError { code: "Failed during getting the amount from token balance".to_string() });
                    }
                };
            }
        }
        
        if amount < 0.0 {
            transaction_from.push(AccountInfo {
                adress: match account_keys.get(account_index) {
                    Some(address) => {
                        address.clone()
                    },
                    None => {
                        return Err(ResponseErrors::IndexError { code: "Index out of bounds for account_keys vector".to_string() });
                    }
                },
                amount: -amount,
                contract: Some(mint) // Token adresi
            })
        } else if amount > 0.0 {
            transaction_to.push(AccountInfo {
                adress: match account_keys.get(account_index) {
                    Some(address) => {
                        address.clone()
                    },
                    None => {
                        return Err(ResponseErrors::IndexError { code: "Index out of bounds for account_keys vector".to_string() });
                    }
                },
                amount: amount,
                contract: Some(mint) // Token adresi
            })
        }
    }

    return Ok(TransactionInfo {
        from: transaction_from,
        to: transaction_to,
        hash: transaction_hash,
        status: transaction_status
    });
}

/************************************* END DOĞA ***************************************************/

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

