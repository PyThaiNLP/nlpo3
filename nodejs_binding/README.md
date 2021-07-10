# oxidized-nodejs-thainlp

Like the root directory, but with nodejs binding.

## Features

- newmm dictionary-based word tokenization, at ultra fast speed
- support custom dictionary

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
Before build, your oxidized_nodejs_thainlp should look like this
```
- oxidized_nodejs_thainlp
    - rust_mod.d.ts
    - index.ts

```
After
```
- oxidized_nodejs_thainlp
    - rust_mod.d.ts
    - rust_mode.node
    - index.ts
    - index.js

```
## Usage

For now, copy the whole oxidized_nodejs_thainlp directory after build to your project 

In NodeJS:
```javascript
const  oxidizedThaiNLP = require(`${path_to_oxidized_nodejs_thainlp_directory}`)

oxidizedThaiNLP.loadDict("path/to/dict.file", "dict_name")
oxidizedThaiNLP.segment("สวัสดีครับ", "dict_name")
```

In TypeScript (run on node)

```typescript

import {segment,loadDict} from `${path_to_oxidized_nodejs_thainlp_directory}`

loadDict("path/to/dict.file", "dict_name")
segment("สวัสดีครับ", "dict_name")
```


Just that!


## Support

Please report issues at https://github.com/PyThaiNLP/oxidized-thainlp

# TODO

Find a way to build and publish on npm.



