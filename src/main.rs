//! This is the main entrypoint for the `evm-bench` CLI tool.
//!
//! This file does not contain any interesting logic beyond parsing CLI arguments and dispatching to the appropriate
//! functions in the library. For more information on how to use the library, see the documentation for the library.
//! For more information on how to use the CLI tool, see the runtime help documentation for the CLI tool.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use bollard::Docker;
use chrono::Utc;
use clap::{Args, Parser, Subcommand};

use evm_bench::{
    benchmarks::{self, BenchmarkMetadata},
    build, compile, execute, read_latest_outputs,
    results::create_markdown_table,
    runners::{self, RunnerMetadata},
    write_outputs,
};

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Args)]
#[group(multiple = false)]
struct BenchmarksIncludeExcludeArgs {
    /// List of benchmarks to include (by ID), if provided only these will be included
    #[arg(long)]
    include_benchmarks: Vec<String>,

    /// List of benchmarks to exclude (by ID), if provided these will be excluded
    #[arg(long)]
    exclude_benchmarks: Vec<String>,
}

#[derive(Args)]
struct BenchmarkArgs {
    /// Path to a directory containing benchmark metadata files
    #[arg(short, long, default_value = "benchmarks")]
    benchmarks: PathBuf,

    #[command(flatten)]
    include_exclude_args: BenchmarksIncludeExcludeArgs,
}

#[derive(Args)]
#[group(multiple = false)]
struct RunnersIncludeExcludeArgs {
    /// List of runners to include (by ID), if provided only these will be included
    #[arg(long)]
    include_runners: Vec<String>,

    /// List of runners to exclude (by ID), if provided these will be excluded
    #[arg(long)]
    exclude_runners: Vec<String>,
}

#[derive(Args)]
struct RunnerArgs {
    /// Path to a directory containing runner metadata files
    #[arg(short, long, default_value = "runners")]
    runners: PathBuf,

    #[command(flatten)]
    include_exclude_args: RunnersIncludeExcludeArgs,
}

#[derive(Args)]
struct OutputArgs {
    /// Path to a directory to dump outputs in
    #[arg(short, long, default_value = "results/outputs")]
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
struct ResultsArgs {
    /// Path to a directory to read outputs from
    #[arg(short, long, default_value = "results/outputs")]
    output: PathBuf,
}

async fn connect_to_docker() -> anyhow::Result<Docker> {
    log::info!("attempting to connect to Docker daemon...");
    let docker =
        Docker::connect_with_local_defaults().context("could not connect to Docker daemon")?;
    let docker_version = &docker
        .version()
        .await
        .context("could not get Docker version, is Docker running?")?;
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

fn construct_filtered_benchmark_metadatas(
    benchmarks: &Path,
    include_exclude_args: &BenchmarksIncludeExcludeArgs,
) -> anyhow::Result<BTreeMap<PathBuf, BenchmarkMetadata>> {
    let BenchmarksIncludeExcludeArgs {
        include_benchmarks: include,
        exclude_benchmarks: exclude,
    } = include_exclude_args;
    let mut benchmark_metadatas = benchmarks::find_all_metadata(benchmarks)?;

    if !include.is_empty() {
        benchmark_metadatas.retain(|_path, metadata| include.contains(&metadata.name));
    } else if !exclude.is_empty() {
        benchmark_metadatas.retain(|_path, metadata| !exclude.contains(&metadata.name));
    }

    Ok(benchmark_metadatas)
}

fn construct_filtered_runner_metadatas(
    runners: &Path,
    include_exclude_args: &RunnersIncludeExcludeArgs,
) -> anyhow::Result<Vec<(RunnerMetadata, PathBuf)>> {
    let RunnersIncludeExcludeArgs {
        include_runners: include,
        exclude_runners: exclude,
    } = include_exclude_args;
    let mut runner_metadatas = runners::find_all_metadata(runners)?;

    if !include.is_empty() {
        runner_metadatas.retain(|(metadata, _path)| include.contains(&metadata.name));
    } else if !exclude.is_empty() {
        runner_metadatas.retain(|(metadata, _path)| !exclude.contains(&metadata.name));
    }

    Ok(runner_metadatas)
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
                benchmark_args:
                    BenchmarkArgs {
                        benchmarks,
                        include_exclude_args,
                    },
            }) => {
                let benchmarks = benchmarks.canonicalize()?;
                let metadatas =
                    construct_filtered_benchmark_metadatas(&benchmarks, &include_exclude_args)?;

                compile(&benchmarks, Some(metadatas))?;
            }
            BuildCommands::Runners(BuildRunnersArgs {
                runner_args:
                    RunnerArgs {
                        runners,
                        include_exclude_args,
                    },
            }) => {
                let runners = runners.canonicalize()?;
                let metadatas =
                    construct_filtered_runner_metadatas(&runners, &include_exclude_args)?;

                build(&runners, Some(metadatas), &connect_to_docker().await?).await?;
            }
            BuildCommands::All(BuildAllArgs {
                benchmark_args:
                    BenchmarkArgs {
                        benchmarks,
                        include_exclude_args: benchmarks_include_exclude_args,
                    },
                runner_args:
                    RunnerArgs {
                        runners,
                        include_exclude_args: runners_include_exclude_args,
                    },
            }) => {
                let benchmarks = benchmarks.canonicalize()?;
                let benchmark_metadatas = construct_filtered_benchmark_metadatas(
                    &benchmarks,
                    &benchmarks_include_exclude_args,
                )?;

                let runners = runners.canonicalize()?;
                let runner_metadatas =
                    construct_filtered_runner_metadatas(&runners, &runners_include_exclude_args)?;

                compile(&benchmarks, Some(benchmark_metadatas))?;
                build(
                    &runners,
                    Some(runner_metadatas),
                    &connect_to_docker().await?,
                )
                .await?;
            }
        },

        Commands::Run(RunArgs {
            benchmark_args:
                BenchmarkArgs {
                    benchmarks,
                    include_exclude_args: benchmarks_include_exclude_args,
                },
            runner_args:
                RunnerArgs {
                    runners,
                    include_exclude_args: runners_include_exclude_args,
                },
            output_args: OutputArgs { output, no_output },
        }) => {
            let benchmarks = benchmarks.canonicalize()?;
            let benchmark_metadatas = construct_filtered_benchmark_metadatas(
                &benchmarks,
                &benchmarks_include_exclude_args,
            )?;

            let runners = runners.canonicalize()?;
            let runner_metadatas =
                construct_filtered_runner_metadatas(&runners, &runners_include_exclude_args)?;

            let runs = execute(
                &benchmarks,
                Some(benchmark_metadatas),
                &runners,
                Some(runner_metadatas),
                &connect_to_docker().await?,
            )
            .await?;

            if !no_output {
                let output = output.canonicalize()?;
                write_outputs(&runs, &output, &start_time)?;
            }
        }

        Commands::Results(ResultsArgs { output }) => {
            let output = output.canonicalize()?;
            let (_, runs) = read_latest_outputs(&output)?;
            println!("{}", create_markdown_table(&runs)?);
        }
    }

    Ok(())
}
