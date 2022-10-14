# evm-bench

[![Rust](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml/badge.svg)](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml)

**evm-bench is a suite of Ethereum Virtual Machine (EVM) stress tests and benchmarks.**

evm-bench makes it easy to compare EVM performance in a scalable, standardized, and portable way.

|                         | py-evm.cpython | py-evm.pypy | pyrevm | revm   |
| ----------------------- | -------------- | ----------- | ------ | ------ |
| erc20.approval-transfer | 1.4258s        | 377.2ms     | 16.8ms | 10ms   |
| erc20.mint              | 1.4092s        | 347.8ms     | 14.4ms | 5.6ms  |
| erc20.transfer          | 1.9988s        | 420.8ms     | 22.6ms | 11.4ms |
| snailtracer             | 17.857s        | 5.17s       | 138ms  | 58ms   |
| ten-thousand-hashes     | 3.4344s        | 665.6ms     | 18.2ms | 7ms    |
|                         |                |             |        |        |
| **sum**                 | 26.1252s       | 6.9814s     | 210ms  | 92ms   |
| **relative**            | 284x           | 75.9x       | 2.28x  | 1.00x  |

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
