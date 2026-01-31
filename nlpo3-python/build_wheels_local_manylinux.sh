#!/bin/bash

# The main build wheel workflow is on GitHub Actions,
# see .github/wheels.yml at the root of main repo.
# This script is meant to be run local and use for testing purpose only.

# This script has to run through manylinux docker image:
# docker run --rm -v `pwd`:/io quay.io/pypa/manylinux2014_x86_64 bash /io/build_wheels_local_manylinux.sh

set -ex

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
export PATH="$HOME/.cargo/bin:$PATH"

cd /io

for PYBIN in /opt/python/cp{36,37,38,39,310,311,312,313,314,315,316,317,318}*/bin; do
    "${PYBIN}/pip" install -U build setuptools setuptools-rust wheel
    "${PYBIN}/python" -m build --wheel
done

for whl in dist/*linux*.whl; do
    auditwheel repair "$whl" -w dist/
done
