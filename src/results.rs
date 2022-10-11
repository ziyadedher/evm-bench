use std::{
    collections::HashSet,
    error,
    fs::{self, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
};

use chrono;

use crate::{metadata::Runner, run::Results};

pub fn record_results(
    results_path: &Path,
    result_file_name: Option<String>,
    results: &Results,
) -> Result<PathBuf, Box<dyn error::Error>> {
    log::debug!("writing all results out...");

    create_dir_all(&results_path)?;

    let mut runners = HashSet::<&Runner>::new();
    for (_, benchmark_results) in results {
        for (runner, _) in benchmark_results {
            runners.insert(runner);
        }
    }

    let mut data = serde_json::Map::new();

    data.insert(
        "benchmarks".to_string(),
        serde_json::Value::Object(serde_json::Map::from_iter(
            results
                .keys()
                .map(|x| (x.name.clone(), serde_json::to_value(x).unwrap())),
        )),
    );
    data.insert(
        "runners".to_string(),
        serde_json::Value::Object(serde_json::Map::from_iter(
            runners
                .into_iter()
                .map(|x| (x.name.clone(), serde_json::to_value(x).unwrap())),
        )),
    );

    let mut runs = serde_json::Map::new();
    for (benchmark, benchmark_results) in results {
        let mut runner_runs = serde_json::Map::new();
        for (runner, run_results) in benchmark_results {
            runner_runs.insert(runner.name.clone(), serde_json::to_value(run_results)?);
        }
        runs.insert(
            benchmark.name.clone(),
            serde_json::Value::Object(runner_runs),
        );
    }
    data.insert("runs".to_string(), serde_json::Value::Object(runs));

    let result = serde_json::Value::Object(data);

    let result_file_path = results_path.join(result_file_name.unwrap_or(format!(
        "{}.evm-bench.results.json",
        chrono::offset::Utc::now().to_rfc3339()
    )));
    let mut result_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .truncate(true)
        .open(&result_file_path)?;
    write!(result_file, "{}", serde_json::to_string_pretty(&result)?)?;

    log::info!(
        "wrote out results to {}",
        result_file_path.to_string_lossy()
    );
    Ok(result_file_path)
}
