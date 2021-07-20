#!/bin/bash

# The main build wheel workflow is on GitHub Actions,
# see .github/wheels.yml at the root of main repo.
# This script is meant to be run local and use for testing purpose only.

# Use pyenv to build on different Python versions.

# If "error: implicit declaration of function 'sendfile' is invalid in C99"
# occurs when installing a Python version, see fix at:
# https://github.com/pyenv/pyenv/issues/1740#issuecomment-738749988

set -ex

# store pyenv Python version before calling the script
SAVE_PYVER=$(pyenv global)

for PYVER in $(ls ~/.pyenv/versions); do
    PYVER_MINOR=$(echo "${PYVER}" | sed -nre 's/^(pypy)?(([0-9]+\.)?[0-9]+).*/\1\2/p')
    echo "Build for Python ${PYVER_MINOR}"
    pyenv global "${PYVER}"
    $(pyenv which pip) install -U pip
    $(pyenv which pip) install -U build setuptools setuptools-rust wheel
    $(pyenv which python) -m build --wheel
done

# restore pyenv Python version
pyenv global ${SAVE_PYVER}
