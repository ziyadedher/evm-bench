//! Tools for writing, reading, and visualizing results.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Run;

#[derive(Deserialize, Serialize)]
struct Runs {
    runs: Vec<Run>,
}

/// Write a new output file for the given runs in the given directory.
///
/// The output file will be named `outputs.<timestamp>.json` where `<timestamp>` is the provided time in the format
/// `%Y-%m-%dT%H-%M-%S%z`. Returns a path to that output file.
///
/// # Errors
///
/// If serialization or writing to the output file fails, an error will be returned.
pub fn write_outputs(
    runs: &[Run],
    outputs_path: &Path,
    time: &DateTime<Utc>,
) -> anyhow::Result<PathBuf> {
    let outputs = serde_json::to_string_pretty(&Runs {
        runs: runs.to_vec(),
    })?;

    let output_file_path = outputs_path.join(format!(
        "outputs.{}.json",
        time.format("%Y-%m-%dT%H-%M-%S%z")
    ));
    log::info!(
        "writing result output to {}...",
        output_file_path.to_string_lossy()
    );
    fs::create_dir_all(outputs_path).context("could not create output directory structure")?;
    fs::write(&output_file_path, outputs).context(format!(
        "could not write to output file {}",
        output_file_path.to_string_lossy()
    ))?;

    Ok(output_file_path)
}

/// Read the most recent output file from the given directory.
///
/// Looks into the given directory and finds the most recent output file by name. The output file must be named
/// `outputs.<timestamp>.json` where `<timestamp>` is the time the file was created in the format
/// `%Y-%m-%dT%H-%M-%S%z`. Returns the path to the chosen output file and the parsed runs from that file.
///
/// # Errors
///
/// If reading the output file or parsing the runs fails, an error will be returned.
pub fn read_latest_outputs(outputs_path: &Path) -> anyhow::Result<(PathBuf, Vec<Run>)> {
    let output_file_path = outputs_path.join(
        fs::read_dir(outputs_path)
            .context("could not read output directory")?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with("outputs."))
                    && entry
                        .path()
                        .extension()
                        .map_or(false, |ext| ext.eq_ignore_ascii_case("json"))
            })
            .max_by_key(fs::DirEntry::path)
            .context("could not find any output files")?
            .path(),
    );

    log::info!(
        "reading result output from {}...",
        output_file_path.to_string_lossy()
    );
    let outputs = fs::read_to_string(&output_file_path).context(format!(
        "could not read output file {}",
        output_file_path.to_string_lossy()
    ))?;
    let runs: Runs = serde_json::from_str(&outputs).context(format!(
        "could not parse output file {}",
        output_file_path.to_string_lossy()
    ))?;

    Ok((output_file_path, runs.runs))
}

/// Create a Markdown table from the given runs.
///
/// Analyzes the given runs and creates a Markdown table from them. The table will have one column for each runner and
/// one row for each benchmark. The cells will contain the average run time for that benchmark and runner. The table
/// also has two additional rows for "relative performance" (the average run time of each runner relative to the
/// fastest, normalized to 100%) and "total time" (the total time taken by each runner to run all benchmarks). The
/// columns are ordered by the total time taken by each runner in ascending order. The table is returned as a string
/// representing the Markdown table.
///
/// # Errors
///
/// If the table cannot be created, an error will be returned.
#[allow(clippy::too_many_lines)]
pub fn create_markdown_table(runs: &[Run]) -> anyhow::Result<String> {
    let mut runners = runs
        .iter()
        .map(|run| run.runner_identifier.clone())
        .collect::<Vec<_>>();
    runners.sort();
    runners.dedup();

    let mut benchmarks = runs
        .iter()
        .map(|run| run.benchmark_identifier.clone())
        .collect::<Vec<_>>();
    benchmarks.sort();
    benchmarks.dedup();

    let total_times = runners
        .iter()
        .map(|runner| {
            (
                runner.clone(),
                runs.iter()
                    .filter(|run| run.runner_identifier == *runner)
                    .map(|r| r.average_duration)
                    .sum::<Duration>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let runners = {
        let mut runners = runners
            .iter()
            .filter_map(|runner| {
                Some((
                    runner.clone(),
                    total_times
                        .get(runner)
                        .context("could not find total time")
                        .ok()?,
                ))
            })
            .collect::<Vec<_>>();
        runners.sort_by_key(|(_, total_time)| *total_time);
        runners
            .iter()
            .map(|(runner, _)| runner.clone())
            .collect::<Vec<_>>()
    };

    let fastest_total_time = total_times
        .values()
        .min()
        .context("could not find fastest total time")?;

    let mut table = String::new();

    table.push_str("| Benchmark |");
    for runner in &runners {
        table.push_str(&format!(" {runner} |"));
    }
    table.push('\n');

    table.push_str("| --- |");
    for _ in &runners {
        table.push_str(" --- |");
    }
    table.push('\n');

    table.push_str("| Relative Performance |");
    for runner in &runners {
        let total_time = total_times
            .get(runner)
            .context("could not find total time")?;
        table.push_str(&format!(
            " {:.2}x |",
            total_time.as_secs_f64() / fastest_total_time.as_secs_f64()
        ));
    }
    table.push('\n');

    table.push_str("| Total Time |");
    for runner in &runners {
        let total_time = total_times
            .get(runner)
            .context("could not find total time")?;
        table.push_str(
            &(if total_time.as_secs_f64() < 1.0 {
                format!(" {:4}ms |", total_time.as_millis())
            } else {
                format!(" {:.2}s |", total_time.as_secs_f64())
            }),
        );
    }
    table.push('\n');

    for benchmark in &benchmarks {
        table.push_str(&format!("| {benchmark} |"));
        for runner in &runners {
            let run = runs
                .iter()
                .find(|run| {
                    run.benchmark_identifier == *benchmark && run.runner_identifier == *runner
                })
                .context("could not find run")?;
            table.push_str(
                &(if run.average_duration.as_secs_f64() < 1.0 {
                    format!(" {:4}ms |", run.average_duration.as_millis())
                } else {
                    format!(" {:.2}s |", run.average_duration.as_secs_f64())
                }),
            );
        }
        table.push('\n');
    }

    Ok(table)
}
