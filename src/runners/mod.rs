//! Utilities for creating and working with runners.
//!
//! The primary entrypoint for this module is the [`build`] function, which builds all runners under a given
//! path and returns a vector of [`Runner`] structs.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::PathBuf;
//!
//! use bollard::Docker;
//! use evm_bench::build;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let runners_path = PathBuf::from("runners");
//!
//! let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
//! let runners = build(&runners_path, None, docker).await.expect("could not build runners");
//! #     Ok(())
//! # }
//! ```

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

mod metadata;

pub use metadata::RunnerMetadata;

/// Glob pattern for runner metadata files.
pub const FILE_PATTERN: &str = "**/*.runner.json";

/// Unique identifier for a runner.
///
/// # Examples
///
/// ```
/// use evm_bench::runners::Identifier;
///
/// let identifier = Identifier::from("foo");
///
/// assert_eq!(identifier.to_string(), "foo");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Identifier(String);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Identifier(s.to_string())
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Identifier(s)
    }
}

/// Total representation of a runner.
///
/// Encapsulates all the information needed to launch a runner, the runner can take any [`crate::Benchmark`] and run
/// it. Typically, this is produced by building a runner image using something like the [`build`] function. But it
/// can also be manually constructed in any other way.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use bollard::Docker;
/// use evm_bench::build;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let runners_path = PathBuf::from("runners");
///
/// let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
/// let runners = build(&runners_path, None, docker).await.expect("could not build runners");
/// #     Ok(())
/// # }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Runner {
    /// Unique identifier for this runner.
    pub identifier: Identifier,
    /// Metadata for this runner.
    pub metadata: RunnerMetadata,
    /// Tag for the built docker image for this runner.
    pub docker_image_tag: String,
}

/// Finds all runner metadata files under the given path.
///
/// Searches for all files matching the [`FILE_PATTERN`] pattern under the given path and attempts to
/// deserialize them into [`RunnerMetadata`] structs. Returns a vector of the metadata for the runner and the path to
/// the Dockerfile to be built.
///
/// # Errors
///
/// If the glob pattern cannot be constructed or the glob search fails, then the error is returned.
///
/// If any of the files matching the pattern cannot be opened, deserialized, or canonicalized, then the error is
/// logged and the file is skipped.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use evm_bench::runners::find_all_metadata;
///
/// let path = PathBuf::from("runners");
///
/// let metadata = find_all_metadata(&path);
/// ```
pub fn find_all_metadata(path: &Path) -> anyhow::Result<Vec<(RunnerMetadata, PathBuf)>> {
    log::info!("getting all runner metadata files...");
    let metadatas: Vec<(RunnerMetadata, PathBuf)> = glob::glob(
        path.join(FILE_PATTERN)
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

        let metadata: RunnerMetadata = serde_json::from_reader(
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
            .join(&metadata.dockerfile)
            .canonicalize()
            .map_err(|err| {
                log::warn!("could not canonicalize dockerfile path: {err}, skipping...");
            })
            .ok()?;

        log::debug!("processed runner metadata file");
        Some((metadata, dockerfile_path))
    })
    .collect();
    log::info!("found {} runner metadata files", metadatas.len());
    log::trace!("runner metadatas: {metadatas:#?}");

    Ok(metadatas)
}

/// Builds a runner image from the given metadata and Dockerfile path.
///
/// The Dockerfile path is mostly just used to get the Dockerfile context, the metadata is used as the source of truth
/// for most things. The runner image is tagged with the name of the runner and the tag `latest`, for now. This is not a guarantee.
///
/// # Errors
///
/// If the Docker image cannot be built for whatever reason, the error is logged and `None` is returned.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use bollard::Docker;
/// use evm_bench::runners::{build_single, RunnerMetadata};
///
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let metadata = RunnerMetadata {
///    name: "foo".to_string(),
///    dockerfile: "Dockerfile".to_string()
/// };
///
/// let dockerfile_path = PathBuf::from("runners/foo/Dockerfile");
///
/// let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
/// let runner = build(metadata, &dockerfile_path, docker).await.expect("could not build runners");
///
/// assert_eq!(runner.identifier.to_string(), "foo");
/// #     Ok(())
/// # }
/// ```
pub async fn build_single(
    metadata: RunnerMetadata,
    dockerfile_path: &Path,
    docker: &Docker,
) -> Option<Runner> {
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
                Ok(build_info) => (acc.0, acc.1 + &build_info.stream.unwrap_or_default()),
                Err(err) => (false, acc.1 + &err.to_string()),
            }
        })
        .map({
            let tag = tag.clone();
            move |(success, logs)| {
                if success {
                    log::info!(
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
}

/// Builds all runner images under the given path.
///
/// If the optional `metadatas` argument is provided, then it will be used to build the runner images. Otherwise, it
/// will search for all runner metadata files under the given path using [`find_all_metadata`]. Returns a vector of all
/// the successfully built runners as [`Runner`] structs.
///
/// # Errors
///
/// If any of the runner images cannot be built for whatever reason, the error is logged and the runner is skipped.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use bollard::Docker;
/// use evm_bench::runners::build;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let runners_path = PathBuf::from("runners");
///
/// let docker = &Docker::connect_with_local_defaults().expect("could not connect to Docker daemon");
/// let runners = build(&runners_path, None, docker).await.expect("could not build runners");
/// #     Ok(())
/// # }
/// ```
pub async fn build(
    path: &Path,
    metadatas: Option<Vec<(RunnerMetadata, PathBuf)>>,
    docker: &Docker,
) -> anyhow::Result<Vec<Runner>> {
    let metadatas = if let Some(metadatas) = metadatas {
        metadatas
    } else {
        find_all_metadata(path)?
    };

    log::info!("building runners...");
    let runners: Vec<Runner> = futures::future::join_all(metadatas.into_iter().map(
        |(metadata, dockerfile_path)| async move {
            build_single(metadata, &dockerfile_path, docker).await
        },
    ))
    .await
    .into_iter()
    .flatten()
    .collect();
    log::info!("built {} runners", runners.len());
    log::trace!("runners: {runners:#?}");

    Ok(runners)
}
