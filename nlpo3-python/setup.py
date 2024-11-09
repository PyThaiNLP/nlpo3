# SPDX-FileCopyrightText: 2024 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

from setuptools import find_packages, setup
from setuptools_rust import Binding, RustExtension, Strip

setup(
    packages=find_packages(exclude=["notebooks", "tests"]),
    rust_extensions=[
        RustExtension(
            "nlpo3._nlpo3_python_backend",
            path="Cargo.toml",
            binding=Binding.PyO3,
            strip=Strip.No,
        )
    ],
)
