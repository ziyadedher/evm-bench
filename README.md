# evm-bench

[![Rust](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml/badge.svg)](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml)

**evm-bench is a suite of Ethereum Virtual Machine (EVM) stress tests and benchmarks.**

evm-bench makes it easy to compare EVM performance in a scalable, standardized, and portable way.

|                         | evmone | revm   | akula | pyrevm  | geth    | py-evm.pypy | py-evm.cpython | ethereumjs |
| ----------------------- | ------ | ------ | ----- | ------- | ------- | ----------- | -------------- | ---------- |
| **sum**                 | 65.8ms | 91.6ms | 101ms | 189.4ms | 232.4ms | 6.6694s     | 24.0542s       | 146.1274s  |
| **relative**            | 1.00x  | 1.39x  | 1.54x | 2.88x   | 3.53x   | 101x        | 366x           | 2220x      |
|                         |        |        |       |         |         |             |                |            |
| erc20.approval-transfer | 7ms    | 10ms   | 10ms  | 16.6ms  | 17ms    | 399.6ms     | 1.386s         | 2.064s     |
| erc20.mint              | 5ms    | 6.4ms  | 7ms   | 14.4ms  | 17.4ms  | 366.8ms     | 1.398s         | 3.1866s    |
| erc20.transfer          | 9.6ms  | 11.4ms | 12ms  | 22.4ms  | 24.6ms  | 430.8ms     | 2.0182s        | 3.7024s    |
| snailtracer             | 42ms   | 60ms   | 68ms  | 125ms   | 161ms   | 5.148s      | 17.3s          | 134.648s   |
| ten-thousand-hashes     | 2.2ms  | 3.8ms  | 4ms   | 11ms    | 12.4ms  | 324.2ms     | 1.952s         | 2.5264s    |

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
