#!/usr/bin/env bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR
poetry env use python3.9 >&2
poetry install >&2
poetry update >&2
poetry run python ../runner.py $@
