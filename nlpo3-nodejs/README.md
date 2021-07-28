<a href="https://opensource.org/licenses/Apache-2.0"><img alt="License" src="https://img.shields.io/badge/License-Apache%202.0-blue.svg"/></a>

# nlpO3 Node.js binding

Node.js binding for nlpO3, a Thai natural language processing library in Rust.

## Features

- Thai word tokenizer
  - use maximal-matching dictionary-based tokenization algorithm and honor Thai Character Cluster boundaries
  - fast backend in Rust
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

Before build, your `nlpo3/` directory should look like this:
```
- nlpo3/
    - index.ts
    - rust_mod.d.ts
```

After build:
```
- nlpo3/
    - index.js
    - index.ts
    - rust_mod.d.ts
    - rust_mode.node
```

## Install

For now, copy the whole `nlpo3/` directory after build to your project.

### npm (experitmental)

npm is still experimental and may not work on all platforms. Please report issues at https://github.com/PyThaiNLP/nlpo3/issues

```bash
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

Please report issues at https://github.com/PyThaiNLP/nlpo3/issues

# TODO

- Find a way to build binaries and publish on npm.
