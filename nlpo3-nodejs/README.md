---
SPDX-FileCopyrightText: 2024 PyThaiNLP Project
SPDX-License-Identifier: Apache-2.0
---

# nlpO3 Node.js binding

[![Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg "Apache-2.0")](https://opensource.org/licenses/Apache-2.0)

Node.js binding for nlpO3, a Thai natural language processing library in Rust.

## Features

- Thai word tokenizer
  - Use maximal-matching dictionary-based tokenization algorithm
    and honor [Thai Character Cluster][tcc] boundaries
  - Fast backend in Rust
  - Support custom dictionary

[tcc]: https://dl.acm.org/doi/10.1145/355214.355225

## Build

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- Node.js v12 or newer

### Steps

```bash
# In this directory
npm run release
```

Before build, your `nlpo3/` directory should look like this:

```text
- nlpo3/
    - index.ts
    - rust_mod.d.ts
```

After build:

```text
- nlpo3/
    - index.js
    - index.ts
    - rust_mod.d.ts
    - rust_mode.node
```

## Install

For now, copy the whole `nlpo3/` directory after build to your project.

### npm (experitmental)

npm is still experimental and may not work on all platforms.
Please report issues at <https://github.com/PyThaiNLP/nlpo3/issues>

```shell
npm i nlpo3
```

## Usage

In JavaScript:

```javascript
const nlpO3 = require(`${path_to_nlpo3}`)

// load dictionary and tokenize a text with it
nlpO3.loadDict("path/to/dict.file", "dict_name")
nloO3.segment("สวัสดีครับ", "dict_name")
```

In TypeScript:

```typescript
import {segment, loadDict} from `${path_to_nlpo3}/index`

// load custom dictionary and tokenize a text with it
loadDict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

## Issues

Please report issues at <https://github.com/PyThaiNLP/nlpo3/issues>

## TODO

- Find a way to build binaries and publish on npm.

## License

nlpO3 Node binding is copyrighted by its authors
and licensed under terms of the Apache Software License 2.0 (Apache-2.0).
See file [LICENSE](./LICENSE) for details.
