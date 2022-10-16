# evm-bench

[![Rust](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml/badge.svg)](https://github.com/ziyadedher/evm-bench/actions/workflows/rust.yml)

**evm-bench is a suite of Ethereum Virtual Machine (EVM) stress tests and benchmarks.**

evm-bench makes it easy to compare EVM performance in a scalable, standardized, and portable way.

|                         | geth    | py-evm.cpython | py-evm.pypy | pyrevm  | revm   |
| ----------------------- | ------- | -------------- | ----------- | ------- | ------ |
| erc20.approval-transfer | 18.2ms  | 1.3636s        | 439ms       | 16.8ms  | 9ms    |
| erc20.mint              | 16.4ms  | 1.4246s        | 352.6ms     | 14.4ms  | 5.4ms  |
| erc20.transfer          | 23.4ms  | 1.9824s        | 460.8ms     | 22.4ms  | 11ms   |
| snailtracer             | 153ms   | 17.484s        | 5.536s      | 126ms   | 57ms   |
| ten-thousand-hashes     | 17.4ms  | 3.4624s        | 685.2ms     | 22.2ms  | 7.4ms  |
|                         |         |                |             |         |        |
| **sum**                 | 228.4ms | 25.717s        | 7.4736s     | 201.8ms | 89.8ms |
| **relative**            | 2.54x   | 286x           | 83.2x       | 2.25x   | 1.00x  |

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
