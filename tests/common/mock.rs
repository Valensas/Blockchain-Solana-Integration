use serde_json::json;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, body_partial_json};

pub async fn start_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;

    let get_slot_req_expected_body = json!({
        "method": "getSlot"
    });

    let get_slot_resp_expected_body = json!({ 
        "jsonrpc": "2.0", 
        "result": 234381065, 
        "id": 1
    });

    let get_slot_response = ResponseTemplate::new(200).set_body_json(get_slot_resp_expected_body);

    Mock::given(method("POST"))
        .and(body_partial_json(get_slot_req_expected_body))
        .respond_with(get_slot_response)
        .mount(&mock_server)
        .await;

    let get_version_req_expected_body = json!({
        "method": "getVersion"
    });

    let get_version_resp_expected_body = json!({ 
        "jsonrpc": "2.0", 
        "result": { "solana-core": "1.15.0" }, 
        "id": 1
    });

    let get_version_response = ResponseTemplate::new(200).set_body_json(get_version_resp_expected_body);

    Mock::given(method("POST"))
        .and(body_partial_json(get_version_req_expected_body))
        .respond_with(get_version_response)
        .mount(&mock_server)
        .await;

    let get_block_req_expected_body = json!({
        "method": "getBlock",
    });

    let get_block_resp_expected_body = json!({
        "jsonrpc": "2.0",
        "result": {
          "blockHeight": 428,
          "blockTime": null,
          "blockhash": "8qGKi2ycYh7xF17VtcSxd6rbrnWLXotNFWJ6gMCLyeY3",
          "parentSlot": 429,
          "previousBlockhash": "mfcyqEXB3DnHXki6KjjmZck6YjmZLvpAByy2fj4nh6B",
          "transactions": [
            {
              "meta": {
                "err": null,
                "fee": 5000,
                "innerInstructions": [],
                "logMessages": [],
                "postBalances": [499998, 26858640, 1, 1, 1],
                "postTokenBalances": [],
                "preBalances": [499998, 26858640, 1, 1, 1],
                "preTokenBalances": [],
                "rewards": [],
                "status": {
                  "Ok": null
                }
              },
              "transaction": {
                "message": {
                  "accountKeys": [
                    "3UVYmECPPMZSCqWKfENfuoTv51fTDTWicX9xmBD2euKe",
                    "AjozzgE83A3x1sHNUR64hfH7zaEBWeMaFuAN9kQgujrc",
                    "SysvarS1otHashes111111111111111111111111111",
                    "SysvarC1ock11111111111111111111111111111111",
                    "Vote111111111111111111111111111111111111111"
                  ],
                  "header": {
                    "numReadonlySignedAccounts": 0,
                    "numReadonlyUnsignedAccounts": 3,
                    "numRequiredSignatures": 1
                  },
                  "instructions": [
                    {
                      "accounts": [1, 2, 3, 0],
                      "data": "37u9WtQpcm6ULa3WRQHmj49EPs4if7o9f1jSRVZpm2dvihR9C8jY4NqEwXUbLwx15HBSNcP1",
                      "programIdIndex": 4
                    }
                  ],
                  "recentBlockhash": "mfcyqEXB3DnHXki6KjjmZck6YjmZLvpAByy2fj4nh6B"
                },
                "signatures": [
                  "2nBhEBYYvfaAe16UMNqRHre4YNSskvuYgx3M6E4JP1oDYvZEJHvoPzyUidNgNX5r9sTyN1J9UxtbCXy2rqYcuyuv"
                ]
              }
            }
          ]
        },
        "id": 1
      });

    let get_block_response = ResponseTemplate::new(200).set_body_json(get_block_resp_expected_body);

    Mock::given(method("POST"))
        .and(body_partial_json(get_block_req_expected_body))
        .respond_with(get_block_response)
        .mount(&mock_server)
        .await;

    let get_block_height_req_expected_body = json!({
        "method": "getBlockHeight"
    });

    let get_block_height_resp_expected_body = json!({
        "jsonrpc": "2.0",
        "result": 1233,
        "id": 1
    });

    let get_block_height_response = ResponseTemplate::new(200).set_body_json(get_block_height_resp_expected_body);

    Mock::given(method("POST"))
        .and(body_partial_json(get_block_height_req_expected_body))
        .respond_with(get_block_height_response)
        .mount(&mock_server)
        .await;

    let get_balance_req_expected_body = json!({
        "method": "getBalance"
    });

    let get_balance_resp_expected_body = json!({
        "jsonrpc": "2.0",
        "result": { "context": { "slot": 1 }, "value": 2000000000 },
        "id": 1
    });

    let get_balance_response = ResponseTemplate::new(200).set_body_json(get_balance_resp_expected_body);

    Mock::given(method("POST"))
        .and(body_partial_json(get_balance_req_expected_body))
        .respond_with(get_balance_response)
        .mount(&mock_server)
        .await;

    let send_transaction_req_expected_body = json!({
        "method": "sendTransaction"
    });

    let send_transaction_resp_expected_body = json!({
        "jsonrpc": "2.0",
        "result": "4rCHUpQW8jJdXgiwcJRyHeoudwKbkZQBYp5hKK85MK39KjKM3MWVzD9fzd2gxF3hWuT3vjGq3kZ77jCbXdtrARi9",
        "id": 1
    });

    let send_transaction_response = ResponseTemplate::new(200).set_body_json(send_transaction_resp_expected_body);

    Mock::given(method("POST"))
        .and(body_partial_json(send_transaction_req_expected_body))
        .respond_with(send_transaction_response)
        .mount(&mock_server)
        .await;

    let get_blockhash_req_expected_body = json!({
      "method": "getLatestBlockhash"
    });
  
    let get_blockhash_resp_expected_body = json!({ 
        "jsonrpc": "2.0",
        "result": {
            "context": {
            "slot": 2792
            },
            "value": {
            "blockhash": "5HGWYu788RuwwKLBkwXsiRdxMHbRSWqXvvPCdkxC6Cn3",
            "lastValidBlockHeight": 3090
            }
        },
        "id": 1
    });
  
    let get_blockhash_response = ResponseTemplate::new(200).set_body_json(get_blockhash_resp_expected_body);
  
    Mock::given(method("POST"))
        .and(body_partial_json(get_blockhash_req_expected_body))
        .respond_with(get_blockhash_response)
        .mount(&mock_server)
        .await;

    let get_fee_for_message_req_expected_body = json!({
      "method": "getFeeForMessage"
    });
  
    let get_fee_for_message_resp_expected_body = json!({ 
      "jsonrpc": "2.0",
      "result": { "context": { "slot": 5068 }, "value": 5000 },
      "id": 1
    });
  
    let get_fee_for_message_response = ResponseTemplate::new(200).set_body_json(get_fee_for_message_resp_expected_body);
  
    Mock::given(method("POST"))
        .and(body_partial_json(get_fee_for_message_req_expected_body))
        .respond_with(get_fee_for_message_response)
        .mount(&mock_server)
        .await;
    
    let get_transaction_req_expected_body = json!({
      "method": "getTransaction",
    });
  
    let get_transaction_resp_expected_body = json!({ 
      "jsonrpc": "2.0",
      "result": {
        "meta": {
          "err": null,
          "fee": 5000,
          "innerInstructions": [],
          "postBalances": [499998, 26858640, 1, 1, 1],
          "postTokenBalances": [],
          "preBalances": [499998, 26858640, 1, 1, 1],
          "preTokenBalances": [],
          "rewards": [],
          "status": {
            "Ok": null
          }
        },
        "slot": 430,
        "transaction": {
          "message": {
            "accountKeys": [
              "3UVYmECPPMZSCqWKfENfuoTv51fTDTWicX9xmBD2euKe",
              "AjozzgE83A3x1sHNUR64hfH7zaEBWeMaFuAN9kQgujrc",
              "SysvarS1otHashes111111111111111111111111111",
              "SysvarC1ock11111111111111111111111111111111",
              "Vote111111111111111111111111111111111111111"
            ],
            "header": {
              "numReadonlySignedAccounts": 0,
              "numReadonlyUnsignedAccounts": 3,
              "numRequiredSignatures": 1
            },
            "instructions": [
              {
                "accounts": [1, 2, 3, 0],
                "data": "37u9WtQpcm6ULa3WRQHmj49EPs4if7o9f1jSRVZpm2dvihR9C8jY4NqEwXUbLwx15HBSNcP1",
                "programIdIndex": 4
              }
            ],
            "recentBlockhash": "mfcyqEXB3DnHXki6KjjmZck6YjmZLvpAByy2fj4nh6B"
          },
          "signatures": [
            "2nBhEBYYvfaAe16UMNqRHre4YNSskvuYgx3M6E4JP1oDYvZEJHvoPzyUidNgNX5r9sTyN1J9UxtbCXy2rqYcuyuv"
          ]
        }
      },
      "blockTime": null,
      "id": 1
    });
  
    let get_transaction_response = ResponseTemplate::new(200).set_body_json(get_transaction_resp_expected_body);
  
    Mock::given(method("POST"))
        .and(body_partial_json(get_transaction_req_expected_body))
        .respond_with(get_transaction_response)
        .mount(&mock_server)
        .await;

    let get_signature_statuses_req_expected_body = json!({
        "method": "getSignatureStatuses"
    });
  
    let get_signature_statuses_resp_expected_body = json!({ 
      "jsonrpc": "2.0",
      "result": {
        "context": {
          "slot": 82
        },
        "value": [
          {
            "slot": 48,
            "confirmations": null,
            "err": null,
            "status": {
              "Ok": null
            },
            "confirmationStatus": "finalized"
          },
          null
        ]
      },
      "id": 1
    });
  
    let get_signature_statuses_response = ResponseTemplate::new(200).set_body_json(get_signature_statuses_resp_expected_body);
  
    Mock::given(method("POST"))
        .and(body_partial_json(get_signature_statuses_req_expected_body))
        .respond_with(get_signature_statuses_response)
        .mount(&mock_server)
        .await;
      
    
    let get_is_blockhash_valid_req_expected_body = json!({
        "method": "isBlockhashValid"
    });

    let get_is_blockhash_valid_resp_expected_body = json!({ 
      "jsonrpc": "2.0",
      "result": {
        "context": {
          "slot": 2483
        },
        "value": true
      },
      "id": 1
    });

    let get_is_blockhash_valid_response = ResponseTemplate::new(200).set_body_json(get_is_blockhash_valid_resp_expected_body);

    Mock::given(method("POST"))
        .and(body_partial_json(get_is_blockhash_valid_req_expected_body))
        .respond_with(get_is_blockhash_valid_response)
        .mount(&mock_server)
        .await;

    
    let get_token_accounts_req_expected_body = json!({
        "method": "getTokenAccountsByOwner"
    });
  
    let get_token_accounts_resp_expected_body = json!({ 
        "jsonrpc": "2.0",
        "result": {
          "context": {
            "slot": 1114
          },
          "value": [
            {
              "account": {
                "data": {
                  "program": "spl-token",
                  "parsed": {
                    "accountType": "account",
                    "info": {
                      "tokenAmount": {
                        "amount": "1",
                        "decimals": 1,
                        "uiAmount": 0.1,
                        "uiAmountString": "0.1"
                      },
                      "delegate": "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T",
                      "delegatedAmount": {
                        "amount": "1",
                        "decimals": 1,
                        "uiAmount": 0.1,
                        "uiAmountString": "0.1"
                      },
                      "state": "initialized",
                      "isNative": false,
                      "mint": "3wyAj7Rt1TWVPZVteFJPLa26JmLvdb1CAKEFZm3NY75E",
                      "owner": "4Qkev8aNZcqFNSRhQzwyLMFSsi94jHqE8WNVTJzTP99F"
                    },
                    "type": "account"
                  },
                  "space": 165
                },
                "executable": false,
                "lamports": 1726080,
                "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "rentEpoch": 4,
                "space": 165
              },
              "pubkey": "C2gJg6tKpQs41PRS1nC8aw3ZKNZK3HQQZGVrDFDup5nx"
            }
          ]
        },
        "id": 1
    });
  
    let get_token_accounts_response = ResponseTemplate::new(200).set_body_json(get_token_accounts_resp_expected_body);
  
    Mock::given(method("POST"))
        .and(body_partial_json(get_token_accounts_req_expected_body))
        .respond_with(get_token_accounts_response)
        .mount(&mock_server)
        .await;

    mock_server
}