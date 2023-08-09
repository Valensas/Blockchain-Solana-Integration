use rocket::serde::{self, json::Json};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Code {
    pub code: String
}

#[derive(Responder, Debug)]
pub enum ResponseError {
    #[response(status = 500, content_type = "json")]
    SendTransactionError(Json<Code>),

    #[response(status = 400, content_type = "json")]
    CreateTransactionError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    LatestSlotError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    CreateInstructionsArray(Json<Code>),

    #[response(status = 404, content_type = "json")]
    GetBlockError(Json<Code>),

    #[response(status = 404, content_type = "json")]
    GetBalanceError(Json<Code>),

    #[response(status = 404, content_type = "json")]
    GetTransactionError(Json<Code>),

    #[response(status = 404, content_type = "json")]
    GetBlockHeightError(Json<Code>),

    #[response(status = 400, content_type = "json")]
    StrToSignatureError(Json<Code>),

    #[response(status = 404, content_type = "json")]
    TransactionMetaError(Json<Code>),

    #[response(status = 404, content_type = "json")]
    BlockTransactionsError(Json<Code>),

    #[response(status = 501, content_type = "json")]
    EncodedTransactionTypeError(Json<Code>),

    #[response(status = 501, content_type = "json")]
    TransactionMessageTypeError(Json<Code>),

    #[response(status = 404, content_type = "json")]
    BalanceAmountError(Json<Code>),

    #[response(status = 404, content_type = "json")]
    IndexError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    EmptyError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    CreatePubkeyError(Json<Code>),

    #[response(status = 400, content_type = "json")]
    CreateByteArrayError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    CreateKeypairError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    GetBlockhashError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    ConvertTransactionError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    CreateTransferError(Json<Code>),
    
    #[response(status = 500, content_type = "json")]
    GetFeeError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    PrometheusError(Json<Code>),

    #[response(status = 500, content_type = "json")]
    ConvertUiAmountError(Json<Code>),

    #[response(status = 501, content_type = "json")]
    UiAccountDataTypeError(Json<Code>)
}