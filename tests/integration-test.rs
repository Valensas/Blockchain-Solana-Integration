use std::env;
use rocket::{http::{Status, ContentType}, local::asynchronous::Client};
use blockchain_solana::models::{SignTransactionRequest, AccountInfo, SendTransactionRequest};
use assert_json_diff::assert_json_eq;
use serde_json::{json, Value};

mod common;

#[rocket::async_test]
async fn test_get_latest_block() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8001");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.get("/blocks/latest").dispatch().await;

    let actual_status = response.status();
    let actual_response: Value = serde_json::from_str(response.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status, Status::Ok);
    
    let expected_response = json!({
      "height":234381065,
      "hash":"8qGKi2ycYh7xF17VtcSxd6rbrnWLXotNFWJ6gMCLyeY3",
      "transactions":[]
    });

    assert_json_eq!(actual_response, expected_response);
}

#[rocket::async_test]
async fn test_scan_block() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8002");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.get("/blocks/234381065").dispatch().await;
    
    let actual_status = response.status();
    let actual_response: Value = serde_json::from_str(response.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status, Status::Ok);

    let expected_response = json!({
      "height": 234381065,
      "hash": "8qGKi2ycYh7xF17VtcSxd6rbrnWLXotNFWJ6gMCLyeY3",
      "transactions":[{
        "blockHash":"8qGKi2ycYh7xF17VtcSxd6rbrnWLXotNFWJ6gMCLyeY3",
        "blockHeight":234381065,
        "fee":5e-6,
        "from":[],
        "to":[],
        "hash":"2nBhEBYYvfaAe16UMNqRHre4YNSskvuYgx3M6E4JP1oDYvZEJHvoPzyUidNgNX5r9sTyN1J9UxtbCXy2rqYcuyuv",
        "status":"Success"}]
    });

    assert_json_eq!(actual_response, expected_response);
}

#[rocket::async_test]
async fn test_get_transaction_details() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8003");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.get("/transactions/2Gqx5zXq8GUvbWZcKBjQzCiFTioYXLdWz7PQ86pvBJftnbtQLkfvVcfiRD271E3bbeP6FjQWc7DFALdZjtAEbW2y/detail").dispatch().await;
    
    let actual_status = response.status();
    let actual_response: Value = serde_json::from_str(response.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status, Status::Ok);

    let expected_response = json!({
      "blockHash":"8qGKi2ycYh7xF17VtcSxd6rbrnWLXotNFWJ6gMCLyeY3",
      "blockHeight":430,
      "fee":5e-6,
      "from":[],
      "to":[],
      "hash":"2nBhEBYYvfaAe16UMNqRHre4YNSskvuYgx3M6E4JP1oDYvZEJHvoPzyUidNgNX5r9sTyN1J9UxtbCXy2rqYcuyuv",
      "status":"Success"
    });

    assert_json_eq!(actual_response, expected_response);
}

#[rocket::async_test]
async fn test_get_confirmation_count() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8004");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.get("/transactions/2Gqx5zXq8GUvbWZcKBjQzCiFTioYXLdWz7PQ86pvBJftnbtQLkfvVcfiRD271E3bbeP6FjQWc7DFALdZjtAEbW2y/confirmations").dispatch().await;
    
    let actual_status = response.status();
    let actual_response: Value = serde_json::from_str(response.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status, Status::Ok);

    let expected_response = json!({
      "confirmationsCount": 805
    });

    assert_json_eq!(actual_response, expected_response);
}

