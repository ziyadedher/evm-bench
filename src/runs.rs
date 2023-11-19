//! Orchestration for running benchmarks on runners.
//!
//! Manages starting and running Docker containers for each benchmark on each runner. This is the main entry point for
//! running benchmarks. The primary function is [`execute`], which runs all the benchmarks on all the runners and
//! returns a list of [`Run`]s.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::PathBuf;
//!
//! use bollard::Docker;
//! use evm_bench::execute;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let benchmarks_path = PathBuf::from("benchmarks");
//! let runners_path = PathBuf::from("runners");
//!
//! let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
//!
//! let runs = execute(&benchmarks_path, None, &runners_path, None, docker).await.expect("could not run benchmarks");
//! #     Ok(())
//! # }
//! ```

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Error;
use bollard::{
    container::{self, CreateContainerOptions, LogsOptions},
    Docker,
};
use ethers_core::utils::hex::ToHex;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::{
    benchmarks::{
        compile, Benchmark, BenchmarkMetadata, BenchmarkMetadataCost,
        Identifier as BenchmarkIdentifier,
    },
    runners::{build, Identifier as RunnerIdentifier, Runner, RunnerMetadata},
};

/// Unique identifier for this run.
///
/// Typically constructed from the [`RunnerIdentifier`] and [`BenchmarkIdentifier`] of the run.
///
/// # Examples
///
/// ```
/// use evm_bench::runs::Identifier;
/// use evm_bench::runners::Identifier as RunnerIdentifier;
/// use evm_bench::benchmarks::Identifier as BenchmarkIdentifier;
///
/// let identifier = Identifier::from(format!("{}_{}", RunnerIdentifier::from("foo"), BenchmarkIdentifier::from("bar")));
///
/// assert_eq!(identifier.to_string(), "foo_bar");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identifier(String);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Total representation of a run.
///
/// Encapsulates all the relevant information from a benchmarking run. This is the result of running a benchmark on a
/// runner, and contains the durations of each pass of the benchmark. Typically, this is produced by the runnin
/// process using something like the [`execute`] function.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use bollard::Docker;
/// use evm_bench::execute;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let benchmarks_path = PathBuf::from("benchmarks");
/// let runners_path = PathBuf::from("runners");
///
/// let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
/// let runs = execute(&benchmarks_path, None, &runners_path, None, docker).await.expect("could not run benchmarks");
/// #     Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    /// Unique identifier for this run.
    pub identifier: Identifier,
    /// Unique identifier for the runner used in this run.
    pub runner_identifier: RunnerIdentifier,
    /// Unique identifier for the benchmark used in this run.
    pub benchmark_identifier: BenchmarkIdentifier,
    /// Durations of each pass of the benchmark.
    pub durations: Vec<Duration>,
    /// Average run time of the benchmark.
    pub average_duration: Duration,
}

fn num_runs_for_benchmark_cost(cost: BenchmarkMetadataCost) -> u32 {
    match cost {
        BenchmarkMetadataCost::Cheap => 25,
        BenchmarkMetadataCost::Moderate => 10,
        BenchmarkMetadataCost::Expensive => 3,
    }
}

