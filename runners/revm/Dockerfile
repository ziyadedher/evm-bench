FROM rust:latest

WORKDIR /runner/
COPY Cargo.toml ./
COPY src ./src

RUN cargo install --path .

ENTRYPOINT [ "runner-revm" ]