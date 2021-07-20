#!/bin/bash

set -e
set -x

cd ../../

python -m venv build_env
source build_env/bin/activate

python -m pip install --upgrade pip
python -m pip install --upgrade setuptools-rust twine

cd oxidized-thainlp/nlpo3-python
python setup.py sdist

twine check dist/*.tar.gz
