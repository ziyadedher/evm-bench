use std::{
    fmt::{self, Display, Formatter},
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
    benchmark::{Benchmark, Identifier as BenchmarkIdentifier},
    runner::{Identifier as RunnerIdentifier, Runner},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub identifier: Identifier,
    pub runner_identifier: RunnerIdentifier,
    pub benchmark_identifier: BenchmarkIdentifier,
    pub durations: Vec<Duration>,
}

pub async fn run<'a>(
    benchmarks: impl Iterator<Item = &'a Benchmark> + Clone,
    runners: impl Iterator<Item = &'a Runner> + Clone,
    docker: &Docker,
) -> anyhow::Result<Vec<Run>> {
    log::info!(
        "running {} benchmarks on {} runners...",
        benchmarks.clone().count(),
        runners.clone().count()
    );
    let run_futures = runners.flat_map(|runner| {
        benchmarks.clone().map(|benchmark| async {
            let run_identifier = Identifier(format!(
                "{}_{}",
                runner.identifier, benchmark.identifier
            ));
            let container_name =
                format!("emv-bench_{run_identifier}");
            let cmd = vec![
                "--contract-code".to_string(),
                benchmark.bytecode.encode_hex(),
                "--calldata".to_string(),
                benchmark.calldata.encode_hex(),
                "--num-runs".to_string(),
                "10".to_string(),
            ];

            log::debug!(
                "[{run_identifier}] running benchmark ({}) on runner ({})...",
                benchmark.identifier.0,
                runner.identifier.0
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
                    "[{run_identifier}] successfully created container with id ({})", res.id
                ),
                Err(err) => log::warn!(
                    "[{run_identifier}] could not create container: {err}, continuing...",
                ),
            }

            let start_response = docker
                .start_container::<String>(&container_name, None)
                .await;
            match start_response {
                Ok(()) => log::debug!(
                    "[{run_identifier}] successfully started container",
                ),
                Err(err) => log::warn!(
                    "[{run_identifier}] could not start container: {err}, continuing...",
                ),
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
                let result = container_stdout.split_whitespace().map(|line| {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    Ok::<Duration, Error>(Duration::from_micros(line.parse::<f64>()?.round() as u64))
                }).collect::<Result<Vec<_>, Error>>();
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
                Ok(()) => log::debug!(
                    "[{run_identifier}] successfully removed container",
                ),
                Err(err) => log::warn!(
                    "[{run_identifier}] could not remove container: {err}, continuing...",
                ),
            }

            result.map(|durations| Run {
                identifier: run_identifier.clone(),
                runner_identifier: runner.identifier.clone(),
                benchmark_identifier: benchmark.identifier.clone(),
                durations,
            })
        })
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
