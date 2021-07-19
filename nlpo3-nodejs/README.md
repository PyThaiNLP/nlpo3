# nlpO3 Node.js binding

Node.js binding for nlpO3, a Thai natural language processing library in Rust.

## Features

- Word tokenizer
  - maximal-matching dictionary-based tokenization
  - 2x faster than similar pure Python implementation (PyThaiNLP's newmm)
  - support custom dictionary

## Build

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- Node.js v12 or newer

### Steps

```bash
# In this directory
npm run release
```

Before build, your `nlpo3` directory should look like this
```
- nlpo3
    - index.ts
    - rust_mod.d.ts
```

After
```
- nlpo3
    - index.js
    - index.ts
    - rust_mod.d.ts
    - rust_mode.node
```

## Install

For now, copy the whole `nlpo3` directory after build to your project.

## Usage

In JavaScript:
```javascript
const nlpO3 = require(`${path_to_nlpo3}`)

// tokenize a text with default dictionary
nloO3.segment("สวัสดีครับ")

// load custom dictionary and tokenize with it
nlpO3.loadDict("path/to/dict.file", "dict_name")
nloO3.segment("สวัสดีครับ", "dict_name")
```

In TypeScript:
```typescript
import {segment, loadDict} from `${path_to_nlpo3}/index`

// tokenize a text with default dictionary
segment("สวัสดีครับ")

// load custom dictionary and tokenize with it
loadDict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

## Issues

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp/issues

# TODO

- Find a way to build and publish on npm.
