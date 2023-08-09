use serde::{Deserialize, Serialize};
use solana_transaction_status::UiTransactionTokenBalance;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, option_serializer::OptionSerializer, EncodedTransaction, UiMessage};
use crate::errors::{ResponseError, Code};
use crate::config::SOL_PRECISION;
use std::{sync::{Arc, RwLock},time::Instant};
use prometheus::{opts, HistogramVec, IntCounterVec, Registry};
use rocket::{
    fairing::{Fairing, Info, Kind},
    Data, Request, Response,
    serde::json::Json
};

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
    pub status: String,
    pub fee: f64,
    #[serde(rename="blockHash")]
    pub block_hash: String,
    /// We use slot instead of height in Solana
    #[serde(rename="blockHeight")]
    pub block_height: u64
}

pub trait TransactionInfoConvertiable {
  fn to_transaction_info(&self, block_slot: u64, hash: &str) -> Result<TransactionInfo, ResponseError>;
}

impl TransactionInfoConvertiable for EncodedTransactionWithStatusMeta {
  fn to_transaction_info(&self, block_slot: u64, block_hash: &str) -> Result<TransactionInfo, ResponseError> {
    let meta = match &self.meta {
        Some(meta) => {
            meta
        },
        None => {
            return Err(ResponseError::TransactionMetaError(Json(Code { code: "Failed during getting the meta from given transaction".to_string() })));
        }
    };

    let transaction_status = if meta.status.is_ok() {
        "Success".to_string()
    } else {
        "Failed".to_string()
    };

    let transaction_fee = adjust_precision(meta.fee as f64);

    let transaction = match &self.transaction {
        EncodedTransaction::LegacyBinary(_legacy_binary) => {
            return Err(ResponseError::EncodedTransactionTypeError(Json(Code{ code: "Encoded transaction type LegacyBinary not implemented".to_string() })));
        },
        EncodedTransaction::Binary(_binary, _encoding) => {
            return Err(ResponseError::EncodedTransactionTypeError(Json(Code{ code: "Encoded transaction type Binary not implemented".to_string() })));
        },
        EncodedTransaction::Accounts(_ui_accounts_list) => {
            return Err(ResponseError::EncodedTransactionTypeError(Json(Code{ code: "Encoded transaction type Accounts not implemented".to_string() })));
        },
        EncodedTransaction::Json(ui_transaction) => ui_transaction
    };

    let transaction_hash = &transaction.signatures[0];

    let mut transaction_from: Vec<AccountInfo> = Vec::new();
    let mut transaction_to: Vec<AccountInfo> = Vec::new();

    let message = match &transaction.message {
        UiMessage::Parsed(_ui_parsed_message) => {
            return Err(ResponseError::TransactionMessageTypeError (Json(Code{ code: "Transaction message type Parsed not implemented".to_string() })));
        },
        UiMessage::Raw(ui_raw_message) => ui_raw_message
    };

    let account_keys = &message.account_keys;

    let pre_balances = &meta.pre_balances;
    let post_balances = &meta.post_balances;
    let empty_vec = Vec::<UiTransactionTokenBalance>::new();

    for (i, account_key) in account_keys.iter().enumerate() {
        let diff = post_balances[i] as f64 - pre_balances[i] as f64;
        let amount: f64 = adjust_precision(diff);

        if amount < 0.0 {
            transaction_from.push(AccountInfo {
                adress: account_key.clone(),
                amount: -amount,
                /// For SOL transactions, contract is None
                contract: None 
            })
        } else if amount > 0.0 {
            transaction_to.push(AccountInfo {
                adress: account_key.clone(),
                amount,
                /// For SOL transactions, contract is None
                contract: None 
            })
        }
    }

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

    for post_token_balance in post_token_balances.iter() {
        let account_index = post_token_balance.account_index as usize;
        let mint = post_token_balance.clone().mint;
        let mut amount = match post_token_balance.ui_token_amount.ui_amount {
            Some(ui_amount) => {
                ui_amount
            },
            None => {
                return Err(ResponseError::BalanceAmountError (Json(Code { code: "Failed during getting the amount from token balance".to_string() })));
            }
        };

        for pre_token_balance in pre_token_balances.iter() {
            if pre_token_balance.account_index as usize == account_index && pre_token_balance.mint == mint {
                amount -= match pre_token_balance.ui_token_amount.ui_amount {
                    Some(ui_amount) => {
                        ui_amount
                    },
                    None => {
                        return Err(ResponseError::BalanceAmountError (Json(Code { code: "Failed during getting the amount from token balance".to_string() })));
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
                        return Err(ResponseError::IndexError (Json(Code{code: "Index out of bounds for account_keys vector".to_string() })));
                    }
                },
                amount: -amount,
                contract: Some(mint)
            })
        } else if amount > 0.0 {
            transaction_to.push(AccountInfo {
                adress: match account_keys.get(account_index) {
                    Some(address) => {
                        address.clone()
                    },
                    None => {
                        return Err(ResponseError::IndexError (Json(Code { code: "Index out of bounds for account_keys vector".to_string() })));
                    }
                },
                amount,
                contract: Some(mint)
            })
        }
    }

    Ok(TransactionInfo {
        from: transaction_from,
        to: transaction_to,
        hash: transaction_hash.to_string(),
        status: transaction_status,
        fee: transaction_fee,
        block_hash: block_hash.to_string(),
        block_height: block_slot
    })
  }
}

