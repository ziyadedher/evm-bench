#!/usr/bin/env bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cargo run --profile=production --manifest-path $SCRIPT_DIR/Cargo.toml -- $@
