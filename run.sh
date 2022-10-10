#!/usr/bin/env bash
set -e

runners/revm/entry.sh --contract-code-path ./outputs/build/snailtracer/SnailTracer.bin --calldata 30627b7c --num-runs 10
