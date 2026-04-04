# SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

# Deepcut tokenizer package for nlpO3.
#
# Install this package alongside `nlpo3` to enable DeepcutTokenizer:
#
#     pip install nlpo3 nlpo3-deepcut
#
# Authors:
# Thanathip Suntorntip
# Arthit Suriyawongkul

from __future__ import annotations

from ._nlpo3_deepcut_backend import DeepcutTokenizer

__all__ = ["DeepcutTokenizer"]
