use std::{fs, path::PathBuf};

use anyhow::Context;
use bollard::Docker;
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;

use evm_bench::execute_all;

#[derive(Parser, Serialize, Deserialize)]
#[command(author, version, about)]
struct Args {
    /// Path to a directory containing benchmark metadata files
    #[arg(short, long, default_value = "benchmarks")]
    benchmarks: PathBuf,

    /// Path to a directory containing runner metadata files
    #[arg(short, long, default_value = "runners")]
    runners: PathBuf,

    #[arg(short, long, default_value = "results")]
    /// Path to a directory to dump outputs in
    output: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    env_logger::init();

    let args = Args::parse();

    let start_time = Utc::now();

    log::info!("attempting to connect to Docker daemon...");
    let docker =
        &Docker::connect_with_local_defaults().context("could not connect to Docker daemon")?;
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

    let runs = execute_all(
        &args.benchmarks.canonicalize()?,
        &args.runners.canonicalize()?,
        docker,
    )
    .await
    .map_err(|err| {
        log::error!("{err}");
        err
    })?;

    let output = serde_json::to_string_pretty(&json!({
        "runs": runs,
    }))?;

    let output_file_path = args.output.join(format!(
        "results.{}.json",
        start_time.format("%Y-%m-%dT%H-%M-%S%z")
    ));
    log::info!(
        "writing result output to {}...",
        output_file_path.to_string_lossy()
    );
    fs::create_dir_all(&args.output).context("could not create output directory structure")?;
    fs::write(&output_file_path, output).context(format!(
        "could not write to output file {}",
        output_file_path.to_string_lossy()
    ))?;

    Ok(())
}
