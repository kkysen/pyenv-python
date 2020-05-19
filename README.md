# pyenv-python
A pyenv shim for python that's much faster than pyenv.

### Usage
This `python` shim, i.e., `$CARGO_HOME/bin/python`, 
loads much faster than `pyenv`'s `python` shim, which is written in Bash. 

This helps a lot when running CLI scripts in Python, 
since the time is takes to run the CLI script is often 
less than the time it takes `pyenv` to find `python`. 

For example, with `starship`, `python --version` is run every time 
when you're in a Python directory to display the current 
Python version on the prompt, but if `pyenv`'s `python` shim 
takes ~1 sec, it makes the prompt painfully slow.
Using this `python` shim, the time for `python --version` is unnoticeable.

### Performance
On my local computer, `$CARGO_HOME/bin/python --version` runs 
about 10x faster than `$PYENV_ROOT/shims/python --version`.

```
workspace/misc/pyenv-python-test via 🐍 v3.5.9 took 19s
❯ hyperfine '$CARGO_HOME/bin/python --version'
Benchmark #1: $CARGO_HOME/bin/python --version
  Time (mean ± σ):      65.3 ms ±   5.8 ms    [User: 1.8 ms, System: 56.7 ms]
  Range (min … max):    57.6 ms …  85.8 ms    32 runs


workspace/misc/pyenv-python-test via 🐍 v3.5.9 took 21s
❯ hyperfine '$PYENV_ROOT/shims/python --version'
Benchmark #1: $PYENV_ROOT/shims/python --version
  Time (mean ± σ):     715.3 ms ± 164.2 ms    [User: 57.3 ms, System: 649.7 ms]
  Range (min … max):   593.1 ms … 1055.0 ms    10 runs
```

### Installation
It's just published to `crates.io`, so you need `cargo` from `rustup` to install. 

Then `cargo install pyenv-python` will install `python` and `python-path`.
For this `python` to wrap the `pyenv` `python` or the system `python`, 
it `$CARGO_HOME/bin` must be before any other `python`s in `$PATH`.
