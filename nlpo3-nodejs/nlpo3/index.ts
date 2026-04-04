// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

import type { TokenizerHandle } from "./rust_mod.js";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const native = require("./rust_mod.node") as typeof import("./rust_mod.js");

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/**
 * Options for dictionary-based tokenizer segment calls.
 *
 * Both flags are accepted for interface consistency but are ignored by
 * {@link DeepcutTokenizer}, which uses a fixed neural-network inference path.
 */
export interface SegmentOptions {
    /**
     * Enable safe mode to avoid long run times on inputs with many ambiguous
     * word boundaries.
     * Default: `false`.
     */
    safe?: boolean;
    /**
     * Target chunk size in bytes for parallel processing.
     * `undefined`, `0`, or too-small values disable parallel mode.
     */
    parallelChunkSize?: number;
}

// ---------------------------------------------------------------------------
// NewmmTokenizer
// ---------------------------------------------------------------------------

/**
 * Dictionary-based maximal-matching Thai word tokenizer (TrieChar backend).
 *
 * Create **one instance** and reuse it for every segment call — the dictionary
 * is loaded only once and the same instance is safe to use for all concurrent
 * async operations on the Node.js event loop.
 *
 * For Node.js worker threads, create one instance per worker; there is no
 * shared-memory mechanism for passing a tokenizer handle across thread
 * boundaries.
 *
 * @example
 * ```typescript
 * import { NewmmTokenizer } from "nlpo3";
 *
 * const tok = new NewmmTokenizer("/path/to/dict.txt");
 *
 * // Reuse tok for every request — the dictionary is never reloaded.
 * const tokens = tok.segment("สวัสดีครับ");
 * const tokensParallel = tok.segment("สวัสดีครับ", { parallelChunkSize: 16384 });
 * ```
 */
export class NewmmTokenizer {
    private readonly _handle: TokenizerHandle;

    /**
     * @param dictPath  Path to a one-word-per-line dictionary file.
     */
    constructor(dictPath: string) {
        this._handle = native.newmmTokenizerNew(dictPath);
    }

    /**
     * Tokenize `text` into words.
     *
     * @param text     Input text.
     * @param options  Optional flags: `safe` and `parallelChunkSize`.
     * @returns        Array of word tokens.
     *
     * @remarks
     * When {@link SegmentOptions.parallelChunkSize} is set, text is split into
     * chunks before tokenization. Token sequences near chunk boundaries may
     * differ from full-text results. Suitable for classification and embedding
     * tasks; not recommended for tasks requiring precise token boundaries.
     */
    segment(text: string, options: SegmentOptions = {}): string[] {
        const { safe = false, parallelChunkSize } = options;
        return native.newmmTokenizerSegment(this._handle, text, safe, parallelChunkSize ?? null);
    }
}

// ---------------------------------------------------------------------------
// NewmmFstTokenizer
// ---------------------------------------------------------------------------

/**
 * Dictionary-based maximal-matching Thai word tokenizer (FST backend).
 *
 * Uses the same algorithm as {@link NewmmTokenizer} but stores the dictionary
 * in a minimized finite-state automaton, reducing memory use by ~49× at the
 * cost of slower per-lookup speed.
 *
 * Create **one instance** and reuse it for every segment call.
 *
 * @example
 * ```typescript
 * import { NewmmFstTokenizer } from "nlpo3";
 *
 * const tok = new NewmmFstTokenizer("/path/to/dict.txt");
 * const tokens = tok.segment("สวัสดีครับ");
 * ```
 */
export class NewmmFstTokenizer {
    private readonly _handle: TokenizerHandle;

    /**
     * @param dictPath  Path to a one-word-per-line dictionary file.
     */
    constructor(dictPath: string) {
        this._handle = native.newmmFstTokenizerNew(dictPath);
    }

    /**
     * Tokenize `text` into words.
     *
     * @param text     Input text.
     * @param options  Optional flags: `safe` and `parallelChunkSize`.
     * @returns        Array of word tokens.
     *
     * @remarks
     * When {@link SegmentOptions.parallelChunkSize} is set, text is split into
     * chunks before tokenization. Token sequences near chunk boundaries may
     * differ from full-text results. Suitable for classification and embedding
     * tasks; not recommended for tasks requiring precise token boundaries.
     */
    segment(text: string, options: SegmentOptions = {}): string[] {
        const { safe = false, parallelChunkSize } = options;
        return native.newmmFstTokenizerSegment(this._handle, text, safe, parallelChunkSize ?? null);
    }
}

// ---------------------------------------------------------------------------
// DeepcutTokenizer
// ---------------------------------------------------------------------------

/**
 * Neural CNN-based Thai word tokenizer (deepcut).
 *
 * Uses a bundled ONNX model for inference.  The model is compiled on
 * construction.  Internally the model is reference-counted, so construction
 * after the first time is O(1).
 *
 * Create **one instance** and reuse it for every segment call — the model is
 * compiled only once.
 *
 * @example
 * ```typescript
 * import { DeepcutTokenizer } from "nlpo3";
 *
 * const tok = new DeepcutTokenizer();
 * const tokens = tok.segment("สวัสดีครับ");
 * ```
 */
export class DeepcutTokenizer {
    private readonly _handle: TokenizerHandle;

    constructor() {
        this._handle = native.deepcutTokenizerNew();
    }

    /**
     * Tokenize `text` using the deepcut CNN model.
     *
     * @param text              Input text.
     * @param parallelChunkSize Target chunk size in bytes for parallel mode.
     *                          `undefined`, `0`, or low values disable parallel mode.
     * @returns                 Array of word tokens.
     *
     * @remarks
     * When `parallelChunkSize` is set, text is split into chunks. Because
     * deepcut uses a fixed-width context window, characters near chunk
     * boundaries have fewer adjacent context characters, so token sequences
     * near boundaries may differ from full-text results. Suitable for
     * classification and embedding tasks; not recommended for tasks requiring
     * precise token boundaries.
     */
    segment(text: string, parallelChunkSize?: number): string[] {
        return native.deepcutTokenizerSegment(this._handle, text, parallelChunkSize ?? null);
    }
}
