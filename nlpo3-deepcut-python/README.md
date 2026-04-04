---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpo3-deepcut

Deepcut CNN-based Thai word tokenizer for [nlpO3](https://github.com/PyThaiNLP/nlpo3).

This is an optional extension package for [`nlpo3`](https://pypi.org/project/nlpo3/).
It ships the deepcut ONNX model and the [tract-onnx](https://github.com/sonos/tract)
runtime compiled into a compact Rust extension module.

## Install

```bash
pip install nlpo3 nlpo3-deepcut
```

## Usage

```python
from nlpo3 import DeepcutTokenizer

# Use the bundled model (installed by nlpo3-deepcut)
tok = DeepcutTokenizer()
tokens = tok.segment("สวัสดีครับ")
print(tokens)

# Use a custom ONNX model file
tok = DeepcutTokenizer(model_path="/path/to/custom.onnx")
```

You can also import directly from `nlpo3_deepcut`:

```python
from nlpo3_deepcut import DeepcutTokenizer

tok = DeepcutTokenizer()
tokens = tok.segment("สวัสดีครับ")
```

## Why a separate package?

The deepcut tokenizer uses an ONNX neural model and the `tract-onnx` inference
engine.  Bundling them in the base `nlpo3` package would make every install
significantly larger, even for users who only need the lightweight
dictionary-based `NewmmTokenizer`.

Splitting the packages lets users choose:

| Need | Install |
|------|---------|
| Dictionary-based tokenization only | `pip install nlpo3` |
| Dictionary + neural deepcut | `pip install nlpo3 nlpo3-deepcut` |

## Error without nlpo3-deepcut

If you call `DeepcutTokenizer()` without installing this package, `nlpo3`
raises a descriptive `ImportError`:

```
ImportError: DeepcutTokenizer requires the nlpo3-deepcut package.
Install it with:  pip install nlpo3-deepcut
```

## License

Apache-2.0. See [LICENSE](https://github.com/PyThaiNLP/nlpo3/blob/main/nlpo3-deepcut-python/LICENSE).

The bundled deepcut ONNX model is derived from
[LEKCut](https://github.com/PyThaiNLP/LEKCut) /
[deepcut](https://github.com/rkcosmos/deepcut) (MIT License).
