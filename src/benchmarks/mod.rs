//! Utilities for creating and working with benchmarks.
//!
//! The primary entrypoint for this module is the [`compile`] function, which compiles all benchmarks under a given
//! path and returns a vector of [`Benchmark`] structs.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::PathBuf;
//!
//! use evm_bench::benchmarks::compile;
//!
//! let path = PathBuf::from("benchmarks");
//!
//! let benchmarks = compile(&path, None);
//! ```

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ethers_core::{types::Bytes, utils::hex::FromHex};
use ethers_solc::{Artifact, Project, ProjectPathsConfig};
use semver::Version;
use serde::{Deserialize, Serialize};

mod metadata;

pub use metadata::{BenchmarkMetadata, BenchmarkMetadataCost};

/// Glob pattern for benchmark metadata files.
pub const FILE_PATTERN: &str = "**/*.benchmark.json";

/// Unique identifier for a benchmark.
///
/// # Examples
///
/// ```
/// use evm_bench::benchmarks::Identifier;
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
        Self(s.to_string())
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Total representation of a benchmark.
///
/// Encapsulates all the information needed to execute a benchmark, any [`crate::Runner`] can take this struct and run
/// the benchmark with no additional data needed. Typically, this is produced by the compilation process of the
/// benchmark, like using the [`compile`] function. But it can also be manually constructed in any other way.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use evm_bench::benchmarks::compile;
///
/// let path = PathBuf::from("benchmarks");
///
/// let benchmarks = compile(&path, None);
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Benchmark {
    /// Unique identifier for this benchmark.
    pub identifier: Identifier,
    /// Metadata for this benchmark.
    pub metadata: BenchmarkMetadata,
    /// Version of the Solidity compiler used to compile this benchmark.
    pub solc_version: Version,
    /// Path to the source Solidity file that was compiled to produce this benchmark.
    pub source_path: PathBuf,
    /// Deployed bytecode for this benchmark. This is _not_ contract creation code.
    pub bytecode: Bytes,
    /// The calldata to be used to run this benchmark.
    pub calldata: Bytes,
}

/// Finds all benchmark metadata files under the given path.
///
/// Searches for all files matching the [`FILE_PATTERN`] pattern under the given path and attempts to
/// deserialize them into [`BenchmarkMetadata`] structs. Returns a map of the source path of the target Solidity file
/// to the metadata for that benchmark.
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
/// use evm_bench::benchmarks::find_all_metadata;
///
/// let path = PathBuf::from("benchmarks");
///
/// let metadata = find_all_metadata(&path);
/// ```
pub fn find_all_metadata(path: &Path) -> anyhow::Result<BTreeMap<PathBuf, BenchmarkMetadata>> {
    log::info!(
        "finding all benchmark metadata files under {}...",
        path.display()
    );
    let metadatas: BTreeMap<PathBuf, BenchmarkMetadata> = glob::glob(
        path.join(FILE_PATTERN)
            .to_str()
            .context("could not convert benchmark metadata pattern to string")?,
    )
    .context("searching for all benchmark metadata files")?
    .filter_map(|r| {
        let path = r
            .map_err(|err| {
                log::warn!("could not get globbed path: {err}, skipping...");
            })
            .ok()?;

        log::debug!("processing benchmark metadata file ({})...", path.display());

        let metadata: BenchmarkMetadata = serde_json::from_reader(
            File::open(&path)
                .map_err(|err| {
                    log::warn!("could not open benchmark metadata file: {err}, skipping...");
                })
                .ok()?,
        )
        .map_err(|err| {
            log::warn!("could not deserialize benchmark metadata: {err}, skipping...");
        })
        .ok()?;

        let source_path = path
            .parent()
            .or_else(|| {
                log::warn!("could not get parent of benchmark metadata file, skipping...");
                None
            })?
            .join(&metadata.contract)
            .canonicalize()
            .map_err(|err| {
                log::warn!("could not canonicalize source path: {err}, skipping...");
            })
            .ok()?;

        log::debug!("processed benchmark metadata file");
        Some((source_path, metadata))
    })
    .collect();
    log::info!("found {} benchmark metadata files", metadatas.len());
    log::trace!("benchmark metadatas: {metadatas:#?}");

    Ok(metadatas)
}

/// Compiles all benchmarks under the given path.
///
/// Compiles all the Solidity files under the given path using [`ethers_solc`]. It then filters the artifacts to only
/// include those that have associated benchmark metadata and returns a vector of [`Benchmark`] structs.
///
/// If the optional `metadatas` argument is provided, then it will be used to filter the artifacts to only include those
/// that have associated benchmark metadata. Otherwise, it will search for all benchmark metadata files under the given
/// path using [`find_all_metadata`].
///
/// # Errors
///
/// If benchmark metadata cannot be found or the compilation process fails, then the error is returned.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use evm_bench::benchmarks::compile;
///
/// let path = PathBuf::from("benchmarks");
///
/// let benchmarks = compile(&path, None);
/// ```
pub fn compile(
    benchmarks: &Path,
    metadatas: Option<BTreeMap<PathBuf, BenchmarkMetadata>>,
) -> anyhow::Result<Vec<Benchmark>> {
    let metadatas = if let Some(metadatas) = metadatas {
        metadatas
    } else {
        find_all_metadata(benchmarks)?
    };

    log::info!("compiling benchmarks...");
    let benchmarks: Vec<Benchmark> = Project::builder()
        .paths(ProjectPathsConfig::builder().root(benchmarks).build()?)
        .include_path(benchmarks)
        .build()?
        .compile()?
        .into_artifacts()
        .filter_map(|(artifact_id, artifact)| {
            log::debug!("processing artifact ({})...", artifact_id.source.display());
            let source_path = artifact_id
                .source
                .canonicalize()
                .map_err(|err| log::warn!("could not canonicalize source path: {err}, skipping..."))
                .ok()?;
            let metadata = metadatas.get(&source_path).or_else(|| {
                log::debug!(
                    "could not find benchmark metadata for {}, skipping...",
                    source_path.display()
                );
                None
            })?;

            let identifier = Identifier(metadata.name.clone());

            let bytecode = artifact
                .get_deployed_bytecode_bytes()
                .filter(|bytecode| !bytecode.is_empty())
                .or_else(|| {
                    log::debug!("[{}] no deployed bytecode, skipping...", identifier);
                    None
                })?;
            let calldata = Bytes::from_hex(&metadata.calldata)
                .map_err(|err| {
                    log::warn!(
                        "[{}] could not hex decode calldata: {err}, skipping...",
                        identifier
                    );
                })
                .ok()?;

            log::info!(
                "[{}] successfully compiled benchmark and proccessed artifacts",
                identifier
            );

            Some(Benchmark {
                identifier: identifier.clone(),
                metadata: metadata.clone(),
                solc_version: artifact_id.version,
                source_path,
                bytecode: bytecode.into_owned(),
                calldata,
            })
        })
        .collect();
    log::info!("compiled {} benchmarks", benchmarks.len());
    log::trace!("benchmarks: {benchmarks:#?}");

    Ok(benchmarks)
}
