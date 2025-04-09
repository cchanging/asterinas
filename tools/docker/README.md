# Astros Development Docker Images

Astros development Docker images are provided to facilitate developing and testing Astros project. These images can be found in the [astros/astros](https://hub.docker.com/r/astros/astros/) repository on DockerHub.

## Building Docker Images

To build a Docker image for Astros and test it on your local machine, navigate to the root directory of the Astros source code tree and execute the following command:

```bash
cd <astros dir>/tools/docker
# Generate Dockerfile
python3 gen_dockerfile.py
cd <astros dir>
# Build Docker image
docker buildx build \
    -f tools/docker/Dockerfile \
    --build-arg ASTROS_RUST_VERSION=${RUST_VERSION} \
    -t astros/astros:${ASTROS_VERSION} \
    .
```

The meanings of the two environment variables in the command are as follows:

- `${ASTROS_VERSION}`: Represents the version number of Astros. You can find this in the `VERSION` file.
- `${RUST_VERSION}`: Denotes the required Rust toolchain version, as specified in the `rust-toolchain` file.

For Intel TDX Docker Image, you can execute the following command:

```bash
cd <astros dir>/tools/docker
# Generate Dockerfile for Intel TDX
python3 gen_dockerfile.py --intel-tdx
cd <astros dir>
# Build Docker image
docker buildx build \
    -f tools/docker/Dockerfile \
    --build-arg ASTROS_RUST_VERSION=${RUST_VERSION} \
    -t astros/astros:${ASTROS_VERSION}-tdx \
    .
```

## Tagging Docker Images

It's essential for each Astros Docker image to have a distinct tag. By convention, the tag is assigned with the version number of the Astros project itself. This methodology ensures clear correspondence between a commit of the source code and its respective Docker image.

If a commit needs to create a new Docker image, it should

1. Update the Dockerfile as well as other materials relevant to the Docker image, and
2. Run [`tools/bump_version.sh`](../bump_version.sh) tool to update the Astros project's version number.
 
For bug fixes or small changes, increment the last number of a [SemVer](https://semver.org/) by one. For major features or releases, increment the second number. All changes made in the two steps should be included in the commit.

## Uploading Docker Images

New versions of Astros's Docker images are automatically uploaded to DockerHub through Github Actions. Simply submit your PR that updates Astros's Docker image for review. After getting the project maintainers' approval, the [Docker image building workflow](../../.github/workflows/docker_build.yml) will be started, building the new Docker image and pushing it to DockerHub.
