use std::{
    collections::HashSet,
    error,
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::Command,
};

use users::{get_current_gid, get_current_uid};

use crate::metadata::Benchmark;

#[derive(Clone, Debug)]
struct BuildContext {
    docker_executable: PathBuf,
    contract_path: PathBuf,
    contract_context_path: PathBuf,
    build_path: PathBuf,
}

#[derive(Debug)]
pub struct BuildResult {
    pub contract_bin_path: PathBuf,
}

#[derive(Debug)]
pub struct BuiltBenchmark {
    pub benchmark: Benchmark,
    pub result: BuildResult,
}

fn build_benchmark(
    benchmark: &Benchmark,
    build_context: &BuildContext,
) -> Result<BuiltBenchmark, Box<dyn error::Error>> {
    let contract_name = benchmark
        .contract
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    log::info!(
        "building benchmark {} ({contract_name} w/ solc@{})...",
        benchmark.name,
        benchmark.solc_version
    );

    let relative_contract_path = build_context
        .contract_path
        .strip_prefix(&build_context.contract_context_path)?;

    let docker_contract_context_path = PathBuf::from("/benchmark");
    let docker_contract_path = docker_contract_context_path.join(relative_contract_path);
    let docker_build_path = PathBuf::from("/build");

    create_dir_all(&build_context.build_path)?;

    let out = Command::new(&build_context.docker_executable)
        .arg("run")
        .args([
            "-u",
            &format!("{}:{}", get_current_uid(), get_current_gid()),
        ])
        .args([
            "-v",
            &format!(
                "{}:{}",
                build_context.contract_context_path.to_string_lossy(),
                docker_contract_context_path.to_string_lossy()
            ),
        ])
        .args([
            "-v",
            &format!(
                "{}:{}",
                build_context.build_path.to_string_lossy(),
                docker_build_path.to_string_lossy()
            ),
        ])
        .arg(format!("ethereum/solc:{}", benchmark.solc_version))
        .args(["-o", &docker_build_path.to_string_lossy()])
        .args(["--abi", "--bin", "--optimize", "--overwrite"])
        .arg(docker_contract_path)
        .output()?;

    log::trace!("stdout: {}", String::from_utf8(out.stdout).unwrap());
    log::trace!("stderr: {}", String::from_utf8(out.stderr).unwrap());

    if out.status.success() {
        let mut contract_bin_path = build_context.build_path.join(&contract_name);
        contract_bin_path.set_extension("bin");

        log::debug!("built benchmark {}", benchmark.name);
        Ok(BuiltBenchmark {
            benchmark: benchmark.clone(),
            result: BuildResult { contract_bin_path },
        })
    } else {
        Err(format!("{}", out.status).into())
    }
}

pub fn build_benchmarks(
    benchmarks: &Vec<Benchmark>,
    docker_executable: &Path,
    builds_path: &Path,
) -> Result<Vec<BuiltBenchmark>, Box<dyn error::Error>> {
    let benchmark_names = benchmarks
        .iter()
        .map(|b| b.name.clone())
        .collect::<HashSet<_>>();

    log::info!("building {} benchmarks...", benchmarks.len());
    log::debug!(
        "benchmarks: {}",
        benchmark_names
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut results = Vec::<BuiltBenchmark>::new();
    for benchmark in benchmarks {
        results.push(
            match build_benchmark(
                benchmark,
                &BuildContext {
                    docker_executable: docker_executable.to_path_buf(),
                    contract_path: benchmark.contract.clone(),
                    contract_context_path: benchmark.build_context.clone(),
                    build_path: builds_path.join(&benchmark.name),
                },
            ) {
                Ok(res) => res,
                Err(e) => {
                    log::warn!("could not build benchmark {}: {e}", benchmark.name);
                    continue;
                }
            },
        );
    }

    log::debug!(
        "built {} benchmarks ({} successful)",
        benchmarks.len(),
        results.len()
    );
    Ok(results)
}
