use serde::{Deserialize, Serialize};
use solana_transaction_status::UiTransactionTokenBalance;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, option_serializer::OptionSerializer, EncodedTransaction, UiMessage};

use crate::errors::ResponseError;
use crate::config::SOL_PRECISION;

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    /// We use slot instead of height in Solana
    pub height: u64,
    pub hash: String,
    pub transactions: Vec<TransactionInfo>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub adress: String,
    pub amount: f64,
    pub contract: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub from: Vec<AccountInfo>,
    pub to: Vec<AccountInfo>,
    pub hash: String,
    pub status: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedTransaction {
    pub from: Vec<AccountInfo>,
    pub to: Vec<AccountInfo>,
    pub hash: String,
    pub status: String,
    pub fee: f64,
    #[serde(rename="blockHash")]
    pub block_hash: String,
    /// We use slot instead of height in Solana
    #[serde(rename="blockHeight")]
    pub block_height: u64

}

pub trait TransactionInfoConvertiable {
  fn to_transaction_info(&self) -> Result<TransactionInfo, ResponseError>;
}

impl TransactionInfoConvertiable for EncodedTransactionWithStatusMeta {
  fn to_transaction_info(&self) -> Result<TransactionInfo, ResponseError> {
    let meta = match &self.meta {  // Transaction metası alındı
        Some(meta) => {
            meta
        },
        None => {
            return Err(ResponseError::TransactionMetaError { code: "Failed during getting the meta from given transaction".to_string() });
        }
    };

    /***** Transaction status burada alınıyor *****/
    let transaction_status = meta.status.is_ok().to_string();

    /* SOL balances */
    let pre_balances = &meta.pre_balances;
    let post_balances = &meta.post_balances;
    let empty_vec = Vec::<UiTransactionTokenBalance>::new();

    /* Token balances */
    let pre_token_balances = match &meta.pre_token_balances {
        OptionSerializer::Some(balances) => balances,
        OptionSerializer::None => &empty_vec,
        OptionSerializer::Skip => &empty_vec,
    };
    let post_token_balances = match &meta.post_token_balances {
        OptionSerializer::Some(balances) => balances,
        OptionSerializer::None => &empty_vec,
        OptionSerializer::Skip => &empty_vec,
    };

    let transaction = match &self.transaction {
        EncodedTransaction::LegacyBinary(_legacy_binary) => {
            return Err(ResponseError::EncodedTransactionTypeError { code: "Encoded transaction type LegacyBinary not implemented".to_string() });
        },
        EncodedTransaction::Binary(_binary, _encoding) => {
            return Err(ResponseError::EncodedTransactionTypeError { code: "Encoded transaction type Binary not implemented".to_string() });
        },
        EncodedTransaction::Accounts(_ui_accounts_list) => {
            return Err(ResponseError::EncodedTransactionTypeError { code: "Encoded transaction type Accounts not implemented".to_string() });
        },
        EncodedTransaction::Json(ui_transaction) => ui_transaction
    };

    /***** Transaction hash burada alınıyor *****/
    let transaction_hash = (&transaction.signatures[0]).to_string();

    let mut transaction_from: Vec<AccountInfo> = Vec::new(); // Transaction objesinin içine konulacak from vektörü açıldı
    let mut transaction_to: Vec<AccountInfo> = Vec::new(); // Transaction objesinin içine konulacak to vektörü açıldı

    let message = match &transaction.message {
        UiMessage::Parsed(_ui_parsed_message) => {
            return Err(ResponseError::TransactionMessageTypeError { code: "Transaction message type Parsed not implemented".to_string() });
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
                return Err(ResponseError::BalanceAmountError { code: "Failed during getting the amount from token balance".to_string() });
            }
        };

        for pre_token_balance in pre_token_balances.iter() {
            if pre_token_balance.account_index as usize == account_index && pre_token_balance.mint == mint {
                amount -= match pre_token_balance.ui_token_amount.ui_amount {
                    Some(ui_amount) => {
                        ui_amount
                    },
                    None => {
                        return Err(ResponseError::BalanceAmountError { code: "Failed during getting the amount from token balance".to_string() });
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
                        return Err(ResponseError::IndexError { code: "Index out of bounds for account_keys vector".to_string() });
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
                        return Err(ResponseError::IndexError { code: "Index out of bounds for account_keys vector".to_string() });
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    #[serde(rename="signedTransaction")]
    pub signed_transaction: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse {
  #[serde(rename="txnHash")]
    pub txn_hash: String,
}