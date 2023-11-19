//! This is the main entrypoint for the `evm-bench` CLI tool.
//!
//! This file does not contain any interesting logic beyond parsing CLI arguments and dispatching to the appropriate
//! functions in the library. For more information on how to use the library, see the documentation for the library.
//! For more information on how to use the CLI tool, see the runtime help documentation for the CLI tool.

use std::{fs, path::PathBuf};

use anyhow::Context;
use bollard::Docker;
use chrono::Utc;
use clap::{Args, Parser, Subcommand};
use serde_json::json;

use evm_bench::{build_all, compile_all, execute_all};

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Args)]
struct BenchmarkArgs {
    /// Path to a directory containing benchmark metadata files
    #[arg(short, long, default_value = "benchmarks")]
    benchmarks: PathBuf,
}

#[derive(Args)]
struct RunnerArgs {
    /// Path to a directory containing runner metadata files
    #[arg(short, long, default_value = "runners")]
    runners: PathBuf,
}

#[derive(Args)]
struct OutputArgs {
    /// Path to a directory to dump outputs in
    #[arg(short, long, default_value = "results")]
    output: PathBuf,

    /// If true, runs the benchmarks but does not output any results
    #[arg(long, default_value = "false")]
    no_output: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build benchmarks and runners
    Build(BuildArgs),
    /// Run benchmarks on runners
    Run(RunArgs),
    /// Generate and visualize results from runs
    Results(ResultsArgs),
}

#[derive(Args)]
struct BuildArgs {
    #[command(subcommand)]
    cmd: BuildCommands,
}

#[derive(Subcommand)]
enum BuildCommands {
    /// Build benchmarks only
    Benchmarks(BuildBenchmarksArgs),
    /// Build runners only
    Runners(BuildRunnersArgs),
    /// Build benchmarks and runners
    All(BuildAllArgs),
}

#[derive(Args)]
struct BuildBenchmarksArgs {
    #[command(flatten)]
    benchmark_args: BenchmarkArgs,
}

#[derive(Args)]
struct BuildRunnersArgs {
    #[command(flatten)]
    runner_args: RunnerArgs,
}

#[derive(Args)]
struct BuildAllArgs {
    #[command(flatten)]
    benchmark_args: BenchmarkArgs,

    #[command(flatten)]
    runner_args: RunnerArgs,
}

#[derive(Args)]
struct RunArgs {
    #[command(flatten)]
    benchmark_args: BenchmarkArgs,

    #[command(flatten)]
    runner_args: RunnerArgs,

    #[command(flatten)]
    output_args: OutputArgs,
}

#[derive(Args)]
struct ResultsArgs {}

async fn connect_to_docker() -> anyhow::Result<Docker> {
    log::info!("attempting to connect to Docker daemon...");
    let docker =
        Docker::connect_with_local_defaults().context("could not connect to Docker daemon")?;
    let docker_version = &docker
        .version()
        .await
        .context("could not get Docker version")?;
    log::info!(
        "connected to Docker daemon with version {} (api: {}, os/arch: {}/{})",
        docker_version
            .version
            .as_ref()
            .unwrap_or(&"unknown".to_string()),
        docker_version
            .api_version
            .as_ref()
            .unwrap_or(&"unknown".to_string()),
        docker_version.os.as_ref().unwrap_or(&"unknown".to_string()),
        docker_version
            .arch
            .as_ref()
            .unwrap_or(&"unknown".to_string()),
    );

    Ok(docker)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();

    let start_time = Utc::now();

    match args.cmd {
        Commands::Build(BuildArgs { cmd }) => match cmd {
            BuildCommands::Benchmarks(BuildBenchmarksArgs {
                benchmark_args: BenchmarkArgs { benchmarks },
            }) => {
                let benchmarks = benchmarks.canonicalize()?;
                compile_all(&benchmarks)?;
            }
            BuildCommands::Runners(BuildRunnersArgs {
                runner_args: RunnerArgs { runners },
            }) => {
                let runners = runners.canonicalize()?;
                build_all(&runners, &connect_to_docker().await?).await?;
            }
            BuildCommands::All(BuildAllArgs {
                benchmark_args: BenchmarkArgs { benchmarks },
                runner_args: RunnerArgs { runners },
            }) => {
                let benchmarks = benchmarks.canonicalize()?;
                let runners = runners.canonicalize()?;
                compile_all(&benchmarks)?;
                build_all(&runners, &connect_to_docker().await?).await?;
            }
        },

        Commands::Run(RunArgs {
            benchmark_args: BenchmarkArgs { benchmarks },
            runner_args: RunnerArgs { runners },
            output_args: OutputArgs { output, no_output },
        }) => {
            let benchmarks = benchmarks.canonicalize()?;
            let runners = runners.canonicalize()?;

            let runs = execute_all(&benchmarks, &runners, &connect_to_docker().await?)
                .await
                .map_err(|err| {
                    log::error!("{err}");
                    err
                })?;

            if !no_output {
                let output = output.canonicalize()?;
                let results = serde_json::to_string_pretty(&json!({
                    "runs": runs,
                }))?;

                let output_file_path = output.join(format!(
                    "results.{}.json",
                    start_time.format("%Y-%m-%dT%H-%M-%S%z")
                ));
                log::info!(
                    "writing result output to {}...",
                    output_file_path.to_string_lossy()
                );
                fs::create_dir_all(&output)
                    .context("could not create output directory structure")?;
                fs::write(&output_file_path, results).context(format!(
                    "could not write to output file {}",
                    output_file_path.to_string_lossy()
                ))?;
            }
        }

        Commands::Results(_results_args) => {
            log::error!("results subcommand not implemented yet");
        }
    }

    Ok(())
}
