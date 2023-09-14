//! Modular benchmarking framework for Ethereum Virtual Machine implementations.
//!
//! evm-bench is a framework for benchmarking diverse Ethereum Virtual Machine (EVM) implementations in a
//! platform-agnostic manner. It is designed to be modular and extensible, allowing for easy integration of new EVM
//! implementations and benchmarks.
//!
//! # Suite
//! evm-bench includes a suite of runners and benchmarks that can be found in the [GitHub repository][repo], but the
//! framework can theoretically be used with any custom runners or benchmarks as long as they implement the required
//! interface and have the proper metadata.
//!
//! ## Building a suite
//! Refer to the documentation in the [GitHub repository][repo] for information on how to build runners and benchmarks:
//! - [Learn more about building runners](https://github.com/ziyadedher/evm-bench/blob/main/runners)
//! - [Learn more about building benchmarks](https://github.com/ziyadedher/evm-bench/blob/main/benchmarks)
//!
//!
//! # Usage
//! evm-bench is primarily designed to be used as an executable, but it is modular and can also be used as a library
//! for integration into a larger system or more granular control over the benchmarking scope and process.
//!
//! ## As an executable
//! Refer to the output of the `--help` flag for information on how to use the evm-bench binary:
//! ```console
//! $ cargo install evm-bench
//! $ evm-bench --help
//! 🚀🪑 evm-bench is a suite of Ethereum Virtual Machine stress tests and benchmarks.
//!
//! Usage: evm-bench [OPTIONS]
//!
//! Options:
//!   -b, --benchmarks <BENCHMARKS>  Path to a directory containing benchmark metadata files [default: benchmarks]
//!   -r, --runners <RUNNERS>        Path to a directory containing runner metadata files [default: runners]
//!   -o, --output <OUTPUT>          Path to a directory to dump outputs in [default: results]
//!       --collect-sysinfo          If true, collects system information (e.g. CPU, memory, etc...) in the output
//!   -h, --help                     Print help
//!   -V, --version                  Print version
//! ```
//!
//! Note that the executable is not shipped with the existing [suite](#suite) of runners and benchmarks. You need to
//! either [build your own suite](#build-a-suite) and pass the directories containing the metadata files to the
//! executable or clone the [repository][repo] and run the executable with the default parameters to run the
//! [existing suite](#suite).
//!
//! ## As a library
//! ```no_run
//! use std::path::PathBuf;
//!
//! use bollard::Docker;
//! use evm_bench::execute_all;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let benchmarks_path = PathBuf::from("benchmarks");
//! let runners_path = PathBuf::from("runners");
//!
//! let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
//! let runs = execute_all(&benchmarks_path, &runners_path, docker).await.expect("could not run benchmarks");
//! #     Ok(())
//! # }
//! ```
//!
//! # Examples
//! ## Using Arbitrary Runners and Benchmarks
//!
//!
//! ## Using the Existing Suite
//! The easiest way to reproduce the benchmark results is to clone the [repository][repo] and run the evm-bench binary
//! with default parameters. This will run the included suite of runners and benchmarks and dump the results in the
//! [`results`][results] directory.
//!
//! ```console
//! $ git clone https://github.com/ziyadedher/evm-bench
//! $ cd evm-bench
//! $ RUST_LOG=info cargo run --release --
//! ```
//!
//! TODO(ziyadedher): add information about visualizing when that's a thing.
//!
//! TODO(ziyadedher): add information about running a custom suite.
//!
//!
//! # Results
//! This project also periodically publishes run results for the included suite of runners and benchmarks. These can
//! also be found in the [GitHub repository][repo]. There is a nice visualization of the results in the root README of
//! the repository, but the raw results can also be found under the [`results` directory][results].
//!
//! [repo]: https://github.com/ziyadedher/evm-bench
//! [results]: https://github.com/ziyadedher/evm-bench/tree/main/results

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

pub mod benchmarks;
pub mod runners;
pub mod runs;

pub use benchmarks::{compile_all, Benchmark};
pub use runners::{build_all, Runner};
pub use runs::execute_all;
