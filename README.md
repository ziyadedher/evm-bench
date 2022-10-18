# evm-bench

[![Rust](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml/badge.svg)](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml)

**evm-bench is a suite of Ethereum Virtual Machine (EVM) stress tests and benchmarks.**

evm-bench makes it easy to compare EVM performance in a scalable, standardized, and portable way.

|                         | evmone | geth    | py-evm.cpython | py-evm.pypy | pyrevm  | revm   |
| ----------------------- | ------ | ------- | -------------- | ----------- | ------- | ------ |
| erc20.approval-transfer | 7ms    | 17.4ms  | 1.373s         | 438.6ms     | 16.2ms  | 10.2ms |
| erc20.mint              | 5ms    | 17.2ms  | 1.3878s        | 355.8ms     | 14.4ms  | 6.2ms  |
| erc20.transfer          | 8.2ms  | 24.4ms  | 1.947s         | 451.4ms     | 22.4ms  | 11.4ms |
| snailtracer             | 42ms   | 151ms   | 17.634s        | 4.385s      | 125ms   | 57ms   |
| ten-thousand-hashes     | 2.4ms  | 13.2ms  | 1.9132s        | 329.2ms     | 10.4ms  | 4ms    |
|                         |        |         |                |             |         |        |
| **sum**                 | 64.6ms | 223.2ms | 24.255s        | 5.96s       | 188.4ms | 88.8ms |
| **relative**            | 1.00x  | 3.46x   | 375x           | 92.3x       | 2.92x   | 1.37x  |

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
