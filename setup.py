from setuptools import find_packages, setup
from setuptools_rust import Binding, RustExtension

setup(
    name="pythainlp-rust-modules",
    version="1.0.0",
    rust_extensions=[
        RustExtension(
            "pythainlp-rust-modules", "Cargo.toml", binding=Binding.PyO3
        )
    ],
    include_package_data=True,
    packages=find_packages(exclude=["tests", "tests.*", "notebooks"]),
    test_suite="tests",
    python_requires=">=3.6",
    zip_safe=False,
    license="Apache-2.0",
    keywords = [
        "thai",
        "tokenizer",
        "nlp",
        "rust",
        "pythainlp",
    ],
    classifiers = [
        # "3 - Alpha", "4 - Beta" or "5 - Production/Stable"
        "Development Status :: 5 - Production/Stable",
        "Programming Language :: Python :: 3",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Natural Language :: Thai",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Text Processing :: Linguistic",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ]

)
