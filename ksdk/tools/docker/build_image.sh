#!/bin/bash

# SPDX-License-Identifier: MPL-2.0

set -e

SCRIPT_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ASTROS_ROOT_DIR=${SCRIPT_DIR}/../../..
ASTROS_RUST_VERSION=$( grep -m1 -o 'nightly-[0-9]\+-[0-9]\+-[0-9]\+' ${ASTROS_ROOT_DIR}/rust-toolchain.toml )
VERSION=$( cat ${ASTROS_ROOT_DIR}/VERSION )
DOCKERFILE=${SCRIPT_DIR}/Dockerfile

if [ "$1" = "intel-tdx" ]; then
    IMAGE_NAME="astros/ksdk:${VERSION}-tdx"
    python3 gen_dockerfile.py --intel-tdx
else
    IMAGE_NAME="astros/ksdk:${VERSION}"
    python3 gen_dockerfile.py
fi

docker build \
    -t ${IMAGE_NAME} \
    --build-arg ASTROS_RUST_VERSION=${ASTROS_RUST_VERSION} \
    -f ${DOCKERFILE} \
    ${SCRIPT_DIR} 
