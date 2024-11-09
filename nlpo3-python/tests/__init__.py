# SPDX-FileCopyrightText: 2024 PyThaiNLP Project
# SPDX-License-Identifier: Apache-2.0

"""
Unit test
"""

import sys
import unittest

sys.path.append("../nlpo3")

loader = unittest.TestLoader()
testSuite = loader.discover("tests")
testRunner = unittest.TextTestRunner(verbosity=1)
testRunner.run(testSuite)
