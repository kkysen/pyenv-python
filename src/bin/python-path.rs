use pyenv_python::python_path;
use std::process::exit;
use print_bytes::println_bytes;

/// Print the current `python` path as determined by `pyenv`.
fn main() {
    let status = match python_path() {
        Some(path) => {
            println_bytes(&path);
            0
        }
        None => 1,
    };
    exit(status);
}
