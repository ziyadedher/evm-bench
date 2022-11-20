#!/usr/bin/env bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR
{
  cmake -S . -B build && cmake --build build --parallel
} > /dev/null
build/runner $@
