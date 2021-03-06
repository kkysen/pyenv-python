#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::env;
use same_file::Handle;
use is_executable::IsExecutable;

mod version;

/// Returns what `$(pyenv root)` would return.
/// That is, `$PYENV_ROOT` or `$HOME/.pyenv` if they exist
pub fn pyenv_root() -> Option<PathBuf> {
    env::var_os("PYENV_ROOT")
        .or_else(|| env::var_os("HOME").map(|mut home| {
            home.push(".pyenv");
            home
        }))
        .map(|root| root.into())
        .filter(|root: &PathBuf| root.is_dir())
}

/// Returns the current pyenv version as determined by
///     <https://github.com/pyenv/pyenv#choosing-the-python-version>.
/// If none is found, then None is returned.
pub fn pyenv_version() -> Option<String> {
    let root = pyenv_root()?;
    let version = version::pyenv_version(root.as_path())?;
    Some(version)
}

/// Returns the `python` path as determined by `pyenv`.
/// Returns None if `pyenv` isn't setup correctly.
pub fn pyenv_python_path() -> Option<PathBuf> {
    // root will fail faster than version, so unwrap it first
    let root = pyenv_root()?;
    let version = version::pyenv_version(root.as_path())?;
    let mut path = root;
    path.push("versions");
    path.push(version);
    path.push("bin");
    path.push("python");
    Some(path)
        .filter(|path| path.is_file())
        .filter(|path| path.is_executable())
}

/// Returns the system `python` on `$PATH`, excluding this program (when run as `python`).
/// If pyenv is installed, then `$PYENV_ROOT/shims/python` is skipped, too.
/// Otherwise, an infinite loop would be formed between ourselves and `$PYENV_ROOT/shims/python`.
pub fn system_python_path() -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let current_path = {
        let mut path = env::current_exe().ok()?;
        path.set_file_name("python");
        path
    };
    let current_handle = Handle::from_path(current_path).ok()?;
    let pyenv_python_handle = pyenv_root()
        .map(|root| root.join("shims/python"))
        .filter(|path| path.is_file())
        .and_then(|path| Handle::from_path(path).ok());
    for mut path in env::split_paths(&path_var) {
        path.push("python");
        if let Ok(handle) = Handle::from_path(path.as_path()) {
            // Ok if path exists
            if current_handle != handle && pyenv_python_handle != Some(handle) {
                return Some(path);
            }
        }
    }
    None
}

/// Get the path of `python` using `pyenv`, i.e., using [`pyenv_version()`].
/// If [`pyenv_version()`] returns None, then the system `python` is used, i.e. $PATH.
pub fn python_path() -> Option<PathBuf> {
    pyenv_python_path().or_else(system_python_path)
}
