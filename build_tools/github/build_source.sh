#!/bin/bash

set -e
set -x

cd ../../python_binding

python -m venv build_env
source build_env/bin/activate

python -m pip install --upgrade pip
#python -m pip install --upgrade build wheel setuptools-rust
python -m pip install --upgrade setuptools-rust
python -m pip install --upgrade twine

cd oxidized-thainlp/oxidized-thainlp
python setup.py sdist

twine check dist/*.tar.gz
