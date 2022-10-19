# evm-bench

[![Rust](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml/badge.svg)](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml)

**evm-bench is a suite of Ethereum Virtual Machine (EVM) stress tests and benchmarks.**

evm-bench makes it easy to compare EVM performance in a scalable, standardized, and portable way.

|                         | ethereumjs | evmone | geth   | py-evm.cpython | py-evm.pypy | pyrevm  | revm   |
| ----------------------- | ---------- | ------ | ------ | -------------- | ----------- | ------- | ------ |
| erc20.approval-transfer | 1.8858s    | 7ms    | 17ms   | 1.3786s        | 424ms       | 16.2ms  | 10.4ms |
| erc20.mint              | 3.0138s    | 5ms    | 17.6ms | 1.4406s        | 349ms       | 14.2ms  | 6ms    |
| erc20.transfer          | 3.4688s    | 8.2ms  | 24.8ms | 1.9648s        | 453ms       | 22.2ms  | 11.4ms |
| snailtracer             | 131.415s   | 42ms   | 153ms  | 17.399s        | 5.591s      | 124ms   | 57ms   |
| ten-thousand-hashes     | 2.3178s    | 2.4ms  | 12.6ms | 1.902s         | 315.2ms     | 10.8ms  | 3.6ms  |
|                         |            |        |        |                |             |         |        |
| **sum**                 | 142.1012s  | 64.6ms | 225ms  | 24.085s        | 7.1322s     | 187.4ms | 88.4ms |
| **relative**            | 2200x      | 1.00x  | 3.48x  | 373x           | 110x        | 2.90x   | 1.37x  |

To reproduce these results, check out [usage with the evm-bench suite below](#with-the-evm-bench-suite).

## Technical Overview

In evm-bench there are [benchmarks](/benchmarks) and [runners](/runners):

- [Benchmarks](/benchmarks) are expensive Solidity contracts paired with configuration.
- [Runners](/runners) are consistent platforms for deploying and calling arbitrary smart contracts.

The evm-bench framework can run any benchmark on any runner. The links above dive deeper into how to build new benchmarks or runners.

## Usage

### With the evm-bench suite

Simply cloning this repository and running `RUST_LOG=info cargo run --release --` will do the trick. You may need to install some dependencies for the benchmark build process and the runner execution.

### With another suite

evm-bench is meant to be used with the pre-developed suite of benchmarks and runners in this repository. However, it should work as an independent framework elsewhere.

See the CLI arguments for evm-bench to figure out how to set it up! Alternatively just reach out to me or post an issue.

## Development

Do it. Reach out to me if you wanna lend a hand but don't know where to start!
