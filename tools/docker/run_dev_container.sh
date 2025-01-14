#!/bin/bash

# SPDX-License-Identifier: MPL-2.0

set -e

SCRIPT_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ASTROS_SRC_DIR=${SCRIPT_DIR}/../..
CARGO_TOML_PATH=${SCRIPT_DIR}/../../Cargo.toml
VERSION=$( cat ${ASTROS_SRC_DIR}/VERSION )

if [ "$1" = "intel-tdx" ]; then
    IMAGE_NAME="astros/astros:${VERSION}-tdx"
else
    IMAGE_NAME="astros/astros:${VERSION}"
fi

docker run -it --privileged --network=host --device=/dev/kvm --device=/dev/vhost-net -v ${ASTROS_SRC_DIR}:/root/astros ${IMAGE_NAME}
