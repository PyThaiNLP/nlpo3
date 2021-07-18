# nlpo3

Like the root directory, but with nodejs binding.

## Build/Install

### Requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- NodeJS V12 ++

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

## Usage

For now, copy the whole `nlpo3` directory after build to your project 

In NodeJS:
```javascript
const nlpO3 = require(`${path_to_nlpo3}`)

// tokenize a text with default dictionary
nloO3.segment("สวัสดีครับ")

// load custom dictionary and tokenize with it
nlpO3.loadDict("path/to/dict.file", "dict_name")
nloO3.segment("สวัสดีครับ", "dict_name")
```

In TypeScript (run on node)
```typescript
import {segment, loadDict} from `${path_to_nlpo3}/index`

// tokenize a text with default dictionary
segment("สวัสดีครับ")

// load custom dictionary and tokenize with it
loadDict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

Just that!


# TODO

Find a way to build and publish on npm.
