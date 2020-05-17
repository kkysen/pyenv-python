use std::{env, io};
use std::path::{Path, PathBuf};
use std::io::{ErrorKind, BufReader, BufRead};
use std::fs::File;

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

fn read_python_version_file(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let version = reader.lines().next().ok_or(ErrorKind::NotFound)??;
    Ok(version)
}

fn from_local_python_version_file_given_cwd(cwd: &Path) -> Result<io::Error, String> {
    for dir in cwd.ancestors() {
        let path = dir.join(".python-version");
        read_python_version_file(path.as_path()).flip()?;
    }
    Ok(ErrorKind::NotFound.into())
}

fn from_local_python_version_file() -> io::Result<String> {
    let cwd = env::current_dir()?;
    let version = from_local_python_version_file_given_cwd(cwd.as_path()).flip()?;
    Ok(version)
}

fn global_python_version_file_path(root: &Path) -> PathBuf {
    let mut path = root.to_path_buf();
    path.push("version");
    path
}

fn from_global_python_version_file(root: &Path) -> io::Result<String> {
    let path = global_python_version_file_path(root);
    read_python_version_file(path.as_path())
}

// use inverted Result<>s here to short circuit on success instead of failure
fn as_result(root: &Path) -> Result<(), String> {
    env::var("PYENV_VERSION").flip()?;
    from_local_python_version_file().flip()?;
    from_global_python_version_file(root).flip()?;
    Ok(())
}

pub fn pyenv_version(root: &Path) -> Option<String> {
    as_result(root).err()
}
