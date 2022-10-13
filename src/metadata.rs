use std::{
    collections::HashSet,
    error, fs,
    path::{Path, PathBuf},
};

use glob::glob;
use serde::{Deserialize, Serialize};

pub trait MetadataParser
where
    Self: Sized,
{
    type Defaults;

    fn parse_schema_from_file(
        schema_path: &Path,
    ) -> Result<serde_json::Value, Box<dyn error::Error>> {
        let schema_file = fs::File::open(schema_path)?;
        Ok(serde_json::from_reader(&schema_file)?)
    }

    fn parse_from_file(
        schema: &serde_json::Value,
        json_path: &Path,
        defaults: &Self::Defaults,
    ) -> Result<Self, Box<dyn error::Error>> {
        let json_file = fs::File::open(json_path)?;
        let json = serde_json::from_reader(&json_file)?;
        Self::parse(
            json_path.parent().ok_or("could not get parent")?,
            schema,
            &json,
            defaults,
        )
    }

    fn parse(
        base_path: &Path,
        schema: &serde_json::Value,
        json: &serde_json::Value,
        defaults: &Self::Defaults,
    ) -> Result<Self, Box<dyn error::Error>> {
        if jsonschema::is_valid(schema, json) {
            Self::parse_inner(base_path, json, defaults)
        } else {
            Err("json does not abide by the schema".into())
        }
    }

    fn parse_inner(
        base_path: &Path,
        json: &serde_json::Value,
        defaults: &Self::Defaults,
    ) -> Result<Self, Box<dyn error::Error>>;
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Benchmark {
    pub name: String,
    pub solc_version: String,
    pub num_runs: u64,
    pub contract: PathBuf,
    pub build_context: PathBuf,
    pub calldata: Vec<u8>,
}

pub struct BenchmarkDefaults {
    pub solc_version: String,
    pub num_runs: u64,
    pub calldata: Vec<u8>,
}

impl MetadataParser for Benchmark {
    type Defaults = BenchmarkDefaults;

    fn parse_inner(
        base_path: &Path,
        json: &serde_json::Value,
        defaults: &Self::Defaults,
    ) -> Result<Self, Box<dyn error::Error>> {
        log::trace!("parsing benchmark metadata...");
        let object = json.as_object().expect("could not parse json as object");
        let benchmark = Self {
            name: object
                .get("name")
                .ok_or("could not find name")?
                .as_str()
                .ok_or("could not parse name as string")?
                .to_string(),
            solc_version: object
                .get("solc-version")
                .map_or(
                    Ok::<&str, Box<dyn error::Error>>(&defaults.solc_version),
                    |x| Ok(x.as_str().ok_or("could not parse solc-version as string")?),
                )?
                .to_string(),
            num_runs: object
                .get("num-runs")
                .map_or(Ok::<u64, Box<dyn error::Error>>(defaults.num_runs), |x| {
                    Ok(x.as_u64().ok_or("could not parse num-runs as u64")?)
                })?,
            contract: base_path
                .join(PathBuf::from(
                    object
                        .get("contract")
                        .ok_or("could not find contract")?
                        .as_str()
                        .ok_or("could not parse contract as string")?,
                ))
                .canonicalize()?,
            build_context: base_path
                .join(PathBuf::from(object.get("build-context").map_or(
                    Ok::<String, Box<dyn error::Error>>(".".into()),
                    |x| {
                        Ok(x.as_str()
                            .ok_or("could not parse build-context as string")?
                            .to_string())
                    },
                )?))
                .canonicalize()?,
            calldata: object.get("calldata").map_or(
                Ok::<Vec<u8>, Box<dyn error::Error>>(defaults.calldata.clone()),
                |x| {
                    Ok(hex::decode(
                        x.as_str().ok_or("could not parse calldata as bytes")?,
                    )?)
                },
            )?,
        };
        log::debug!("parsed benchmark metadata: {}", &benchmark.name);
        log::trace!("benchmark metadata: {:?}", benchmark);
        Ok(benchmark)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Runner {
    pub name: String,
    pub entry: PathBuf,
}

impl MetadataParser for Runner {
    type Defaults = ();

    fn parse_inner(
        base_path: &Path,
        json: &serde_json::Value,
        _: &Self::Defaults,
    ) -> Result<Self, Box<dyn error::Error>> {
        log::trace!("parsing runner metadata...");
        let object = json.as_object().expect("could not parse json as object");
        let runner = Self {
            name: object
                .get("name")
                .ok_or("could not find name")?
                .as_str()
                .ok_or("could not parse name as string")?
                .to_string(),
            entry: base_path
                .join(PathBuf::from(
                    object
                        .get("entry")
                        .ok_or("could not find entry")?
                        .as_str()
                        .ok_or("could not parse entry as string")?,
                ))
                .canonicalize()?,
        };
        log::debug!("parsed runner metadata: {}", &runner.name);
        log::trace!("runner metadata: {:?}", runner);
        Ok(runner)
    }
}

fn find_metadata<T: MetadataParser>(
    file_name: &str,
    schema_path: &Path,
    search_path: &Path,
    defaults: T::Defaults,
) -> Result<Vec<T>, Box<dyn error::Error>> {
    let schema = Benchmark::parse_schema_from_file(schema_path)?;

    let search_path = search_path.canonicalize()?;
    if !search_path.is_dir() {
        return Err(format!("{} is not a directory", search_path.display()).into());
    }

    Ok(
        glob(&search_path.join("**").join(file_name).to_string_lossy())?
            .flat_map(|entry| match entry {
                Ok(path) => {
                    log::debug!(
                        "found {}",
                        path.strip_prefix(&search_path).unwrap_or(&path).display()
                    );
                    Some(path)
                }
                Err(e) => {
                    log::warn!("error globing file: {:?}", e);
                    None
                }
            })
            .flat_map(|path| match T::parse_from_file(&schema, &path, &defaults) {
                Ok(res) => {
                    log::debug!(
                        "parsed {}",
                        path.strip_prefix(&search_path).unwrap_or(&path).display()
                    );
                    Some(res)
                }
                Err(e) => {
                    log::warn!("error parsing file: {:?}", e);
                    None
                }
            })
            .collect(),
    )
}

pub fn find_benchmarks(
    file_name: &str,
    schema_path: &Path,
    search_path: &Path,
    benchmark_defaults: BenchmarkDefaults,
) -> Result<Vec<Benchmark>, Box<dyn error::Error>> {
    let benchmarks =
        find_metadata::<Benchmark>(file_name, schema_path, search_path, benchmark_defaults)?;
    let benchmark_names = benchmarks
        .iter()
        .map(|b| b.name.clone())
        .collect::<HashSet<_>>();
    if benchmark_names.len() != benchmarks.len() {
        Err("found duplicate benchmark names".into())
    } else {
        log::info!(
            "found {} benchmarks: {}",
            benchmarks.len(),
            benchmark_names
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
        Ok(benchmarks)
    }
}

pub fn find_runners(
    file_name: &str,
    schema_path: &Path,
    search_path: &Path,
    runner_defaults: (),
) -> Result<Vec<Runner>, Box<dyn error::Error>> {
    let runners = find_metadata::<Runner>(file_name, schema_path, search_path, runner_defaults)?;
    let runner_names = runners
        .iter()
        .map(|b| b.name.clone())
        .collect::<HashSet<_>>();
    if runner_names.len() != runners.len() {
        Err("found duplicate runners names".into())
    } else {
        log::info!(
            "found {} runners: {}",
            runners.len(),
            runner_names.iter().cloned().collect::<Vec<_>>().join(", ")
        );
        Ok(runners)
    }
}
