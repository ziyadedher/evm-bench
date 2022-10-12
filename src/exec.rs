use std::{
    error,
    path::{Path, PathBuf},
    process::Command,
};

pub fn validate_executable(
    name: &str,
    executable: &Path,
) -> Result<PathBuf, Box<dyn error::Error>> {
    log::trace!("validating executable {} ({name})", executable.display());
    match Command::new(&executable).arg("--version").output() {
        Ok(out) => {
            log::debug!(
                "found {name} ({}): {}",
                executable.display(),
                String::from_utf8(out.stdout)
                    .expect("could not decode program stdout")
                    .trim_end_matches("\n")
            );
            Ok(executable.to_path_buf())
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                Err(format!("{name} not found, tried {}", executable.display()).into())
            }
            _ => Err(format!("unknown error: {e}").into()),
        },
    }
}
