use std::{
    collections::{HashMap, HashSet},
    error,
    fs::{self, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono;
use serde::{Deserialize, Serialize};
use tabled::{builder::Builder, Style};

use crate::{
    metadata::{Benchmark, Runner},
    run::{Results, RunResult},
};

#[derive(Deserialize, Serialize)]
struct ResultsFormatted {
    benchmarks: HashMap<String, Benchmark>,
    runners: HashMap<String, Runner>,
    runs: HashMap<String, HashMap<String, RunResult>>,
}

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

    let results_formatted = ResultsFormatted {
        benchmarks: results
            .keys()
            .map(|b| (b.name.clone(), b.clone()))
            .collect(),
        runners: runners
            .into_iter()
            .map(|r| (r.name.clone(), r.clone()))
            .collect(),
        runs: results
            .iter()
            .map(|(b, br)| {
                (
                    b.name.clone(),
                    br.iter()
                        .map(|(r, rr)| (r.name.clone(), rr.clone()))
                        .collect(),
                )
            })
            .collect(),
    };

    let result_file_path = results_path.join(result_file_name.unwrap_or(format!(
        "{}.evm-bench.results.json",
        chrono::offset::Utc::now().to_rfc3339()
    )));
    let mut result_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .truncate(true)
        .open(&result_file_path)?;
    write!(
        result_file,
        "{}",
        serde_json::to_string_pretty(&results_formatted)?
    )?;

    log::info!(
        "wrote out results to {}",
        result_file_path.to_string_lossy()
    );
    Ok(result_file_path)
}

pub fn print_results(results_file_path: &Path) -> Result<(), Box<dyn error::Error>> {
    log::info!(
        "reading and parsing results from {}...",
        results_file_path.to_string_lossy()
    );
    let results =
        serde_json::from_str::<ResultsFormatted>(&fs::read_to_string(results_file_path)?)?;
    log::debug!(
        "read and parsed results from {}",
        results_file_path.to_string_lossy()
    );

    let mut runner_names: Vec<_> = results.runners.keys().cloned().collect();
    runner_names.sort();

    let mut runs = results.runs.into_iter().collect::<Vec<_>>();
    runs.sort_by_key(|(b, _)| b.clone());

    let mut runner_times = HashMap::<String, Vec<Duration>>::new();

    let mut builder = Builder::default();
    for (benchmark_name, benchmark_runs) in runs.iter() {
        let vals = runner_names.iter().map(|runner_name| {
            let run = benchmark_runs.get(runner_name)?;
            let avg_run_time = run
                .run_times
                .iter()
                .fold(Duration::ZERO, |a, v| a + v.clone())
                .div_f64(run.run_times.len() as f64);
            runner_times
                .entry(runner_name.clone())
                .or_default()
                .push(avg_run_time);
            Some(avg_run_time)
        });

        let mut record = vec![benchmark_name.clone()];
        record.extend(
            vals.map(|val| Some(format!("{:?}", val?)))
                .map(|s| s.unwrap_or_default()),
        );
        builder.add_record(record);
    }

    let average_runner_times = runner_times
        .into_iter()
        .map(|(name, times)| (name, times.iter().sum::<Duration>()))
        .collect::<HashMap<String, Duration>>();
    let mut record = vec!["sum".to_string()];
    record.extend(
        runner_names
            .iter()
            .map(|runner_name| average_runner_times.get(runner_name))
            .map(|val| Some(format!("{:?}", val?)))
            .map(|s| s.unwrap_or_default()),
    );
    builder.add_record(record);

    let mut columns = vec!["".to_owned()];
    columns.extend(runner_names);
    builder.set_columns(columns);

    let mut table = builder.build();
    table.with(Style::markdown());
    println!("{}", table);

    Ok(())
}
