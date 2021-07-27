#![forbid(unsafe_code)]

use std::{env, fmt, io};
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::path::{Path, PathBuf};

use apply::Apply;
use is_executable::IsExecutable;
use same_file::Handle;
use thiserror::Error;

mod version;

/// A root `pyenv` directory.
#[derive(Debug)]
pub struct PyenvRoot {
    root: PathBuf,
}

impl Display for PyenvRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root.display())
    }
}

/// Why the pyenv root, i.e. what `$(pyenv root)` returns, could not be found.
///
/// See [`PyenvRoot::new`].
#[derive(Debug, Error)]
pub enum PyenvRootError {
    /// Either the environment variable `$PYENV_ROOT` does not exist,
    /// or the home directory does not exist
    /// (very unlikely, unless in an OS like wasm).
    #[error("the environment variable $PYENV_ROOT does not exist")]
    NoEnvVarOrHomeDir,
    /// The pyenv root is not a directory.
    #[error("pyenv root is not a directory: {root}")]
    NotADir { root: PathBuf },
    /// The pyenv root could not be accessed (usually it doesn't exist).
    #[error("could not find pyenv root: {root}")]
    IOError { root: PathBuf, source: io::Error },
}

impl PyenvRoot {
    /// Returns what `$(pyenv root)` would return.
    /// That is, `$PYENV_ROOT` or `$HOME/.pyenv` if they exist.
    ///
    /// See [`PyenvRootError`] for possible errors.
    pub fn new() -> Result<Self, PyenvRootError> {
        use PyenvRootError::*;
        let root = env::var_os("PYENV_ROOT")
            .map(|root| root.into())
            .or_else(|| dirs_next::home_dir().map(|home| home.join(".pyenv")))
            .ok_or(NoEnvVarOrHomeDir)?;
        match root.metadata() {
            Ok(metadata) => if metadata.is_dir() {
                Ok(Self { root })
            } else {
                Err(NotADir { root })
            },
            Err(source) => Err(IOError { root, source }),
        }
    }
}

/// Where the given [`PyenvVersion`] was found from.
#[derive(Debug, Copy, Clone)]
pub enum PyenvVersionFrom {
    Shell,
    Local,
    Global,
}

impl Display for PyenvVersionFrom {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Shell => "shell",
            Self::Local => "local",
            Self::Global => "global",
        };
        write!(f, "{}", name)
    }
}

/// A `pyenv` version, either a `python` version or a virtualenv name,
/// and where it was looked-up from.
#[derive(Debug)]
pub struct PyenvVersion {
    version: String,
    from: PyenvVersionFrom,
}

impl Display for PyenvVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "python {} from {}", self.version, self.from)
    }
}

impl PyenvRoot {
    /// Returns the current pyenv version as determined by
    /// [https://github.com/pyenv/pyenv#choosing-the-python-version].
    fn version(&self) -> Result<PyenvVersion, ()> {
        self
            .root
            .as_path()
            .apply(version::pyenv_version)
            .ok_or(())
    }
    
    fn python_path(&self, path_components: &[&str]) -> UncheckedPythonPath {
        let mut path = self.root.clone();
        for path_component in path_components {
            path.push(path_component)
        }
        path.push("python");
        UncheckedPythonPath::from_existing(path)
    }
    
    fn python_version_path(&self, version: &PyenvVersion) -> UncheckedPythonPath {
        self.python_path(&[
            "versions",
            version.version.as_str(),
            "bin",
        ])
    }
    
    fn python_shim_path(&self) -> UncheckedPythonPath {
        self.python_path(&[
            "shims",
        ])
    }
}

/// A path that might be a `python` executable.
#[derive(Debug)]
pub struct UncheckedPythonPath {
    path: PathBuf,
}

impl Display for UncheckedPythonPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "unchecked({})", self.path.display())
    }
}

/// The path to an existing (likely) `python` executable.
#[derive(Debug, Eq)]
pub struct PythonExecutable {
    /// The name to execute this python executable as (arg0).
    /// If [`None`], then the file name of the [`PythonExecutable::path`] is used instead.
    name: Option<OsString>,
    /// The path to the python executable.
    path: PathBuf,
    /// An open handle to the python executable for file equality.
    handle: Handle,
}

