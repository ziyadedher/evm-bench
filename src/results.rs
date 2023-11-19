//! Tools for writing, reading, and visualizing results.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::Run;

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
    let outputs = serde_json::to_string_pretty(&json!({
        "runs": runs,
    }))?;

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
