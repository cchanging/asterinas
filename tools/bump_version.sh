#!/bin/bash

# SPDX-License-Identifier: MPL-2.0

set -e

# This script is used to update Astros version numbers in all relevant files in the repository.
# Usage: ./tools/bump_version.sh bump_type
# bump_type can be one of: patch, minor, or major.

# TODO: we may remove the VERSION file in the future, 
# and retrieve the current version from git tag.

# Update the package version (`version = "{version}"`) in file $1
update_package_version() {
    echo "Updating file $1"
    # Package version is usually the first version in Cargo.toml,
    # so only the first matched version is updated.
    pattern="^version = \"[[:digit:]]\+\.[[:digit:]]\+\.[[:digit:]]\+\"$"
    sed -i "0,/${pattern}/s/${pattern}/version = \"${new_version}\"/1" $1
}

# Update the version of the $2 dependency (`$2 = { version = "", ...`) in file $1
update_dep_version() {
    echo "Updating the version of $2 in file $1"
    pattern="^$2 = { version = \"[[:digit:]]\+\.[[:digit:]]\+\.[[:digit:]]\+\""
    sed -i "0,/${pattern}/s/${pattern}/$2 = { version = \"${new_version}\"/1" $1
}

# Update Docker image versions (`astros/astros:{version}`) in file $1
update_image_versions() {
    echo "Updating file $1"
    # Update the version of the development container
    sed -i "s/astros\/astros:[[:digit:]]\+\.[[:digit:]]\+\.[[:digit:]]\+/astros\/astros:${new_version}/g" $1
    # Update the test environment described in the KSDK manual
    sed -i "s/astros\/ksdk:[[:digit:]]\+\.[[:digit:]]\+\.[[:digit:]]\+/astros\/ksdk:${new_version}/g" $1
}

# Print the help message
print_help() {
    echo "Usage: $0 bump_type"
    echo ""
    echo "The bump_type argument must be either \"patch\", \"minor\", or \"major\","
    echo "which instructs the script to increment the patch, minor, and major part"
    echo "of the semantic version number of Astros, respectively."
}

# Add the number $1 by 1
# Bash cannot deal with 0 by using `$((num + 1))`,
# So this function is defined to specially deal with 0.
add_one() {
    local num=$1
    if [ "$num" == "0" ]; then
        echo "1"
    else
        local bumped=$((num + 1))
        echo "$bumped"
    fi
}

# Bump the version based on $bump_type
bump_version() {
    local IFS="."
    local version_parts=($current_version)

    case "$bump_type" in
        "patch")
            version_parts[2]=$(add_one "${version_parts[2]}")
            ;;
        "minor")
            version_parts[1]=$(add_one "${version_parts[1]}")
            version_parts[2]=0
            ;;
        "major")
            version_parts[0]=$(add_one "${version_parts[0]}")
            version_parts[1]=0
            version_parts[2]=0
            ;;
    esac

    echo "${version_parts[*]}"
}

# Validate the bump type
validate_bump_type() {
    case "$bump_type" in
        "patch" | "minor" | "major")
            ;;
        *)
        echo "Error: Invalid bump_type. Allowed values are: patch, minor, or major."
        print_help
        exit 1
        ;;
    esac
}

# Update tag version (`v{version}`) in file $1
update_tag_version() {
    echo "Updating file $1"
    sed -i "s/v[[:digit:]]\+\.[[:digit:]]\+\.[[:digit:]]\+/v${new_version}/g" $1
}

SCRIPT_DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ASTROS_SRC_DIR=${SCRIPT_DIR}/..
DOCS_DIR=${ASTROS_SRC_DIR}/docs
KSTD_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/kstd/Cargo.toml
KSTD_TEST_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/kstd/libs/kstd-test/Cargo.toml
KSTD_MACROS_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/kstd/libs/kstd-macros/Cargo.toml
LINUX_BOOT_PARAMS_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/kstd/libs/linux-bzimage/boot-params/Cargo.toml
LINUX_BZIMAGE_BUILDER_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/kstd/libs/linux-bzimage/builder/Cargo.toml
LINUX_BZIMAGE_SETUP_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/kstd/libs/linux-bzimage/setup/Cargo.toml
KSDK_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/ksdk/Cargo.toml
KSDK_TEST_RUNNER_CARGO_TOML_PATH=${ASTROS_SRC_DIR}/ksdk/test-kernel/Cargo.toml
VERSION_PATH=${ASTROS_SRC_DIR}/VERSION

current_version=$(cat ${VERSION_PATH})
bump_type=$1

if [[ "$bump_type" == "--help" || "$bump_type" == "-h" ]]; then
  print_help
  exit 0
fi

validate_bump_type
new_version=$(bump_version ${current_version})

# Update the versions in Cargo.toml
update_package_version ${KSTD_TEST_CARGO_TOML_PATH}
update_package_version ${KSTD_MACROS_CARGO_TOML_PATH}
update_package_version ${KSTD_CARGO_TOML_PATH}
update_package_version ${LINUX_BOOT_PARAMS_CARGO_TOML_PATH}
update_package_version ${LINUX_BZIMAGE_BUILDER_CARGO_TOML_PATH}
update_package_version ${LINUX_BZIMAGE_SETUP_CARGO_TOML_PATH}
update_dep_version ${KSTD_CARGO_TOML_PATH} kstd-test
update_dep_version ${KSTD_CARGO_TOML_PATH} linux-boot-params
update_dep_version ${KSTD_CARGO_TOML_PATH} kstd-macros
update_dep_version ${LINUX_BZIMAGE_SETUP_CARGO_TOML_PATH} linux-boot-params
update_package_version ${KSDK_CARGO_TOML_PATH}
update_package_version ${KSDK_TEST_RUNNER_CARGO_TOML_PATH}
update_dep_version ${KSDK_TEST_RUNNER_CARGO_TOML_PATH} kstd
update_dep_version ${KSDK_CARGO_TOML_PATH} linux-bzimage-builder

# Automatically bump Cargo.lock files
cargo update -p astros-nix --precise $new_version # For Cargo.lock
cd ksdk && cargo update -p cargo-ksdk --precise $new_version # For ksdk/Cargo.lock

# Update Docker image versions in README files
update_image_versions ${ASTROS_SRC_DIR}/README.md
update_image_versions ${ASTROS_SRC_DIR}/README_CN.md
update_image_versions ${SCRIPT_DIR}/docker/README.md
update_image_versions ${DOCS_DIR}/src/kernel/intel_tdx.md

# Update Docker image versions in workflows
WORKFLOWS=$(find "${ASTROS_SRC_DIR}/.github/workflows/" -type f -name "*.yml")
for workflow in $WORKFLOWS; do
    update_image_versions $workflow
done

# Update tag version in release_tag workflow
RELEASE_TAG_WORKFLOW=${ASTROS_SRC_DIR}/.github/workflows/push_git_tag.yml
update_tag_version $RELEASE_TAG_WORKFLOW

# Update Docker image versions in the documentation
GET_STARTED_PATH=${ASTROS_SRC_DIR}/docs/src/kernel/README.md
update_image_versions $GET_STARTED_PATH

# Create or update VERSION
# `-n` is used to avoid adding a '\n' in the VERSION file.
echo -n "${new_version}" > ${VERSION_PATH}

echo "Bumped Astros KSTD & KSDK version to $new_version"
