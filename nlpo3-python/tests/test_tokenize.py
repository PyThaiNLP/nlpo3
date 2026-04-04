# SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

import unittest

from nlpo3 import DeepcutTokenizer, NewmmFstTokenizer, NewmmTokenizer


DICT_FILENAME = "data/test_dict.txt"


class TestNewmmTokenizer(unittest.TestCase):
    def setUp(self):
        self.tok = NewmmTokenizer(DICT_FILENAME)

        self.TEXT_1 = "หมอนทองตากลมหูว์MBK39 :.ฉฺ๐๐๓-#™±"
        self.TEXT_2 = "ทดสอบ"

        self.LONG_TEXT = (
            "ไต้หวัน (แป่ะเอ๋ยี้: Tâi-oân; ไต่อวัน) หรือ ไถวาน "
            "(อักษรโรมัน: Taiwan; จีนตัวย่อ: 台湾; จีนตัวเต็ม: 臺灣/台灣; พินอิน: "
            "Táiwān; ไถวาน) หรือชื่อทางการว่า สาธารณรัฐจีน (จีนตัวย่อ: 中华民国; "
            "จีนตัวเต็ม: 中華民國; พินอิน: Zhōnghuá "
            "Mínguó) เป็นรัฐในทวีปเอเชียตะวันออก[7][8][9] ปัจจุบันประกอบด้วย"
            "เกาะใหญ่ 5 แห่ง คือ จินเหมิน (金門), ไต้หวัน, เผิงหู (澎湖), หมาจู่ "
            "(馬祖), และอูชิว (烏坵) กับทั้งเกาะเล็กเกาะน้อยอีกจำนวนหนึ่ง "
            'ท้องที่ดังกล่าวเรียกรวมกันว่า "พื้นที่ไต้หวัน" (臺灣地區)\n'
        )

        self.DANGER_TEXT_1 = (
            "ชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิชิ"
        )

        self.DANGER_TEXT_2 = (
            "ด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้าน"
            "หน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้าน"
        )

        self.DANGER_TEXT_3 = (
            "ด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้า"
            "ด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้า"
            "ด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้าด้านหน้า"
            "ด้านหน้าด้านหน้าด้านกกกกกก"
            "กกกกกกกกกกกกกกกกกกกกกกกกกกกกกกกกกกกกกก"
        )

    def test_empty_input(self):
        self.assertEqual(self.tok.segment(""), [])

    def test_space(self):
        self.assertEqual(self.tok.segment(" "), [" "])

    def test_basic(self):
        self.assertEqual(
            self.tok.segment("ไข่คน2021"),
            ["ไข่", "คน", "2021"],
        )

    def test_maximal_matching(self):
        # dict contains both "ค่า" and "ค่าจ้าง"; maximal match picks the longer
        self.assertIn(
            "ค่าจ้าง",
            self.tok.segment(
                "ค่าจ้างที่ได้รับต้องทำให้แรงงาน"
                "สามารถเลี้ยงดูตัวเองและครอบครัว"
                "อย่างสมศักดิ์ศรีความเป็นมนุษย์",
            ),
        )

    def test_returns_list(self):
        self.assertIsInstance(self.tok.segment(self.TEXT_1), list)
        self.assertIsInstance(self.tok.segment(self.TEXT_2), list)
        self.assertIsInstance(self.tok.segment(self.LONG_TEXT), list)
        self.assertIsInstance(self.tok.segment(self.DANGER_TEXT_1), list)
        self.assertIsInstance(self.tok.segment(self.DANGER_TEXT_2), list)
        self.assertIsInstance(self.tok.segment(self.DANGER_TEXT_3), list)

    def test_shared_instance(self):
        # The same instance can be reused across many calls — the dictionary
        # is loaded once and shared by all calls to this tokenizer object.
        tok = self.tok
        results = [tok.segment(t) for t in [self.TEXT_1, self.TEXT_2, "ทดสอบ"]]
        self.assertEqual(len(results), 3)
        for r in results:
            self.assertIsInstance(r, list)

    def test_safe_mode(self):
        result = self.tok.segment(self.DANGER_TEXT_1, safe=True)
        self.assertIsInstance(result, list)

    def test_parallel_mode(self):
        result = self.tok.segment(self.LONG_TEXT, parallel_chunk_size=16_384)
        self.assertIsInstance(result, list)
        self.assertTrue(len(result) > 0)


class TestNewmmFstTokenizer(unittest.TestCase):
    def setUp(self):
        self.tok = NewmmFstTokenizer(DICT_FILENAME)

    def test_empty_input(self):
        self.assertEqual(self.tok.segment(""), [])

    def test_basic(self):
        self.assertEqual(
            self.tok.segment("ไข่คน2021"),
            ["ไข่", "คน", "2021"],
        )

    def test_returns_list(self):
        self.assertIsInstance(self.tok.segment("ทดสอบ"), list)

    def test_shared_instance(self):
        tok = self.tok
        results = [tok.segment(t) for t in ["ทดสอบ", "ไข่คน", "สวัสดี"]]
        self.assertEqual(len(results), 3)


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
# Error handling
# ---------------------------------------------------------------------------


class TestNewmmTokenizerErrors(unittest.TestCase):
    def test_bad_dict_path_raises(self):
        with self.assertRaises(RuntimeError):
            NewmmTokenizer("/nonexistent/path/to/dict.txt")


class TestNewmmFstTokenizerErrors(unittest.TestCase):
    def test_bad_dict_path_raises(self):
        with self.assertRaises(RuntimeError):
            NewmmFstTokenizer("/nonexistent/path/to/dict.txt")


# ---------------------------------------------------------------------------
# Thread safety
# ---------------------------------------------------------------------------


class TestNewmmTokenizerThreadSafety(unittest.TestCase):
    def test_concurrent_segment(self):
        import concurrent.futures

        tok = NewmmTokenizer(DICT_FILENAME)
        texts = ["ไข่คน2021", "ทดสอบ", "สวัสดีครับ"] * 10
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(tok.segment, texts))
        self.assertEqual(len(results), 30)
        for r in results:
            self.assertIsInstance(r, list)

    def test_concurrent_segment_matches_serial(self):
        import concurrent.futures

        tok = NewmmTokenizer(DICT_FILENAME)
        texts = ["ไข่คน2021", "ทดสอบ", "สวัสดีครับ"]
        serial = [tok.segment(t) for t in texts]
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
            parallel = list(pool.map(tok.segment, texts))
        self.assertEqual(serial, parallel)
