use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use anyhow::Context;
use bollard::{image::BuildImageOptions, Docker};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};

const RUNNER_METADATA_PATTERN: &str = "**/*.runner.json";

typify::import_types!(
    schema = "runners/runner.schema.json",
    patch = { EmvBenchRunnerMetadata = { rename = "RunnerMetadata" } }
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Runner {
    pub identifier: Identifier,
    pub metadata: RunnerMetadata,
    pub docker_image_tag: String,
}

pub async fn build(runners: &Path, docker: &Docker) -> anyhow::Result<Vec<Runner>> {
    log::info!("getting all runner metadata files...");
    let runner_metadatas: Vec<(RunnerMetadata, PathBuf)> = glob::glob(
        runners
            .join(RUNNER_METADATA_PATTERN)
            .to_str()
            .context("could not convert runner metadata pattern to string")?,
    )
    .context("searching for all runner metadata files")?
    .filter_map(|r| {
        let path = r
            .map_err(|err| {
                log::warn!("could not get globbed path: {err}, skipping...");
            })
            .ok()?;

        log::debug!("processing runner metadata file ({})...", path.display());

        let runner_metadata: RunnerMetadata = serde_json::from_reader(
            File::open(&path)
                .map_err(|err| {
                    log::warn!("could not open runner metadata file: {err}, skipping...");
                })
                .ok()?,
        )
        .map_err(|err| {
            log::warn!("could not deserialize runner metadata: {err}, skipping...");
        })
        .ok()?;

        let dockerfile_path = path
            .parent()
            .or_else(|| {
                log::warn!("could not get parent of runner metadata file, skipping...");
                None
            })?
            .join(&runner_metadata.dockerfile)
            .canonicalize()
            .map_err(|err| {
                log::warn!("could not canonicalize dockerfile path: {err}, skipping...");
            })
            .ok()?;

        log::debug!("processed runner metadata file");
        Some((runner_metadata, dockerfile_path))
    })
    .collect();
    log::info!("found {} runner metadata files", runner_metadatas.len());
    log::trace!("runner metadatas: {runner_metadatas:#?}");

    log::info!("building runners...");
    let eventual_runners =
        runner_metadatas
            .into_iter()
            .map(|(metadata, dockerfile_path)| async move {
                let tag = &format!("{}:{}", metadata.name, "latest");

                log::debug!("[{tag}] building runner ({}) image...", metadata.name);

                let context_directory = dockerfile_path.parent().or_else(|| {
                    log::warn!("[{tag}] could not get parent of runner metadata file, skipping...");
                    None
                })?;

                let tarball = {
                    let mut tarball = tar::Builder::new(BufWriter::new(vec![]));
                    tarball
                        .append_dir_all(".", context_directory)
                        .map_err(|err| {
                            log::warn!("[{tag}] could not create tarball: {err}, skipping...");
                        })
                        .ok()?;
                    tarball
                        .into_inner()
                        .map_err(|err| {
                            log::warn!("[{tag}] could not get tarball writer: {err}, skipping...");
                        })
                        .ok()?
                        .into_inner()
                        .map_err(|err| {
                            log::warn!("[{tag}] could not get tarball data: {err}, skipping...");
                        })
                        .ok()?
                        .into()
                };

                docker
                    .build_image(
                        BuildImageOptions {
                            dockerfile: metadata.dockerfile.clone(),
                            t: tag.to_string(),
                            rm: true,
                            ..Default::default()
                        },
                        None,
                        Some(tarball),
                    )
                    .fold((true, String::new()), |acc, r| async move {
                        match r {
                            Ok(build_info) => {
                                (acc.0, acc.1 + &build_info.stream.unwrap_or_default())
                            }
                            Err(err) => (false, acc.1 + &err.to_string()),
                        }
                    })
                    .map({
                        let tag = tag.clone();
                        move |(success, logs)| {
                            if success {
                                log::debug!(
                                    "[{tag}] successfully built runner ({}) image",
                                    metadata.name,
                                );
                                log::trace!("[{tag}] build logs\n{logs}");
                                Some(Runner {
                                    identifier: Identifier(metadata.name.clone()),
                                    metadata,
                                    docker_image_tag: tag.to_string(),
                                })
                            } else {
                                log::warn!(
                                    "[{tag}] failed to build runner ({}) image, skipping...",
                                    metadata.name
                                );
                                log::debug!("[{tag}] build logs\n{logs}");
                                None
                            }
                        }
                    })
                    .await
            });
    let runners: Vec<Runner> = futures::future::join_all(eventual_runners)
        .await
        .into_iter()
        .flatten()
        .collect();
    log::info!("built {} runners", runners.len());
    log::trace!("runners: {runners:#?}");

    Ok(runners)
}
