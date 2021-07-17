from setuptools import find_packages, setup
from setuptools_rust import Binding, RustExtension

long_description = """
Thai Natural Language Processing in Rust,

## Features

- Word tokenizer
  - maximal-matching dictionary-based tokenization
  - 2x faster than similar pure Python implementation (PyThaiNLP's newmm)
  - support custom dictionary

## Usage

Install:
```bash
pip install nlpo3
```

Use in Python:
```python
from nlpo3 import load_dict, segment

load_dict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

## Issues

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp
"""

setup(
    name="nlpo3",
    version="1.1.0",
    description=(
        "Python binding for NLPO3 Thai language processing library"
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
    author_email="wannaphong@yahoo.com",
    url="https://github.com/PyThaiNLP/oxidized-thainlp/",
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
    include_package_data=True,
    packages=find_packages(),
    zip_safe=False,
    rust_extensions=[
        RustExtension(
            "nlpo3", "Cargo.toml", binding=Binding.PyO3
        )
    ],
)