#[rocket::async_test]
async fn test_get_calculated_fee() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8005");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let response_param = client.get("/fee/estimate?usdxvpXrXHw8WEDrNbj3EPdJaUopvrNDXToCPHSnaEs").dispatch().await;
    let response_no_param = client.get("/fee/estimate").dispatch().await;

    let actual_status_param = response_param.status();
    let actual_response_param: Value = serde_json::from_str(response_param.into_string().await.unwrap().as_ref()).unwrap();

    let actual_status_no_param = response_no_param.status();
    let actual_response_no_param: Value = serde_json::from_str(response_no_param.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;

    assert_eq!(actual_status_param, Status::Ok);
    assert_eq!(actual_status_no_param, Status::Ok);

    let expected_response_param = json!({
      "calculatedFee": 5000
    });

    let expected_response_no_param = json!({
      "calculatedFee": 5000
    });

    assert_json_eq!(actual_response_param, expected_response_param);
    assert_json_eq!(actual_response_no_param, expected_response_no_param);
}

#[rocket::async_test]
async fn test_address_generate() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8006");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.post("/address").dispatch().await;

    let actual_status = response.status();
    let _actual_response: Value = serde_json::from_str(response.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status, Status::Ok);
}

#[rocket::async_test]
async fn test_address_balance() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8007");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let response_param = client.get("/address/B45rg4rxKLncrNP8vYZLaT32qq1ohEDLvkNV8aQr8KzN/balance?contract=usdxvpXrXHw8WEDrNbj3EPdJaUopvrNDXToCPHSnaEs").dispatch().await;
    let response_no_param = client.get("/address/B45rg4rxKLncrNP8vYZLaT32qq1ohEDLvkNV8aQr8KzN/balance").dispatch().await;
    
    let actual_status_no_param = response_no_param.status();
    let actual_response_no_param: Value = serde_json::from_str(response_no_param.into_string().await.unwrap().as_ref()).unwrap();
    
    let actual_status_param = response_param.status();
    let actual_response_param: Value = serde_json::from_str(response_param.into_string().await.unwrap().as_ref()).unwrap();

    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status_no_param, Status::Ok);
    assert_eq!(actual_status_param, Status::Ok);

    let expected_response_no_param: Value = json!({
        "balance":2.0,
      });
    
    let expected_response_param: Value = json!({
        "balance":0.1,
    });

    assert_json_eq!(actual_response_no_param, expected_response_no_param);
    assert_json_eq!(actual_response_param, expected_response_param);
}

#[rocket::async_test]
async fn test_sign_transaction() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8008");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let request : SignTransactionRequest = SignTransactionRequest { from: [
        AccountInfo{
          adress: "B45rg4rxKLncrNP8vYZLaT32qq1ohEDLvkNV8aQr8KzN".to_string(),
          amount: 200000000 as f64,
          contract: None
        }
      ].into(), to: [
        AccountInfo{
          adress: "22NCQiCUivo4kijCSJwyA7qhSrx4kpba5UXvzdCZ66hH".to_string(),
          amount: 99999999 as f64,
          contract: None
        }
        ,
        AccountInfo{
          adress: "DAiB6ZGYa5xXuTMjsNVrS4PEnzutWB33HeQM6gmPJSZi".to_string(),
          amount: 99999999 as f64,
          contract: None
        }
      ].into(), private_key: "3J5zuqwqdecmVY7Xvk5T9j4ks1LTYAiq7mxsenuXGaZH".to_string() };

    let json_payload = rocket::serde::json::to_string(&request).unwrap();
    let response = client.post("/transactions/sign").header(ContentType::JSON).body(json_payload).dispatch().await;
    
    let actual_status = response.status();
    let actual_response: Value = serde_json::from_str(response.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status, Status::Ok);

    let expected_response = json!({
        "txnHash":"4fXvvk1kZiuBjz4J7AcVJF4QaL2pPRDBHeqKrWvMsq3L6hUT7xZXPefDdNuARk2bkpaQGRn2LSJjMixm62ecmf8b",
        "signedTransaction":"{\"signatures\":[[1],[183,71,215,29,64,235,0,38,169,70,188,152,223,229,143,100,104,46,242,228,87,98,25,248,64,57,247,51,154,28,60,166,197,198,91,114,149,86,76,136,31,102,1,43,117,114,193,42,134,206,252,155,15,148,232,137,173,99,177,27,101,210,30,0]],\"message\":{\"header\":{\"numRequiredSignatures\":1,\"numReadonlySignedAccounts\":0,\"numReadonlyUnsignedAccounts\":1},\"accountKeys\":[[4],[149,95,26,86,184,220,32,31,179,153,193,147,215,146,85,9,96,123,219,164,195,170,107,233,178,158,99,94,249,52,187,207],[15,53,60,36,1,78,135,209,144,88,0,25,72,28,15,76,170,126,38,26,203,71,44,132,38,182,16,114,73,30,145,252],[180,201,6,172,2,243,102,48,245,131,13,38,125,29,147,84,41,138,173,82,226,184,74,214,165,231,122,38,9,51,73,181],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]],\"recentBlockhash\":[63,153,147,200,127,206,218,202,247,209,28,249,231,186,254,250,40,119,53,9,55,42,149,113,109,245,113,11,127,81,28,88],\"instructions\":[[2],{\"programIdIndex\":3,\"accounts\":[[2],0,1],\"data\":[[12],2,0,0,0,255,224,245,5,0,0,0,0]},{\"programIdIndex\":3,\"accounts\":[[2],0,2],\"data\":[[12],2,0,0,0,255,224,245,5,0,0,0,0]}]}}"
      });
  
    assert_json_eq!(actual_response, expected_response);
}