/// Runs a benchmark on a runner.
///
/// Creates a container from the runner's Docker image, runs the benchmark, and parses the output of the benchmark. The
/// output of the benchmark is expected to be a list of durations in microseconds, one per line. The output is parsed
/// and converted to a list of [`Duration`]s which is packaged up with identifiers for the runner and benchmark and
/// returned as a [`Run`].
///
/// # Errors
///
/// If a fatal error occurs during the run, such as the container failing to start or the output of the benchmark not
/// being parseable, then this function will return `None` and log the error.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use bollard::Docker;
/// use evm_bench::{compile, build};
/// use evm_bench::runs::execute_single;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let benchmarks_path = PathBuf::from("benchmarks");
/// let runners_path = PathBuf::from("runners");
///
/// let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
/// let benchmarks = compile(&benchmarks_path, None).expect("could not compile benchmarks");
/// let runners = build(&runners_path, None, docker).await.expect("could not build runners");
/// let runs = execute_single(&benchmarks[0], &runners[0], docker).await.expect("could not run benchmarks");
/// #     Ok(())
/// # }
/// ```
#[allow(clippy::too_many_lines)]
pub async fn execute_single(
    benchmark: &Benchmark,
    runner: &Runner,
    docker: &Docker,
) -> Option<Run> {
    let run_identifier = Identifier(format!("{}_{}", runner.identifier, benchmark.identifier));
    let container_name = format!("emv-bench_{run_identifier}");
    let cmd = vec![
        "--contract-code".to_string(),
        benchmark.bytecode.encode_hex(),
        "--calldata".to_string(),
        benchmark.calldata.encode_hex(),
        "--num-runs".to_string(),
        num_runs_for_benchmark_cost(benchmark.metadata.cost).to_string(),
    ];

    log::debug!(
        "[{run_identifier}] running benchmark ({}) on runner ({})...",
        benchmark.identifier,
        runner.identifier
    );
    log::trace!("[{run_identifier}] arguments: {cmd:#?}");

    let create_response = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name.clone(),
                ..Default::default()
            }),
            container::Config {
                image: Some(runner.docker_image_tag.clone()),
                cmd: Some(cmd),
                ..Default::default()
            },
        )
        .await;
    match create_response {
        Ok(res) => log::debug!(
            "[{run_identifier}] successfully created container with id ({})",
            res.id
        ),
        Err(err) => {
            log::warn!("[{run_identifier}] could not create container: {err}, continuing...");
        }
    }

    let start_response = docker
        .start_container::<String>(&container_name, None)
        .await;
    match start_response {
        Ok(()) => log::debug!("[{run_identifier}] successfully started container",),
        Err(err) => {
            log::warn!("[{run_identifier}] could not start container: {err}, continuing...");
        }
    }

    let wait_response = docker
        .wait_container::<String>(&container_name, None)
        .try_for_each_concurrent(None, |_| async move { Ok(()) })
        .await;

    let (err, container_stdout, container_stderr) = docker
        .logs::<String>(
            &container_name,
            Some(LogsOptions {
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        )
        .fold((None, String::new(), String::new()), |acc, r| async move {
            match r {
                Ok(container::LogOutput::StdOut { message }) => {
                    (acc.0, acc.1 + &String::from_utf8_lossy(&message), acc.2)
                }
                Ok(container::LogOutput::StdErr { message }) => {
                    (acc.0, acc.1, acc.2 + &String::from_utf8_lossy(&message))
                }
                Ok(_) => acc,
                Err(err) => (Some(err.to_string()), acc.1, acc.2),
            }
        })
        .await;

    let result = if let Some(err) = err {
        log::warn!(
            "[{run_identifier}] could not get all container run logs: {err}, continuing...\nstdout:\n{container_stdout}\nstderr:\n{container_stderr}",
        );
        None
    } else if let Err(err) = wait_response {
        log::warn!(
            "[{run_identifier}] container did not finish cleanly: {err}, continuing...\nstdout:\n{container_stdout}\nstderr:\n{container_stderr}",
        );
        None
    } else {
        log::trace!(
            "[{run_identifier}] run logs\nstdout:\n{container_stdout}\nstderr:\n{container_stderr}",
        );
        let result = container_stdout
            .split_whitespace()
            .map(|line| {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Ok::<Duration, Error>(Duration::from_micros(line.parse::<f64>()?.round() as u64))
            })
            .collect::<Result<Vec<_>, Error>>();
        match result {
            Ok(result) => Some(result),
            Err(err) => {
                log::warn!(
                    "[{run_identifier}] could not parse container run output: {err}, continuing...",
                );
                None
            }
        }
    };

    let remove_response = docker.remove_container(&container_name, None).await;
    match remove_response {
        Ok(()) => log::debug!("[{run_identifier}] successfully removed container",),
        Err(err) => {
            log::warn!("[{run_identifier}] could not remove container: {err}, continuing...");
        }
    }

    result.map(|durations| {
        let average_duration = if durations.is_empty() {
            Duration::from_secs(0)
        } else {
            durations.iter().sum::<Duration>() / u32::try_from(durations.len()).unwrap()
        };

        Run {
            identifier: run_identifier.clone(),
            runner_identifier: runner.identifier.clone(),
            benchmark_identifier: benchmark.identifier.clone(),
            durations,
            average_duration,
        }
    })
}

/// Runs all benchmarks on all runners.
///
/// Runs all the benchmarks on all the runners and returns a list of [`Run`]s. This is a convenience function that
/// simply calls [`compile`], [`build`], and [`execute`] in sequence.
///
/// # Errors
///
/// If any of the steps fail, then this function will return an error. This includes errors from [`compile`],
/// [`build`], and [`execute`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use bollard::Docker;
/// use evm_bench::execute;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let benchmarks_path = PathBuf::from("benchmarks");
/// let runners_path = PathBuf::from("runners");
///
/// let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
///
/// let runs = execute(&benchmarks_path, None, &runners_path, None, docker).await.expect("could not run benchmarks");
/// #     Ok(())
/// # }
/// ```
pub async fn execute<'a>(
    benchmarks_path: &Path,
    benchmark_metadatas: Option<BTreeMap<PathBuf, BenchmarkMetadata>>,
    runners_path: &Path,
    runner_metadatas: Option<Vec<(RunnerMetadata, PathBuf)>>,
    docker: &Docker,
) -> anyhow::Result<Vec<Run>> {
    let benchmarks = compile(benchmarks_path, benchmark_metadatas)?;
    let runners = build(runners_path, runner_metadatas, docker).await?;

    log::info!(
        "running {} benchmarks on {} runners...",
        benchmarks.len(),
        runners.len()
    );
    let run_futures = runners.iter().flat_map(|runner| {
        benchmarks
            .iter()
            .map(|benchmark| async { execute_single(benchmark, runner, docker).await })
    });

    // 🔮 This is bad futures usage! We'd typically `join_all` here so we can have all the awaiting for all the futures
    // happen concurrently. However, we want to run the benchmarking sequentially, so we await each future. This pretty
    // much gets rid of all the parallelization benefits, but gives us more stable results with less interference
    // between different benchmarking runs.
    let mut runs = Vec::new();
    for run_future in run_futures {
        if let Some(run) = run_future.await {
            log::info!(
                "[{}] run finished with {} passes (avg: {:?})",
                run.identifier,
                run.durations.len(),
                if run.durations.is_empty() {
                    Duration::from_secs(0)
                } else {
                    run.durations.iter().sum::<Duration>() / u32::try_from(run.durations.len())?
                },
            );
            log::trace!("[{}] run durations: {:#?}", run.identifier, run.durations);
            runs.push(run);
        }
    }

    Ok(runs)
}
