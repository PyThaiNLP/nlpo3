# SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

import unittest

from nlpo3_deepcut import DeepcutTokenizer


class TestDeepcutTokenizer(unittest.TestCase):
    def setUp(self):
        self.tok = DeepcutTokenizer()

    def test_empty_input(self):
        self.assertEqual(self.tok.segment(""), [])

    def test_returns_list(self):
        result = self.tok.segment("ทดสอบ")
        self.assertIsInstance(result, list)
        self.assertTrue(len(result) > 0)

    def test_basic(self):
        result = self.tok.segment("ทดสอบการตัดคำ")
        self.assertIsInstance(result, list)
        self.assertTrue(len(result) > 0)
        # Reconstructed text must equal the input.
        self.assertEqual("".join(result), "ทดสอบการตัดคำ")

    def test_reconstructs_input(self):
        text = "หมอนทองตากลมหูว์MBK39"
        result = self.tok.segment(text)
        self.assertEqual("".join(result), text)

    def test_shared_instance(self):
        # The same DeepcutTokenizer instance (with Arc-backed model) can be
        # reused across many calls — the ONNX model is compiled once and
        # shared by all calls to this tokenizer object.
        tok = self.tok
        texts = ["ทดสอบ", "สวัสดีครับ", "การตัดคำ"]
        results = [tok.segment(t) for t in texts]
        self.assertEqual(len(results), 3)
        for r in results:
            self.assertIsInstance(r, list)


# ---------------------------------------------------------------------------
# DeepcutTokenizer is also accessible via the nlpo3 base package shim
# ---------------------------------------------------------------------------


class TestDeepcutTokenizerViaBasePackage(unittest.TestCase):
    """Verify that nlpo3.DeepcutTokenizer delegates to nlpo3_deepcut when
    the latter is installed.
    """

    def test_importable_from_nlpo3(self):
        from nlpo3 import DeepcutTokenizer as NlpO3DeepcutTokenizer

        tok = NlpO3DeepcutTokenizer()
        result = tok.segment("ทดสอบ")
        self.assertIsInstance(result, list)
        self.assertTrue(len(result) > 0)

    def test_returns_same_results(self):
        from nlpo3 import DeepcutTokenizer as NlpO3DeepcutTokenizer

        text = "ทดสอบการตัดคำ"
        tok_direct = DeepcutTokenizer()
        tok_shim = NlpO3DeepcutTokenizer()
        self.assertEqual(tok_direct.segment(text), tok_shim.segment(text))


# ---------------------------------------------------------------------------
# Thread safety
# ---------------------------------------------------------------------------


class TestDeepcutTokenizerThreadSafety(unittest.TestCase):
    def test_concurrent_segment(self):
        import concurrent.futures

        tok = DeepcutTokenizer()
        texts = ["ทดสอบ", "สวัสดีครับ", "การตัดคำ"] * 10
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(tok.segment, texts))
        self.assertEqual(len(results), 30)
        for r in results:
            self.assertIsInstance(r, list)