#[rocket::async_test]
async fn test_send_transaction() {
    dotenv::dotenv().ok();
    env::set_var("ROCKET_PORT", "8009");

    let rocket = common::setup().await;
    let client = Client::tracked(rocket).await.unwrap();

    let request : SendTransactionRequest = SendTransactionRequest { signed_transaction: "{\"signatures\":[[1],[192,121,121,143,216,202,102,196,11,13,214,44,86,244,245,201,28,233,91,194,78,106,95,42,171,37,195,24,63,33,93,82,230,248,1,203,171,207,190,0,71,82,19,206,53,211,198,27,89,6,220,46,116,17,142,154,62,226,182,118,134,10,211,10]],\"message\":{\"header\":{\"numRequiredSignatures\":1,\"numReadonlySignedAccounts\":0,\"numReadonlyUnsignedAccounts\":1},\"accountKeys\":[[4],[15,53,60,36,1,78,135,209,144,88,0,25,72,28,15,76,170,126,38,26,203,71,44,132,38,182,16,114,73,30,145,252],[149,95,26,86,184,220,32,31,179,153,193,147,215,146,85,9,96,123,219,164,195,170,107,233,178,158,99,94,249,52,187,207],[180,201,6,172,2,243,102,48,245,131,13,38,125,29,147,84,41,138,173,82,226,184,74,214,165,231,122,38,9,51,73,181],[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]],\"recentBlockhash\":[30,154,253,182,227,181,136,44,127,146,246,92,11,83,117,143,30,210,87,19,122,69,121,181,199,53,54,26,250,255,141,148],\"instructions\":[[2],{\"programIdIndex\":3,\"accounts\":[[2],0,1],\"data\":[[12],2,0,0,0,255,224,245,5,0,0,0,0]},{\"programIdIndex\":3,\"accounts\":[[2],0,2],\"data\":[[12],2,0,0,0,255,224,245,5,0,0,0,0]}]}}".to_string()};

    let json_payload = rocket::serde::json::to_string(&request).unwrap();

    let response = client.post("/transactions/send").header(ContentType::JSON).body(json_payload).dispatch().await;
    
    let actual_status = response.status();
    let actual_response: Value = serde_json::from_str(response.into_string().await.unwrap().as_ref()).unwrap();
    
    client.terminate().await.shutdown().await;
    
    assert_eq!(actual_status, Status::Ok);

    let expected_response = json!({
        "txnHash":"4rCHUpQW8jJdXgiwcJRyHeoudwKbkZQBYp5hKK85MK39KjKM3MWVzD9fzd2gxF3hWuT3vjGq3kZ77jCbXdtrARi9",
      });
  
    assert_json_eq!(actual_response, expected_response);
}