use std::{error, path::PathBuf, process::exit};

extern crate glob;

use clap::Parser;
use results::{print_results, record_results};

mod build;
mod exec;
mod metadata;
mod results;
mod run;

use crate::{
    build::build_benchmarks,
    exec::validate_executable,
    metadata::{find_benchmarks, find_runners, BenchmarkDefaults},
    run::run_benchmarks_on_runners,
};

/// Ethereum Virtual Machine Benchmark (evm-bench)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to use as the base for benchmarks searching
    #[arg(long, default_value = "./benchmarks")]
    benchmark_search_path: PathBuf,

    /// Path to use as the base for runners searching
    #[arg(short, long, default_value = "./runners")]
    runner_search_path: PathBuf,

    /// Output path for build artifacts and other things
    #[arg(short, long, default_value = "./outputs")]
    output_path: PathBuf,

    /// Name of the output file, will not overwrite.
    /// Default means to use the current datetime.
    #[arg(long, default_value = None)]
    output_file_name: Option<String>,

    /// Path to a Docker executable (this is used for solc)
    #[arg(long, default_value = "docker")]
    docker_executable: PathBuf,

    /// Path to benchmark metadata schema
    #[arg(long, default_value = "./benchmarks/schema.json")]
    benchmark_metadata_schema: PathBuf,

    /// Name of benchmark metadata file to search for
    #[arg(long, default_value = "benchmark.evm-bench.json")]
    benchmark_metadata_name: String,

    /// Path to runner metadata schema
    #[arg(long, default_value = "./runners/schema.json")]
    runner_metadata_schema: PathBuf,

    /// Name of benchmark metadata file to search
    #[arg(long, default_value = "runner.evm-bench.json")]
    runner_metadata_name: String,

    /// Default solc version to use if none specified in the benchmark metadata
    #[arg(long, default_value = "stable")]
    default_solc_version: String,

    /// Default number of runs to use if none specified in the benchmark metadata
    #[arg(long, default_value = "10")]
    default_num_runs: u64,

    /// Default calldata to use if none specified in the benchmark metadata
    #[arg(long, default_value = "")]
    default_calldata_str: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    (|| -> Result<(), Box<dyn error::Error>> {
        let docker_executable = validate_executable("docker", &args.docker_executable)?;
        let _ = validate_executable("cargo", &PathBuf::from("cargo"))?;

        let default_calldata = hex::decode(args.default_calldata_str.to_string())?;

        let benchmarks_path = args.benchmark_search_path.canonicalize()?;
        let benchmarks = find_benchmarks(
            &args.benchmark_metadata_name,
            &args.benchmark_metadata_schema,
            &benchmarks_path,
            BenchmarkDefaults {
                solc_version: args.default_solc_version,
                num_runs: args.default_num_runs,
                calldata: default_calldata,
            },
        )?;

        let runners_path = args.runner_search_path.canonicalize()?;
        let runners = find_runners(
            &args.runner_metadata_name,
            &args.runner_metadata_schema,
            &runners_path,
            (),
        )?;

        let outputs_path = args.output_path.canonicalize()?;

        let builds_path = outputs_path.join("build");
        let built_benchmarks = build_benchmarks(&benchmarks, &docker_executable, &builds_path)?;

        let results = run_benchmarks_on_runners(&built_benchmarks, &runners)?;

        let results_path = outputs_path.join("results");
        let result_file_path = record_results(&results_path, args.output_file_name, &results)?;
        print_results(&result_file_path)?;

        Ok(())
    })()
    .unwrap_or_else(|e| {
        log::error!("{e}");
        exit(-1);
    });
}
