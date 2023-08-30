FROM rust:latest

WORKDIR /revm/
COPY . .

RUN cargo install --path .

ENTRYPOINT [ "runner-revm" ]