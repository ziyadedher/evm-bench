# evm-bench

[![Rust](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml/badge.svg)](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml)

**evm-bench is a suite of Ethereum Virtual Machine (EVM) stress tests and benchmarks.**

evm-bench makes it easy to compare EVM performance in a scalable, standardized, and portable way.

|                         | evmone | revm    | pyrevm | geth    | py-evm.cpython | ethereumjs |
| ----------------------- | ------ | ------- | ------ | ------- | -------------- | ---------- |
| **sum**                 | 69.2ms | 100.4ms | 218ms  | 231.4ms | 21.7272s       | 31.3376s   |
| **relative**            | 1.000x | 1.451x  | 3.150x | 3.344x  | 313.977x       | 452.855x   |
|                         |        |         |        |         |                |            |
| erc20.approval-transfer | 7.4ms  | 10.2ms  | 21.8ms | 17.4ms  | 1.374s         | 1.8832s    |
| erc20.mint              | 5.2ms  | 6.2ms   | 16.8ms | 18.4ms  | 1.2822s        | 2.8656s    |
| erc20.transfer          | 8.6ms  | 12ms    | 24.4ms | 26ms    | 1.8158s        | 3.3676s    |
| snailtracer             | 45ms   | 67ms    | 143ms  | 157ms   | 15.455s        | 21.592s    |
| ten-thousand-hashes     | 3ms    | 5ms     | 12ms   | 12.6ms  | 1.8002s        | 1.6292s    |

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
