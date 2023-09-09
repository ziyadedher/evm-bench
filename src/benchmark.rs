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

const BENCHMARK_METADATA_PATTERN: &str = "**/*.benchmark.json";

typify::import_types!(
    schema = "benchmarks/benchmark.schema.json",
    patch = {
        EmvBenchBenchmarkMetadata = { rename = "BenchmarkMetadata" },
        EmvBenchBenchmarkMetadataCost = { rename = "BenchmarkMetadataCost" },
    }
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Benchmark {
    pub identifier: Identifier,
    pub metadata: BenchmarkMetadata,
    pub solc_version: Version,
    pub source_path: PathBuf,
    pub bytecode: Bytes,
    pub calldata: Bytes,
}

pub fn compile(benchmarks: &Path) -> anyhow::Result<Vec<Benchmark>> {
    log::info!("getting all benchmark metadata files...");
    let benchmark_metadatas: BTreeMap<PathBuf, BenchmarkMetadata> = glob::glob(
        benchmarks
            .join(BENCHMARK_METADATA_PATTERN)
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

        let benchmark_metadata: BenchmarkMetadata = serde_json::from_reader(
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
            .join(&benchmark_metadata.contract)
            .canonicalize()
            .map_err(|err| {
                log::warn!("could not canonicalize source path: {err}, skipping...");
            })
            .ok()?;

        log::debug!("processed benchmark metadata file");
        Some((source_path, benchmark_metadata))
    })
    .collect();
    log::info!(
        "found {} benchmark metadata files",
        benchmark_metadatas.len()
    );
    log::trace!("benchmark metadatas: {benchmark_metadatas:#?}");

    log::info!("compiling benchmarks...");
    let benchmarks: Vec<Benchmark> = Project::builder()
        .paths(ProjectPathsConfig::builder().root(benchmarks).build()?)
        .include_path(benchmarks)
        .build()?
        .compile()?
        .into_artifacts()
        .filter_map(|(artifact_id, artifact)| {
            log::debug!("processing artifact ({})...", artifact_id.identifier());

            let bytecode = artifact
                .get_deployed_bytecode_bytes()
                .filter(|bytecode| !bytecode.is_empty())
                .or_else(|| {
                    log::debug!("no deployed bytecode, skipping...",);
                    None
                })?;

            let source_path = artifact_id
                .source
                .canonicalize()
                .map_err(|err| log::warn!("could not canonicalize source path: {err}, skipping..."))
                .ok()?;
            let metadata = benchmark_metadatas.get(&source_path).or_else(|| {
                log::warn!(
                    "could not find benchmark metadata for {}, skipping...",
                    source_path.display()
                );
                None
            })?;

            log::debug!("processed artifact");

            Some(Benchmark {
                identifier: Identifier(metadata.name.clone()),
                metadata: metadata.clone(),
                solc_version: artifact_id.version,
                source_path,
                bytecode: bytecode.into_owned(),
                calldata: Bytes::from_hex(&metadata.calldata)
                    .map_err(|err| {
                        log::warn!("could not hex decode calldata: {err}, skipping...");
                    })
                    .ok()?,
            })
        })
        .collect();
    log::info!("compiled {} benchmarks", benchmarks.len());
    log::trace!("benchmarks: {benchmarks:#?}");

    Ok(benchmarks)
}
