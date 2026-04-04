---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: CC0-1.0
---

# Changelog — nlpo3-deepcut

All notable changes to the `nlpo3-deepcut` Python package are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This package uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] — 2026-04-04

### Added

- Initial release as a standalone optional package.
- `DeepcutTokenizer` class: CNN-based Thai word tokenizer backed by the
  deepcut ONNX model and the `tract-onnx` inference engine.
- Bundled deepcut ONNX model (derived from LEKCut / deepcut, MIT License).
- `segment(text, parallel_chunk_size=None)` method with GIL-releasing
  parallel chunk mode.
- PEP 561 support: `py.typed` marker and `_nlpo3_deepcut_backend.pyi` stub.

### Notes

- Install alongside `nlpo3>=2.0,<3` for `DeepcutTokenizer` support.
- Users who install only `nlpo3` (without this package) receive a descriptive
  `ImportError` with an installation hint when they try to use
  `DeepcutTokenizer`.
