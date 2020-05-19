use pyenv_python::python_path;
use std::process::exit;
use std::env;

/// Run the current `python` (as determined by `pyenv`) with the given args (using exec::Command).
#[cfg(unix)]
fn main() {
    let path = python_path().expect("python not found");
    let err = exec::execvp(path, env::args_os());
    eprintln!("{:?}", err);
    exit(1);
}

/// Run the current `python` (as determined by `pyenv`) with the given args (using std::process::Command).
#[cfg(not(unix))]
fn main() {
    use std::process::Command;
    let path = python_path().expect("python not found");
    let status = Command::new(path)
        .args(env::args_os().skip(1))
        .status()
        .expect("failed to run python subprocess")
        .code().unwrap_or_default();
    exit(status);
}
