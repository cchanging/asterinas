#!/bin/bash

# SPDX-License-Identifier: MPL-2.0

set -e

SCRIPT_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ASTROS_ROOT_DIR=${SCRIPT_DIR}/../..
VERSION=$( cat ${ASTROS_ROOT_DIR}/VERSION )

if [ "$1" = "intel-tdx" ]; then
    IMAGE_NAME="astros/ksdk:${VERSION}-tdx"
else
    IMAGE_NAME="astros/ksdk:${VERSION}"
fi

docker run -it -v ${ASTROS_ROOT_DIR}:/root/astros ${IMAGE_NAME}
