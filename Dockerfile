FROM docker.io/rust:1.70 as build

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

RUN cargo build --release

COPY ./src ./src

RUN cargo build --release

CMD ["./target/release/blockchain-solana"]