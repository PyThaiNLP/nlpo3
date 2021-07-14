from setuptools import find_packages, setup
from setuptools_rust import Binding, RustExtension

long_description = """
Thai Natural Language Processing in Rust, with Python-binding.

## Features

- Word tokenizer
  - maximal-matching dictionary-based tokenization
  - 2x faster than similar pure Python implementation (PyThaiNLP's newmm)
  - support custom dictionary

## Usage

Install:
```bash
pip install pythainlp-rust-modules
```

Use in Python:
```python
from oxidized_thainlp import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

## Issues

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp
"""

setup(
    name="pythainlp-rust-modules",
    version="1.0.0",
    description=(
        "Thai Natural Language Processing in Rust, "
        "with Python-binding"
    ),
    long_description=long_description,
    long_description_content_type="text/markdown",
    python_requires=">=3.6",
    license="Apache-2.0",
    keywords=[
        "thai",
        "tokenizer",
        "nlp",
        "rust",
        "pythainlp",
    ],
    author=(
        "Thanathip Suntorntip, "
        "Arthit Suriyawongkul, "
        "Wannaphong Phatthiyaphaibun"
    ),
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Programming Language :: Python :: 3",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Natural Language :: Thai",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Text Processing :: Linguistic",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    project_urls={
        "homepage": "https://github.com/PyThaiNLP/oxidized-thainlp",
        "repository": "https://github.com/PyThaiNLP/oxidized-thainlp",
    },
    include_package_data=True,
    packages=find_packages(
        exclude=[
            "tests",
            "tests.*",
            "notebooks",
        ]
    ),
    test_suite="tests",
    zip_safe=False,
    rust_extensions=[
        RustExtension(
            "pythainlp-rust-modules", "Cargo.toml", binding=Binding.PyO3
        )
    ],
)
