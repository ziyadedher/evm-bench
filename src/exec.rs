use std::{
    path::{Path, PathBuf},
    process::{exit, Command},
};

pub fn validate_executable_or_exit(name: &str, executable: &Path) -> PathBuf {
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
            executable.to_path_buf()
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                log::error!("{name} not found, tried {}", executable.display());
                exit(-1);
            }
            _ => {
                log::error!("unknown error: {e}");
                exit(-1);
            }
        },
    }
}
