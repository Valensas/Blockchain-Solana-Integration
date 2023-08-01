FROM docker.io/rust:1.70 as build

WORKDIR /app
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm ./target/release/deps/blockchain_solana*

COPY ./src ./src

RUN cargo build --release

CMD ["./target/release/blockchain-solana"]