impl PartialEq for PythonExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Display for PythonExecutable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl PythonExecutable {
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    
    pub fn into_path(self) -> PathBuf {
        self.path
    }
    
    pub fn name(&self) -> &OsStr {
        self.name.as_deref()
            .unwrap_or_else(|| self.path.file_name()
                .expect("python executable should always have a file name (i.e. not root)")
            )
    }
    
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
    
    pub fn file(&self) -> &File {
        self.handle.as_file()
    }
}

#[derive(Error, Debug)]
pub enum PyenvPythonExecutableError {
    #[error("python not found: {0}")]
    NotFound(#[from] io::Error),
    #[error("python must be executable")]
    NotExecutable,
}

impl PythonExecutable {
    /// Check that the `python` path actually points to an executable
    /// (can't verify that it's actually Python, but it's at least an executable named `python`).
    ///
    /// See [`PyenvPythonExecutableError`] for possible errors.
    pub fn new(path: PathBuf) -> Result<Self, (PyenvPythonExecutableError, PathBuf)> {
        use PyenvPythonExecutableError::*;
        match (|path: &Path| {
            let handle = Handle::from_path(path)?;
            if !path.is_executable() {
                // Means path must have a file name.
                return Err(NotExecutable);
            }
            Ok(handle)
        })(path.as_path()) {
            Ok(handle) => Ok(Self {
                name: None,
                path,
                handle,
            }),
            Err(e) => Err((e, path))
        }
    }
    
    pub fn current() -> io::Result<Self> {
        let name = env::args_os()
            .next()
            .map(PathBuf::from)
            .and_then(|path| path.file_name().map(|name| name.to_os_string()));
        // TODO What to do if arg0 doesn't exist or is `/` (no file name)?
        // TODO Do I just silently default to the current executable's name?
        // TODO Though in all normal invocations (like as a symlink), this won't happen.
        let path = env::current_exe()?;
        let handle = Handle::from_path(path.as_path())?;
        Ok(Self {
            name,
            path,
            handle,
        })
    }
}

impl UncheckedPythonPath {
    pub fn from_existing(path: PathBuf) -> Self {
        Self { path }
    }
    
    pub fn check(self) -> Result<PythonExecutable, (PyenvPythonExecutableError, PathBuf)> {
        PythonExecutable::new(self.path)
    }
}

pub trait HasPython {
    fn python(&self) -> &PythonExecutable;
    
    fn into_python(self) -> PythonExecutable;
}

impl HasPython for PythonExecutable {
    fn python(&self) -> &PythonExecutable {
        self
    }
    
    fn into_python(self) -> PythonExecutable {
        self
    }
}

/// An existing `pyenv` `python` executable.
#[derive(Debug)]
pub struct Pyenv {
    root: PyenvRoot,
    version: PyenvVersion,
    python_path: PythonExecutable,
}

impl Display for Pyenv {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "pyenv {} at {}", self.version, self.python_path)
    }
}

impl HasPython for Pyenv {
    fn python(&self) -> &PythonExecutable {
        &self.python_path
    }
    
    fn into_python(self) -> PythonExecutable {
        self.python_path
    }
}

/// Possible errors in looking up the current `pyenv` `python` executable.
#[derive(Error, Debug)]
pub enum PyenvError {
    // The `pyenv` root couldn't be found, so the `pyenv` version or `python` executable couldn't.
    #[error("pyenv python can't be found because no root was found: {error}")]
    NoRoot {
        #[from] error: PyenvRootError,
    },
    /// The `pyenv` version can't be found anywhere,
    /// neither the shell, local, or global versions.
    ///
    /// See [https://github.com/pyenv/pyenv#choosing-the-python-version] for the algorithm.
    #[error("pyenv python can't be found because no version was found in shell, local, or global using root {root}")]
    NoVersion {
        root: PyenvRoot,
    },
    /// The `pyenv` `python` executable can't be found or is not an executable.
    #[error("pyenv {version} can't be found at {python_path}")]
    NoExecutable {
        #[source] error: PyenvPythonExecutableError,
        root: PyenvRoot,
        version: PyenvVersion,
        python_path: PathBuf,
    },
}

