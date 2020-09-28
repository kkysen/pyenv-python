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

#### Fast Script Invocation
For all globally install programs, like through `pip install`, 
`pyenv` places a shim file in `$PYENV_ROOT/shims`, which is placed in `$PATH`.  
This shim has to call the real script throuh `pyenv`, 
which makes every script startup very slow, 
adding up to 1 second to every script,
even on simple `--help` commands.

As a solution, `pyenv-python`'s `python` executable supports 
invoking scripts with no overhead.
If `python` is symlinked to `script`,
then when `./script` is run, it calls `python script`,
where `script` should be in the same directory as the real `python`.
That is, if `python` is the real `python` executable,
then `./script` calls `python "$(dirname "$(which python)")"/script`.

Installed `python` scripts are normally installed
in the same directory as `python`, 
so this makes it very easy to invoke scripts with no `pyenv` overhead.

### Performance
On my local computer, `$CARGO_HOME/bin/python --version` runs 
about 23x faster than `$PYENV_ROOT/shims/python --version`.

```console
pyenv-python on  master [!] is 📦 v0.1.0 via 🦀 v1.44.0-nightly took 23s
❯ hyperfine '$CARGO_HOME/bin/python --version' '$PYENV_ROOT/shims/python --version'
Benchmark #1: $CARGO_HOME/bin/python --version
  Time (mean ± σ):      11.8 ms ±   1.3 ms    [User: 1.3 ms, System: 9.6 ms]
  Range (min … max):    10.1 ms …  18.4 ms    181 runs

Benchmark #2: $PYENV_ROOT/shims/python --version
  Time (mean ± σ):     270.6 ms ±   6.6 ms    [User: 20.3 ms, System: 238.9 ms]
  Range (min … max):   265.7 ms … 288.5 ms    10 runs

Summary
  '$CARGO_HOME/bin/python --version' ran
   23.00 ± 2.62 times faster than '$PYENV_ROOT/shims/python --version'
```

### Installation
It's just published to `crates.io`, 
so you need `cargo` from `rustup` to install it.

Then `cargo install pyenv-python` will install the `python` wrapper.
For this `python` to wrap the `pyenv` `python` or the system `python`, 
`$CARGO_HOME/bin` must be before any other `python`s in `$PATH`.

This `python` wrapper also supports a few other commands.
* `python --path` prints the path of the `python` that it will execute.
* `python --prefix` prints the directory of the `python` that it will execute.
  This is the same as what `python -c 'import sys; print(sys.prefix)''` prints.

These extra commands aren't compatible with actual `python`,
but they don't clash with any actual `python` commands, 
and they're very useful for inspection.
Previously, there was a separate `python-path` executable
that did what `python --path` now does,
but having one executable is much simpler.
