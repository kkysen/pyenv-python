use pyenv_python::{python_path, pyenv_python_path};
use std::process::exit;
use std::env;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

#[cfg(unix)]
fn run(path: &Path) {
    let err = exec::execvp(path, env::args_os());
    eprintln!("{:?}", err);
    exit(1);
}

#[cfg(not(unix))]
fn run(path: &Path) {
    use std::process::Command;
    let status = Command::new(path)
        .args(env::args_os().skip(1))
        .status()
        .expect("failed to run python subprocess")
        .code().unwrap_or_default();
    exit(status);
}

/// Run the current `python` (as determined by `pyenv`) with the given args (using std::process::Command).
fn main() {
    let path = python_path().expect("python not found");
    run(path.as_path())
}
