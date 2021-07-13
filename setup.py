from setuptools import find_packages, setup
from setuptools_rust import Binding, RustExtension

setup(
    rust_extensions=[
        RustExtension(
            "pythainlp-rust-modules", "Cargo.toml", binding=Binding.PyO3
        )
    ],
    include_package_data=True,
    packages=find_packages(exclude=["tests", "tests.*", "notebooks",]),
    test_suite="tests",
    zip_safe=False,
)
