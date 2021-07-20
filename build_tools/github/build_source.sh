#!/bin/bash

set -e
set -x

cd ../../

python -m venv build_env
source build_env/bin/activate

python -m pip install --upgrade pip
python -m pip install --upgrade setuptools-rust twine

cd nlpo3/nlpo3-python
python setup.py sdist

twine check dist/*.tar.gz
