# nlpo3

Like the root directory, but with nodejs binding.

## Build It Yourself

### Build requirements

- [Rust 2018 Edition](https://www.rust-lang.org/tools/install)
- NodeJS V12 ++

### Build steps
1.
```bash
# In this directory
npm run release
```

Before build, your `nlpo3` directory should look like this
```
- nlpo3
    - rust_mod.d.ts
    - index.ts
```

After
```
- nlpo3
    - rust_mod.d.ts
    - rust_mode.node
    - index.ts
    - index.js
```

## Usage

For now, copy the whole `nlpo3` directory after build to your project 

In NodeJS:
```javascript
const oxidizedNLP = require(`${path_to_nlpo3}`)

oxidizedNLP.loadDict("path/to/dict.file", "dict_name")
oxidizedNLP.segment("สวัสดีครับ", "dict_name")
```

In TypeScript (run on node)
```typescript
import {segment,loadDict} from `${path_to_nlpo3}/index`

loadDict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```

Just that!


# TODO

Find a way to build and publish on npm.
