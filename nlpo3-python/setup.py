from setuptools import setup
from setuptools_rust import Binding, RustExtension, Strip

setup(
    rust_extensions=[
        RustExtension(
            "nlpo3._nlpo3_python_backend",
            path="Cargo.toml",
            binding=Binding.PyO3,
            strip=Strip.No,
        )
    ],
)
