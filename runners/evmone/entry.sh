#!/usr/bin/env bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR
{
  cmake -S . -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build --parallel
} > /dev/null
build/runner $@