impl Pyenv {
    /// Looks up the current `pyenv` `python` executable and version,
    /// or returns which part could not be found.
    ///
    /// See [`PyenvError`] for possible errors.
    pub fn new() -> Result<Self, PyenvError> {
        use PyenvError::*;
        let root = PyenvRoot::new()?;
        // Have to use `match` here instead of `map_err()?` so rustc can see the moves are disjoint.
        let version = match root.version() {
            Err(()) => return Err(NoVersion { root }),
            Ok(version) => version,
        };
        let python_path = match root.python_version_path(&version).check() {
            Err((error, python_path)) => return Err(NoExecutable {
                error,
                root,
                version,
                python_path,
            }),
            Ok(path) => path,
        };
        Ok(Self {
            root,
            version,
            python_path,
        })
    }
}

/// A `python` executable, either a `pyenv` one or the system `python`
/// (i.e. whatever else is in `$PATH`).
#[derive(Debug)]
pub enum Python {
    Pyenv(Pyenv),
    System(PythonExecutable),
}

impl Display for Python {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pyenv(pyenv) =>
                write!(f, "{}", pyenv),
            Self::System(python_executable) =>
                write!(f, "system python on $PATH at {}", python_executable),
        }
    }
}

impl HasPython for Python {
    fn python(&self) -> &PythonExecutable {
        match self {
            Self::Pyenv(pyenv) => pyenv.python(),
            Self::System(python) => python.python(),
        }
    }
    
    fn into_python(self) -> PythonExecutable {
        match self {
            Self::Pyenv(pyenv) => pyenv.into_python(),
            Self::System(python) => python.into_python(),
        }
    }
}

#[derive(Error, Debug)]
pub enum SystemPythonError {
    #[error("failed to get current executable: {0}")]
    NoCurrentExe(#[from] io::Error),
    #[error("no $PATH to search")]
    NoPath,
    #[error("no other python in $PATH")]
    NotInPath,
}

impl Python {
    /// Lookup the current system `python`, i.e., whatever next is in `$PATH`
    /// that's not the current executable or a `pyenv` shim.
    ///
    /// Pass a [`PyenvRoot`] to avoid `pyenv` shims.
    /// If there is no `pyenv` root than [`None`] will work.
    ///
    /// Specifically, this returns the next `python` on `$PATH`,
    /// excluding the current executable and `$PYENV_ROOT/shims/python`.
    /// Otherwise, an infinite loop would be formed between ourselves and `$PYENV_ROOT/shims/python`.
    ///
    /// See [`SystemPythonError`] for possible errors.
    pub fn system(pyenv_root: Option<PyenvRoot>) -> Result<PythonExecutable, SystemPythonError> {
        use SystemPythonError::*;
        let current_python = PythonExecutable::current()?;
        let pyenv_shim_python = pyenv_root
            .map(|root| root.python_shim_path())
            .and_then(|path| path.check().ok());
        let path_var = env::var_os("PATH").ok_or(NoPath)?;
        env::split_paths(&path_var)
            .map(|mut path| {
                path.push(current_python.name());
                path
            })
            .map(UncheckedPythonPath::from_existing)
            .filter_map(|python| python.check().ok())
            .find(|python| python != &current_python && Some(python) != pyenv_shim_python.as_ref())
            .ok_or(NotInPath)
    }
}

#[derive(Error, Debug)]
#[error("couldn't find pyenv and system python: {pyenv}, {system}")]
pub struct PythonError {
    pub pyenv: PyenvError,
    pub system: SystemPythonError,
}

impl Python {
    /// Lookup a `python` executable.
    ///
    /// If a `pyenv` `python` cannot be found (see [`Pyenv::new`]),
    /// try finding the system `python` (see [`Python::system`]).
    /// If neither can be found, return the errors for both in [`PythonError`].
    pub fn new() -> Result<Self, PythonError> {
        match Pyenv::new() {
            Ok(pyenv) => Ok(Self::Pyenv(pyenv)),
            Err(pyenv_error) => match Self::system(None) {
                Ok(system_python) => Ok(Self::System(system_python)),
                Err(system_python_error) => Err(PythonError {
                    pyenv: pyenv_error,
                    system: system_python_error,
                }),
            },
        }
    }
}
