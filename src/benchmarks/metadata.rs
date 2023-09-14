#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

typify::import_types!(
    schema = "benchmarks/benchmark.schema.json",
    patch = {
        EmvBenchBenchmarkMetadata = { rename = "BenchmarkMetadata" },
        EmvBenchBenchmarkMetadataCost = { rename = "BenchmarkMetadataCost" },
    }
);
