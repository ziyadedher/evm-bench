use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{exit, Command},
    time::Duration,
};

extern crate glob;

use clap::Parser;
use users::{get_current_gid, get_current_uid};

mod metadata;

use metadata::{find_metadata, Benchmark, Runner};

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

    /// Path to a Docker executable (this is used for solc)
    #[arg(long, default_value = "docker")]
    docker_executable: PathBuf,

    /// Path to benchmark metadata schema
    #[arg(long, default_value = "./benchmarks/schema.json")]
    benchmark_metadata_schema: PathBuf,

    /// Name of benchmark metadata file to search for
    #[arg(long, default_value = "evm-bench.benchmark.json")]
    benchmark_metadata_name: String,

    /// Path to runner metadata schema
    #[arg(long, default_value = "./runners/schema.json")]
    runner_metadata_schema: PathBuf,

    /// Name of benchmark metadata file to search
    #[arg(long, default_value = "evm-bench.runner.json")]
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

#[derive(Debug)]
struct RunResult {
    run_times: Vec<Duration>,
}

fn validate_executable_or_exit(name: &str, executable: &Path) -> PathBuf {
    log::debug!("validating executable {} ({name})", executable.display());
    match Command::new(&executable).arg("--version").output() {
        Ok(out) => {
            log::debug!(
                "found {name} ({}): {}",
                executable.display(),
                String::from_utf8(out.stdout)
                    .expect("could not decode program stdout")
                    .trim_end_matches("\n")
            );
            executable.to_path_buf()
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                log::error!("{name} not found, tried {}", executable.display());
                exit(-1);
            }
            _ => {
                log::error!("unknown error: {e}");
                exit(-1);
            }
        },
    }
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let docker_executable = validate_executable_or_exit("docker", &args.docker_executable);

    let benchmarks_path = args.benchmark_search_path.canonicalize().unwrap();
    let runners_path = args.runner_search_path.canonicalize().unwrap();
    let outputs_path = args.output_path.canonicalize().unwrap();
    let builds_path = outputs_path.join("build");
    let results_path = outputs_path.join("results");

    let docker_benchmarks_path = PathBuf::from("/benchmarks");
    let docker_runners_path = PathBuf::from("/runners");
    let docker_outputs_path = PathBuf::from("/outputs");
    let docker_builds_path = docker_outputs_path.join("build");
    let docker_results_path = docker_outputs_path.join("results");

    let benchmarks = find_metadata::<Benchmark>(
        &args.benchmark_metadata_name,
        &args.benchmark_metadata_schema,
        &benchmarks_path,
    )
    .unwrap_or_else(|e| {
        log::error!("could not find benchmarks: {e}");
        exit(-1);
    });
    let runners = find_metadata::<Runner>(
        &args.runner_metadata_name,
        &args.runner_metadata_schema,
        &runners_path,
    )
    .unwrap_or_else(|e| {
        log::error!("could not find runners: {e}");
        exit(-1);
    });

    // TODO: Check that there are no collisions between names.
    // TODO: Pull out most of the below into helpers and more gracefully handle failures.

    let mut results: HashMap<Benchmark, HashMap<Runner, RunResult>> = HashMap::new();

    for benchmark in &benchmarks {
        let benchmark_name = &benchmark.name;
        let solc_version = benchmark
            .solc_version
            .as_ref()
            .unwrap_or(&args.default_solc_version);
        let num_runs = benchmark.num_runs.unwrap_or(args.default_num_runs);
        let contract = &benchmark.contract;

        let default_calldata = hex::decode(args.default_calldata_str.to_string())
            .expect("error parsing default calldata")
            .into();
        let calldata = benchmark.calldata.as_ref().unwrap_or(&default_calldata);

        let contract_name = contract.file_name().unwrap().to_str().unwrap();

        let relative_contract_path = contract.strip_prefix(&benchmarks_path).unwrap();
        let docker_contract_path = docker_benchmarks_path.join(relative_contract_path);

        let build_path = builds_path.join(benchmark_name);
        let docker_build_path = docker_builds_path.join(benchmark_name);

        log::info!(
            "building benchmark {benchmark_name} ({} w/ solc@{solc_version})",
            contract_name
        );

        match Command::new(&docker_executable)
            .arg("run")
            .args([
                "-u",
                &format!("{}:{}", get_current_uid(), get_current_gid()),
            ])
            .args([
                "-v",
                &format!(
                    "{}:{}",
                    benchmarks_path.to_string_lossy(),
                    docker_benchmarks_path.to_string_lossy()
                ),
            ])
            .args([
                "-v",
                &format!(
                    "{}:{}",
                    outputs_path.to_string_lossy(),
                    docker_outputs_path.to_string_lossy()
                ),
            ])
            .arg(format!("ethereum/solc:{solc_version}"))
            .args(["-o", &docker_build_path.to_string_lossy()])
            .args(["--abi", "--bin", "--optimize", "--overwrite"])
            .arg(docker_contract_path)
            .output()
        {
            Ok(out) => {
                log::trace!("stdout: {}", String::from_utf8(out.stdout).unwrap());
                log::trace!("stderr: {}", String::from_utf8(out.stderr).unwrap());

                if out.status.success() {
                    log::debug!("successfully built benchmark {benchmark_name}");
                } else {
                    log::warn!(
                        "could not compile benchmark {benchmark_name}: {}",
                        out.status
                    )
                }
            }
            Err(e) => log::warn!("could not compile benchmark {benchmark_name}: {e}"),
        }

        for runner in &runners {
            let Runner {
                name: runner_name,
                entry,
            } = runner;

            let mut contract_code_path = build_path.join(contract_name);
            contract_code_path.set_extension("bin");

            log::info!("running benchmark {benchmark_name} on runner {runner_name}");
            log::debug!(
                "running {} times using code {:?} with calldata {:?}",
                num_runs,
                contract_code_path.file_name().unwrap(),
                calldata
            );

            match Command::new(entry)
                .args([
                    "--contract-code-path",
                    &contract_code_path.to_string_lossy(),
                ])
                .args(["--calldata", &hex::encode(calldata)])
                .args(["--num-runs", &format!("{}", num_runs)])
                .output()
            {
                Ok(out) => {
                    let stdout = String::from_utf8(out.stdout).unwrap();
                    log::trace!("stdout: {}", stdout);
                    log::trace!("stderr: {}", String::from_utf8(out.stderr).unwrap());

                    if out.status.success() {
                        log::debug!(
                            "successfully ran benchmark {benchmark_name} on runner {runner_name}"
                        );
                        let mut times: Vec<Duration> = Vec::new();
                        for line in stdout.trim().split("\n") {
                            times.push(Duration::from_millis(
                                str::parse::<u64>(line).expect("could not parse output"),
                            ));
                        }
                        results
                            .entry(benchmark.clone())
                            .or_default()
                            .insert(runner.clone(), RunResult { run_times: times });
                    } else {
                        log::warn!(
                            "could not run benchmark {benchmark_name} on runner {runner_name}: {}",
                            out.status
                        )
                    }
                }
                Err(e) => log::warn!(
                    "could not run benchmark {benchmark_name} on runner {runner_name}: {e}"
                ),
            }
        }
    }

    println!("{:?}", results);
}