fn adjust_precision(val: f64) -> f64 {
    (val) / (10_u32.pow(SOL_PRECISION) as f64)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    #[serde(rename="signedTransaction")]
    pub signed_transaction: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse {
  #[serde(rename="txnHash")]
    pub txn_hash: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SignTransactionRequest {
    pub from: Vec<AccountInfo>,
    pub to: Vec<AccountInfo>,
    pub private_key: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SignTransactionResponse {
    #[serde(rename="signedTransaction")]
    pub signed_transaction: String,
    #[serde(rename="txnHash")]
    pub txn_hash: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub balance: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletResponse {
    pub address: String,
    #[serde(rename="privateKey")]
    pub private_key: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractResponse {
    #[serde(rename="calculatedFee")]
    pub calculated_fee: u64
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmationCount {
    #[serde(rename="confirmationsCount")]
    pub confirmations_count: u64
}

pub struct PrometheusMetrics{
    http_request_count: IntCounterVec,
    http_request_durations: HistogramVec,
    registry: Registry
}

impl PrometheusMetrics{
    pub fn new(namespace: &str) -> Result<Self, ResponseError>{
        let registry = Registry::new();

        let http_request_count_opts = opts!(
            "http_request_count", 
            "Total number of HTTP requests"
        ).namespace(namespace);

        let http_request_count = IntCounterVec::new(
            http_request_count_opts,
            &["endpoint", "method", "status"]
        ).map_err(|err| {
            log::error!("Error while creating the IntCounterVec for prometheus: {}", err);
            ResponseError::PrometheusError(Json(Code {code: "CREATE_INTCOUNTER_ERROR".to_string()}))
        })?;

        let http_request_durations_opts = opts!(
            "http_request_durations", 
            "HTTP request duration in seconds for all requests"
        ).namespace(namespace);

        let http_request_durations = HistogramVec::new(
            http_request_durations_opts.into(),
            &["endpoint", "method", "status"],
        ).map_err(|err| {
            log::error!("Error while creating the HistogramVec for prometheus: {}", err);
            ResponseError::PrometheusError(Json(Code {code: "CREATE_HISTOGRAMVEC_ERROR".to_string()}))
        })?;

        registry.register(Box::new(http_request_count.clone())).map_err(|err| {
            log::error!("Error while adding the IntCounterVec to the register: {}", err);
            ResponseError::PrometheusError(Json(Code{code: "ADD_INTCOUNTER_ERROR".to_string()}))
        })?;
        registry.register(Box::new(http_request_durations.clone())).map_err(|err|{
            log::error!("Error while adding the HistogramVec to the register: {}", err);
            ResponseError::PrometheusError(Json(Code{code: "ADD_HISTOGRAMVEC_ERROR".to_string()}))
        })?;

        Ok(Self { http_request_count, http_request_durations, registry})
    }

    pub const fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn http_requests_count(&self) -> &IntCounterVec {
        &self.http_request_count
    }

    pub fn http_request_durations(&self) -> &HistogramVec {
        &self.http_request_durations
    }

}

impl Clone for PrometheusMetrics{
    fn clone(&self) -> Self{
        Self { http_request_count: self.http_request_count.clone(),
               http_request_durations: self.http_request_durations.clone(),
               registry: self.registry.clone()
        }
    }
}


pub trait ArcRwLockPrometheusTrait {
    type ArcRwLock;
    fn clone(&self) -> Arc<RwLock<PrometheusMetrics>>;
}

pub struct ArcRwLockPrometheus {
    pub rw_lock: Arc<RwLock<PrometheusMetrics>>,
}

impl ArcRwLockPrometheus {
    pub fn new(prometheus: Arc<RwLock<PrometheusMetrics>>) -> Self {
        Self {
            rw_lock: prometheus,
        }
    }
}

impl Clone for ArcRwLockPrometheus {
    fn clone(&self) -> Self {
        Self {
            rw_lock: Arc::clone(&self.rw_lock),
        }
    }
}

impl ArcRwLockPrometheusTrait for ArcRwLockPrometheus {
    type ArcRwLock = Arc<RwLock<PrometheusMetrics>>;

    fn clone(&self) -> Arc<RwLock<PrometheusMetrics>> {
        Arc::clone(&self.rw_lock)
    }
}


#[derive(Copy, Clone)]
struct TimerStart(Option<Instant>);

#[rocket::async_trait]
impl Fairing for ArcRwLockPrometheus {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus metric collection",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        req.local_cache(|| TimerStart(Some(Instant::now())));
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, response: &mut Response<'r>) {
        if req.route().is_none() {
            return;
        }

        let endpoint = req.route().unwrap().uri.as_str();
        let method = req.method().as_str();
        let status = response.status().code.to_string();
        self.rw_lock
            .read()
            .unwrap()
            .http_request_count
            .with_label_values(&[endpoint, method, status.as_str()])
            .inc();

        let start_time = req.local_cache(|| TimerStart(None));
        if let Some(duration) = start_time.0.map(|st| st.elapsed()) {
            let duration_secs = duration.as_secs_f64();
            self.rw_lock
                .read()
                .unwrap()
                .http_request_durations
                .with_label_values(&[endpoint, method, status.as_str()])
                .observe(duration_secs);
        }
    }
}
