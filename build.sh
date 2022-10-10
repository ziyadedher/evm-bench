#!/usr/bin/env bash
set -e

CWD=`pwd`

docker run                                          \
    -u `id -u $USER`:`id -g $USER`                  \
    -v $CWD:/work                                   \
    -w /work                                        \
    ethereum/solc:0.4.26                            \
    -o ./outputs/build/snailtracer                  \
    --abi --bin --optimize --overwrite              \
    ./benchmarks/snailtracer/SnailTracer.sol
