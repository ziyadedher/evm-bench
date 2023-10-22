FROM rust:latest

WORKDIR /runner/
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo install --locked --path .

ENTRYPOINT [ "runner-revm" ]