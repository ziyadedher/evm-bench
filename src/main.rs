//! TODO

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

use std::path::PathBuf;

use anyhow::Context;
use bollard::{
    container::{self, CreateContainerOptions, LogsOptions},
    Docker,
};
use clap::Parser;
use ethers_core::utils::hex::ToHex;
use futures::TryStreamExt;

use crate::{benchmarks::compile, runners::build};

mod benchmarks;
mod runners;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    benchmarks: PathBuf,

    #[arg(short, long)]
    runners: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();
    env_logger::init();

    let args = Args::parse();

    log::info!("attempting to connect to Docker daemon...");
    let docker =
        &Docker::connect_with_local_defaults().context("could not connect to Docker daemon")?;
    let docker_version = docker
        .version()
        .await
        .context("could not get Docker version")?;
    log::info!(
        "connected to Docker daemon with version {} (api: {}, os/arch: {}/{})",
        docker_version.version.unwrap_or_default(),
        docker_version.api_version.unwrap_or_default(),
        docker_version.os.unwrap_or_default(),
        docker_version.arch.unwrap_or_default()
    );

    let benchmarks = &compile(&args.benchmarks.canonicalize()?).map_err(|err| {
        log::error!("{err}");
        err
    })?;
    let runners = &build(&args.runners.canonicalize()?, docker)
        .await
        .map_err(|err| {
            log::error!("{err}");
            err
        })?;

    let _results: Vec<_> = futures::future::join_all(runners.iter().flat_map(|runner| {
        benchmarks.iter().map(|benchmark| async {
            let container_name =
                format!("emv-bench-{}-{}", runner.identifier, benchmark.identifier);
            let cmd = vec![
                "--contract-code".to_string(),
                benchmark.bytecode.encode_hex(),
                "--calldata".to_string(),
                benchmark.calldata.encode_hex(),
                "--num-runs".to_string(),
                "1".to_string(),
            ];

            log::info!(
                "[{container_name}] running benchmark ({}) on runner ({})...",
                benchmark.identifier,
                runner.identifier
            );
            log::trace!("[{container_name}] arguments: {cmd:#?}");

            docker
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
                .await
                .map_err(|err| {
                    log::warn!("could not create container ({container_name}): {err}, skipping...");
                })
                .ok()?;

            docker
                .start_container::<String>(&container_name, None)
                .await
                .map_err(|err| {
                    log::warn!("could not start container ({container_name}): {err}, skipping...");
                })
                .ok()?;

            docker.logs::<String>(
                &container_name,
                Some(LogsOptions {
                    ..Default::default()
                }),
            );

            docker
                .wait_container::<String>(&container_name, None)
                .try_for_each_concurrent(None, |_| async move { Ok(()) })
                .await
                .map_err(|err| {
                    log::warn!(
                        "could not wait for container ({container_name}): {err}, skipping..."
                    );
                })
                .ok()?;

            docker
                .remove_container(&container_name, None)
                .await
                .map_err(|err| {
                    log::warn!("could not remove container ({container_name}): {err}, skipping...");
                })
                .ok()?;

            Some(())
        })
    }))
    .await;

    Ok(())
}
