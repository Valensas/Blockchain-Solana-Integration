# Blockchain Integrations - Solana

This project presents a comprehensive solution for backend integration with the Solana blockchain, employing Rust and the Rocket framework. The system offers essential endpoints for tracking and employing blockchain operations. Prometheus metrics measuring request numbers and response times are added to ensure effective monitoring. Integration tests are implemented for each endpoint using Wiremock for Solana RPC Endpoints to enable offline testing. Project is dockerized and supported with Gitlab CI/CD pipelines.

## Usage

- Build the project:

```bash
cargo build
```

- Run the project:
```bash
cargo run
```

- Run tests:
```bash
cargo test
```

- Dockerize the project:
```bash
docker build . -t blockchain-solana
docker run --rm -p 8000:8000 blockchain-solana
```

**Note:** Address and port numbers are obtained from the environment (include .env file with ROCKET\_PORT and ROCKET\_ADDRESS to specify)

## Endpoints

| Endpoints                                     | Description |
| --------------------------------------------- | --------------------------------------------------------------------- |
| [GET] /blocks/latest                          | Get latest block information on the chain |                      
| [GET] /blocks/\<slot>                         | Get block details including transactions on the specified block |
| [POST] /transactions/sign                     | Generate signed transaction with given parameters in the request body |
| [POST] /transactions/send                     | Send signed transaction to the chain  |
| [GET] /transactions/\<txnHash>/detail         | Get detailed information of the specified transaction |
| [GET] /transactions/\<txnHash>/confirmations  | Get confirmation count of the given transaction   |
| [POST] /address                               | Generate wallet address   |
| [GET] /address/\<address>/balance?\<contract> | Get wallet SOL/token balance  |
| [GET] /fee/estimate?\<contract>               | Get a fee estimate for SOL/token transactions |

## Contributors

- Onur Sezen (onursezen@sabanciuniv.edu)

- Doğa Demirtürk (ddemirturk18@ku.edu.tr)