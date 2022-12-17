# evm-bench

[![Rust](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml/badge.svg)](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml)

**evm-bench is a suite of Ethereum Virtual Machine (EVM) stress tests and benchmarks.**

evm-bench makes it easy to compare EVM performance in a scalable, standardized, and portable way.

|                         | evmone | revm   | pyrevm | geth   | py-evm.pypy | py-evm.cpython | ethereumjs |
| ----------------------- | ------ | ------ | ------ | ------ | ----------- | -------------- | ---------- |
| **sum**                 | 66ms   | 84.8ms | 194ms  | 235ms  | 7.201s      | 19.0886s       | 146.3218s  |
| **relative**            | 1.000x | 1.285x | 2.939x | 3.561x | 109.106x    | 289.221x       | 2216.997x  |
|                         |        |        |        |        |             |                |            |
| erc20.approval-transfer | 7ms    | 9.6ms  | 16.2ms | 17ms   | 425.2ms     | 1.13s          | 2.0006s    |
| erc20.mint              | 5ms    | 6.4ms  | 14.8ms | 17.2ms | 334ms       | 1.1554s        | 3.1352s    |
| erc20.transfer          | 8.6ms  | 11.6ms | 22.8ms | 24.6ms | 449.2ms     | 1.6172s        | 3.6564s    |
| snailtracer             | 43ms   | 53ms   | 128ms  | 163ms  | 5.664s      | 13.675s        | 135.059s   |
| ten-thousand-hashes     | 2.4ms  | 4.2ms  | 12.2ms | 13.2ms | 328.6ms     | 1.511s         | 2.4706s    |

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
