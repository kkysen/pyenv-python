use pyenv_python::python_path;
use std::process::{exit, Command};
use std::env;

/// Run the current `python` (as determined by `pyenv`) with the given args.
fn main() {
    let path = python_path().expect("python not found");
    let status = Command::new(path)
        .args(env::args_os().skip(1))
        .status()
        .expect("failed to run python subprocess")
        .code().unwrap_or_default();
    exit(status);
}
