#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

set -e

echo "*** Running the LMbench memory-copy bandwidth test ***"

/benchmark/bin/lmbench/bw_mem -P 1 128m fcp