---
SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpO3 Node.js binding

[![Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg "Apache-2.0")](https://opensource.org/licenses/Apache-2.0)

Rust-powered Thai tokenization library for Node.js.

## Overview

- Node.js-native ESM package backed by a Rust `.node` addon.
- Three tokenizer classes:
  - `NewmmTokenizer` — Dictionary-based maximal-matching tokenizer.
  - `NewmmFstTokenizer` — FST-accelerated dictionary tokenizer.
  - `DeepcutTokenizer` — Neural network-based tokenizer.
- Framework-independent and suitable for plain Node.js applications.
- Built and type-checked with TypeScript `module: "NodeNext"` for Node-native ESM behavior.

## Runtime support

- Supported Node.js versions: 22.x, 23.x, 24.x, 25.x

For implementation details and design choices, see
[../docs/implementation.md](../docs/implementation.md).

## Install

Install the package via [npm](https://www.npmjs.com/package/nlpo3):

```shell
npm i nlpo3
```

## Quick start

### JavaScript (ESM)

```javascript
import { NewmmTokenizer, NewmmFstTokenizer, DeepcutTokenizer } from "nlpo3";

const tok = new NewmmTokenizer("/path/to/dict.txt");
const tokens = tok.segment("สวัสดีครับ");

const tokFst = new NewmmFstTokenizer("/path/to/dict.txt");
const tokensFst = tokFst.segment("สวัสดีครับ");

const deepcut = new DeepcutTokenizer();
const tokensDeepcut = deepcut.segment("สวัสดีครับ");
```

### TypeScript

```typescript
import { NewmmTokenizer, NewmmFstTokenizer, DeepcutTokenizer } from "nlpo3";
import type { SegmentOptions } from "nlpo3";

const tok = new NewmmTokenizer("/path/to/dict.txt");
const tokens: string[] = tok.segment("สวัสดีครับ");

const options: SegmentOptions = { safe: false, parallelChunkSize: 16384 };
const tokensParallel: string[] = tok.segment("สวัสดีครับ", options);
```

### Development note for local imports

When working on the binding itself under `module: "NodeNext"`, use explicit
`.js` extensions in relative TypeScript imports so the emitted code matches
Node.js ESM resolution:

```typescript
import type { TokenizerHandle } from "./rust_mod.js";
```

## Error handling

- `segment(...)` throws `Error` if tokenization or inference fails.
- Constructor calls can also throw `Error` when dictionary/model loading fails.

Handle these with standard JavaScript/TypeScript `try/catch` blocks.

## Segment options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `safe` | `boolean` | `false` | For `NewmmTokenizer` and `NewmmFstTokenizer`: avoid long run times on inputs with many ambiguous word boundaries |
| `parallelChunkSize` | `number \| undefined` | `undefined` | For `NewmmTokenizer` and `NewmmFstTokenizer`: target chunk size in bytes for chunked parallel processing; `undefined`, `0`, or low values disable parallel mode |

`DeepcutTokenizer.segment(text, parallelChunkSize?)` supports optional chunk-size configuration.

> **Note on parallel mode accuracy:** when `parallelChunkSize` is set, text is
> split into chunks before tokenization. Token sequences near chunk boundaries
> may differ from full-text results. This is acceptable for tasks such as text
> classification and word embedding, but may not be suitable for tasks that
> require precise linguistic unit identification.

## Dictionary

To keep the package small, nlpO3 does not include a dictionary.
For dictionary-based tokenizers you must provide one.

Recommended dictionaries:

- [words_th.txt][dict-pythainlp] from [PyThaiNLP][pythainlp] (~62,000 words, CC0-1.0)
- [word break dictionary][dict-libthai] from [libthai][libthai] (LGPL-2.1)

[pythainlp]: https://github.com/PyThaiNLP/pythainlp
[libthai]: https://github.com/tlwg/libthai/
[dict-pythainlp]: https://github.com/PyThaiNLP/pythainlp/blob/dev/pythainlp/corpus/words_th.txt
[dict-libthai]: https://github.com/tlwg/libthai/tree/master/data

## Support

- Issues: <https://github.com/PyThaiNLP/nlpo3/issues>

## License

nlpO3 Node.js binding is copyrighted by its authors
and licensed under terms of the Apache Software License 2.0 (Apache-2.0).
See file [LICENSE](./LICENSE) for details.
