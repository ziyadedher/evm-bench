#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

typify::import_types!(
    schema = "runners/runner.schema.json",
    patch = { EmvBenchRunnerMetadata = { rename = "RunnerMetadata" } }
);
