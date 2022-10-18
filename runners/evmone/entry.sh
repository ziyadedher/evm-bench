#!/usr/bin/env bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR
make build-opt --silent
LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:lib/ ./runner $@

