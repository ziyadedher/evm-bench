use std::{
    error, fs,
    path::{Path, PathBuf},
    process::exit,
};

use bytes::Bytes;
use glob::glob;

pub trait Metadata {
    fn parse_schema_from_file(
        schema_path: &Path,
    ) -> Result<serde_json::Value, Box<dyn error::Error>>
    where
        Self: Sized,
    {
        let schema_file = fs::File::open(schema_path)?;
        Ok(serde_json::from_reader(&schema_file)?)
    }

    fn parse_from_file(
        schema: &serde_json::Value,
        json_path: &Path,
    ) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized,
    {
        let json_file = fs::File::open(json_path)?;
        let json = serde_json::from_reader(&json_file)?;
        Self::parse(
            json_path.parent().ok_or("could not get parent")?,
            schema,
            &json,
        )
    }

    fn parse(
        base_path: &Path,
        schema: &serde_json::Value,
        json: &serde_json::Value,
    ) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized,
    {
        // if jsonschema::is_valid(schema, json) {
        if true {
            Self::parse_inner(base_path, json)
        } else {
            Err("json does not abide by the schema".into())
        }
    }

    fn parse_inner(
        base_path: &Path,
        json: &serde_json::Value,
    ) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Benchmark {
    pub name: String,
    pub solc_version: Option<String>,
    pub num_runs: Option<u64>,
    pub contract: PathBuf,
    pub calldata: Option<Bytes>,
}

impl Metadata for Benchmark {
    fn parse_inner(
        base_path: &Path,
        json: &serde_json::Value,
    ) -> Result<Self, Box<dyn error::Error>> {
        let object = json.as_object().expect("could not parse json as object");
        Ok(Self {
            name: object
                .get("name")
                .ok_or("could not find name")?
                .as_str()
                .ok_or("could not parse name as string")?
                .to_string(),
            solc_version: object.get("solc-version").map_or(
                Ok::<Option<std::string::String>, Box<dyn error::Error>>(None),
                |x| {
                    Ok(Some(
                        x.as_str()
                            .ok_or("could not parse solc-version as string")?
                            .to_string(),
                    ))
                },
            )?,
            num_runs: object
                .get("num-runs")
                .map_or(Ok::<Option<u64>, Box<dyn error::Error>>(None), |x| {
                    Ok(Some(x.as_u64().ok_or("could not parse num-runs as u64")?))
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
            calldata: object.get("calldata").map_or(
                Ok::<Option<bytes::Bytes>, Box<dyn error::Error>>(None),
                |x| {
                    Ok(Some(
                        hex::decode(x.as_str().ok_or("could not parse calldata as bytes")?)?.into(),
                    ))
                },
            )?,
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Runner {
    pub name: String,
    pub entry: PathBuf,
}

impl Metadata for Runner {
    fn parse_inner(
        base_path: &Path,
        json: &serde_json::Value,
    ) -> Result<Self, Box<dyn error::Error>> {
        let object = json.as_object().expect("could not parse json as object");
        Ok(Self {
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
        })
    }
}

pub fn find_metadata<T: Metadata>(
    file_name: &str,
    schema_path: &Path,
    search_path: &Path,
) -> Result<Vec<T>, Box<dyn error::Error>> {
    let schema = Benchmark::parse_schema_from_file(schema_path)?;

    let path = search_path
        .canonicalize()
        .expect("could not canonicalize search path");
    if !path.is_dir() {
        log::error!("{} is not a directory", path.display());
        exit(-1);
    }

    Ok(glob(
        path.join("**")
            .join(file_name)
            .to_str()
            .expect("could not construct glob"),
    )
    .expect("could not read glob pattern")
    .flat_map(|entry| match entry {
        Ok(path) => {
            log::debug!("found {}", path.display());
            Some(path)
        }
        Err(e) => {
            log::warn!("error globing file: {:?}", e);
            None
        }
    })
    .flat_map(|path| match T::parse_from_file(&schema, &path) {
        Ok(res) => {
            log::debug!("parsed {}", path.display());
            Some(res)
        }
        Err(e) => {
            log::warn!("error parsing file: {:?}", e);
            None
        }
    })
    .collect())
}
