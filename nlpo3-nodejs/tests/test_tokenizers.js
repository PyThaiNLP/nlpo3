// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/**
 * Integration tests for the nlpO3 Node.js binding.
 *
 * Requires the native addon to be built and TypeScript compiled first:
 *   npm run build && tsc
 *   node --test tests/test_tokenizers.js
 *
 * Uses Node.js built-in test runner (node:test, available since Node 18).
 * This package supports Node.js 22, 23, 24, and 25.
 */

import assert from "node:assert/strict";
import { test, describe } from "node:test";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { NewmmTokenizer, NewmmFstTokenizer, DeepcutTokenizer } from "../nlpo3/index.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DICT_PATH = path.join(__dirname, "data", "test_dict.txt");

test("runtime version is supported", () => {
    const major = Number(process.versions.node.split(".")[0]);
    assert.ok([22, 23, 24, 25].includes(major), "Node.js 22/23/24/25 is required");
});

// ---------------------------------------------------------------------------
// NewmmTokenizer
// ---------------------------------------------------------------------------

describe("NewmmTokenizer", () => {
    test("constructor loads dictionary without error", () => {
        assert.doesNotThrow(() => new NewmmTokenizer(DICT_PATH));
    });

    test("segment returns an array", () => {
        const tok = new NewmmTokenizer(DICT_PATH);
        const result = tok.segment("ไข่คน");
        assert.ok(Array.isArray(result), "result should be an array");
    });

    test("segment empty string returns empty array", () => {
        const tok = new NewmmTokenizer(DICT_PATH);
        assert.deepStrictEqual(tok.segment(""), []);
    });

    test("segment reconstructs input", () => {
        const tok = new NewmmTokenizer(DICT_PATH);
        const text = "ไข่คน";
        assert.strictEqual(tok.segment(text).join(""), text);
    });

    test("segment with safe=true returns an array", () => {
        const tok = new NewmmTokenizer(DICT_PATH);
        assert.ok(Array.isArray(tok.segment("ไข่คน", { safe: true })));
    });

    test("segment with parallelChunkSize returns an array", () => {
        const tok = new NewmmTokenizer(DICT_PATH);
        assert.ok(Array.isArray(tok.segment("ไข่คน", { parallelChunkSize: 16384 })));
    });

    test("same instance reused across many calls", () => {
        const tok = new NewmmTokenizer(DICT_PATH);
        for (const t of ["ไข่คน", "ค่าจ้าง", "คน"]) {
            const r = tok.segment(t);
            assert.ok(Array.isArray(r));
            assert.strictEqual(r.join(""), t);
        }
    });

    test("constructor throws for nonexistent dictionary path", () => {
        assert.throws(
            () => new NewmmTokenizer("/nonexistent/path/to/dict.txt"),
            /failed to load dictionary/i,
        );
    });
});

// ---------------------------------------------------------------------------
// NewmmFstTokenizer
// ---------------------------------------------------------------------------

describe("NewmmFstTokenizer", () => {
    test("constructor loads dictionary without error", () => {
        assert.doesNotThrow(() => new NewmmFstTokenizer(DICT_PATH));
    });

    test("segment returns an array", () => {
        const tok = new NewmmFstTokenizer(DICT_PATH);
        assert.ok(Array.isArray(tok.segment("ไข่คน")));
    });

    test("segment empty string returns empty array", () => {
        const tok = new NewmmFstTokenizer(DICT_PATH);
        assert.deepStrictEqual(tok.segment(""), []);
    });

    test("segment reconstructs input", () => {
        const tok = new NewmmFstTokenizer(DICT_PATH);
        const text = "ไข่คน";
        assert.strictEqual(tok.segment(text).join(""), text);
    });

    test("constructor throws for nonexistent dictionary path", () => {
        assert.throws(
            () => new NewmmFstTokenizer("/nonexistent/path/to/dict.txt"),
            /failed to load dictionary/i,
        );
    });

    test("output matches NewmmTokenizer for same input", () => {
        const trie = new NewmmTokenizer(DICT_PATH);
        const fst = new NewmmFstTokenizer(DICT_PATH);
        const text = "ไข่คน";
        assert.deepStrictEqual(trie.segment(text), fst.segment(text));
    });
});

// ---------------------------------------------------------------------------
// DeepcutTokenizer
// ---------------------------------------------------------------------------

describe("DeepcutTokenizer", () => {
    test("constructor succeeds", () => {
        assert.doesNotThrow(() => new DeepcutTokenizer());
    });

    test("segment returns a non-empty array", () => {
        const tok = new DeepcutTokenizer();
        const result = tok.segment("ทดสอบ");
        assert.ok(Array.isArray(result));
        assert.ok(result.length > 0);
    });

    test("segment empty string returns empty array", () => {
        const tok = new DeepcutTokenizer();
        assert.deepStrictEqual(tok.segment(""), []);
    });

    test("segment reconstructs input", () => {
        const tok = new DeepcutTokenizer();
        const text = "ทดสอบการตัดคำ";
        assert.strictEqual(tok.segment(text).join(""), text);
    });

    test("same instance reused across many calls", () => {
        const tok = new DeepcutTokenizer();
        for (const t of ["ทดสอบ", "สวัสดีครับ", "การตัดคำ"]) {
            const r = tok.segment(t);
            assert.ok(Array.isArray(r));
            assert.ok(r.length > 0);
        }
    });
});
