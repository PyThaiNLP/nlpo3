
# BETA Release

```bash
pip install pythainlp-rust-modules
```


# Feature

- newmm word tokenization with default dict, ultra fast speed.
- support custom dict.
------------------------------------------------------------------------------------------------------------------

------------------------
# How To Build It Yourself.

## Requirements

- Rust 2018 Edition [Download here](https://www.rust-lang.org/tools/install)
- Python 3.6+
- PyO3 - already included in Cargo.toml
- *** For Linux *** sudo apt install python3-dev python-dev
- [Maturin](https://github.com/PyO3/maturin)



At oxidized-thainlp/rust_modules directory, run the following commands.

## Linux


```bash


maturin build --release -i python --manylinux off  
# Or 
maturin build --release -i python

```

## Windows Powershell
```shell
path\\to\\maturin.exe build --release -i python

```

## MacOS
```zsh
maturin build --release -i python
```

Note: You can try omitting "-i python". This will let Maturin build this lib for many versions of python if detected.

This should generate a wheel file which can be installed by pip

-------------------
# How To Use In Python

```python
from oxidized_thainlp import segment,load_dict

load_dict("path/to/dict.file","dict_name")
segment("สวัสดีครับ","dict_name")



```

Just that!

--------------------------------------------------------------------------------------------------------------------------------------------------------
# TODO

- Distribute module on pip or whatever, I really do not know much about Python env.

# oxidized-thainlp