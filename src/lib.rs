use std::path::PathBuf;
use std::env;
use std::ffi::OsString;

/// Returns the current pyenv version as determined by
///     <https://github.com/pyenv/pyenv#choosing-the-python-version>.
/// If none is found, then None is returned.
pub fn pyenv_version() -> Option<OsString> {
    None
}

/// Returns what $(pyenv root) would return.
/// That is, $PYENV_ROOT or $HOME/.pyenv if they exist
pub fn pyenv_root() -> Option<PathBuf> {
    env::var_os("PYENV_ROOT")
        .or_else(|| env::var_os("HOME").map(|mut home| {
            home.push(".pyenv");
            home
        }))
        .map(|root| root.into())
        .filter(|root| root.is_dir())
}

/// Returns the `python` path as determined by `pyenv`.
/// Returns None if `pyenv` isn't setup correctly.
pub fn pyenv_python_path() -> Option<PathBuf> {
    // root will fail faster than version, so unwrap it first
    let root = pyenv_root()?;
    let version = pyenv_version()?;
    let mut path = root;
    path.push("versions");
    path.push(version);
    path.push("bin");
    path.push("python");
    Some(path)
}

/// Returns the system `python` on $PATH, excluding this program.
pub fn system_python() -> Option<PathBuf> {
    None
}

/// Get the path of `python` using `pyenv`, i.e., using [`pyenv_version()`].
/// If [`pyenv_version()`] returns None, then the system `python` is used, i.e. $PATH.
pub fn python_path() -> Option<PathBuf> {
    pyenv_python_path().or_else(system_python)
}
