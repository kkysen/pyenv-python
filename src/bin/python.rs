use pyenv_python::python_path;
use std::process::{exit, Command};
use std::env;

fn main() {
    let path = python_path().expect("python not found");
    let status = Command::new(path)
        .args(env::args_os())
        .status()
        .expect("failed to run python subprocess")
        .code().unwrap_or_default();
    exit(status);
}
