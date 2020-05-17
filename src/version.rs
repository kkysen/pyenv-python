use std::ffi::OsString;
use std::{env, io, fs};
use std::path::{Path, PathBuf};

trait FlipResult<T, E> {
    fn flip(self) -> Result<E, T>;
}

impl<T, E> FlipResult<T, E> for Result<T, E> {
    fn flip(self) -> Result<E, T> {
        match self {
            Ok(t) => Err(t),
            Err(e) => Ok(e),
        }
    }
}

fn from_env() -> Result<(), OsString> {
    env::var_os("PYENV_VERSION")
        .ok_or(())
        .flip()
}

fn read_python_version_file(path: &Path) -> io::Result<OsString> {
    let bytes = fs::read(path)?;
    let version = OsString::from_vec(bytes);
    Ok(version)
}

fn from_local_python_version_file() -> Result<io::Error, OsString> {
    // let mut path = env::current_dir().flip()?;
    Err("".into())
}

fn global_python_version_file_path(root: &Path) -> PathBuf {
    let mut path = root.to_path_buf();
    path.push("version");
    path
}

fn from_global_python_version_file(root: &Path) -> Result<io::Error, OsString> {
    let path = global_python_version_file_path(root);
    read_python_version_file(path.as_path()).flip()
}

// use inverted Result<>s here to short circuit on success instead of failure
fn as_result(root: &Path) -> Result<(), OsString> {
    from_env()?;
    from_local_python_version_file()?;
    from_global_python_version_file(root)?;
    Ok(())
}

pub fn pyenv_version(root: &Path) -> Option<OsString> {
    as_result(root).err()
}
