#[derive(Responder)]
pub enum ResponseError {
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
    },
    #[response(status = 500, content_type = "json")]
    EmptyError{
        code: String
    },
    #[response(status = 500, content_type = "json")]
    CreatePubkeyError{
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