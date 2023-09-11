//! TODO

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::too_many_lines)]

use std::{fs, path::PathBuf};

use anyhow::Context;
use bollard::Docker;
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sysinfo::SystemExt;

use crate::{benchmark::compile, run::run, runner::build};

mod benchmark;
mod run;
mod runner;

#[derive(Parser, Serialize, Deserialize)]
#[command(author, version, about)]
struct Args {
    // Path to a directory containing benchmark metadata files.
    #[arg(short, long, default_value = "benchmarks")]
    benchmarks: PathBuf,

    // Path to a directory containing runner metadata files.
    #[arg(short, long, default_value = "runners")]
    runners: PathBuf,

    // Path to a directory to dump outputs in.
    #[arg(short, long, default_value = "results")]
    output: PathBuf,

    // If false, does not collect system information (e.g. CPU, memory, etc...) in the output.
    #[arg(long, default_value = "true")]
    collect_sysinfo: bool,
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

    let benchmarks = compile(&args.benchmarks.canonicalize()?).map_err(|err| {
        log::error!("{err}");
        err
    })?;
    let runners = build(&args.runners.canonicalize()?, docker)
        .await
        .map_err(|err| {
            log::error!("{err}");
            err
        })?;
    let runs = run(benchmarks.iter(), runners.iter(), docker)
        .await
        .map_err(|err| {
            log::error!("{err}");
            err
        })?;

    let system_info = if sysinfo::System::IS_SUPPORTED {
        if args.collect_sysinfo {
            log::debug!("collecting system information...");
            let mut system_info = sysinfo::System::new_all();
            system_info.refresh_all();
            log::debug!("successfully collected system information");
            log::trace!("system information: {system_info:#?}");
            Some(system_info)
        } else {
            log::info!(
                "user disabled system information collection, not gathering system information"
            );
            None
        }
    } else {
        log::warn!("sysinfo is not supported on this platform, not gathering system information");
        None
    };

    let output = serde_json::to_string_pretty(&json!({
        "metadata": {
            "version": env!("CARGO_PKG_VERSION"),
            "docker": docker_version,
            "timestamp": start_time.to_rfc3339(),
            "command": std::env::args().collect::<Vec<_>>(),
            "args": args,
            "system_information": system_info,
        },
        "benchmarks": benchmarks,
        "runners": runners,
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
