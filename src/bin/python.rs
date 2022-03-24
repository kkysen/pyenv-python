#![forbid(unsafe_code)]

use std::{env, fmt, io};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use apply::Apply;
use is_executable::IsExecutable;
use print_bytes::println_bytes;
use thiserror::Error;

use pyenv_python::{HasPython, Python};

use crate::Argv0ProgramType::{Binary, PythonScript, Script};

#[derive(Eq, PartialEq, Debug)]
enum Argv0ProgramType {
    Binary,
    PythonScript,
    Script,
}

#[derive(Debug)]
struct Argv0Program {
    python_path: PathBuf,
    path: PathBuf,
    exe_type: Argv0ProgramType,
}

impl Argv0Program {
    pub fn python_path(&self) -> &Path {
        self.python_path.as_path()
    }
    
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

#[derive(Debug, Error)]
#[error("error running argv0 program {:?}: {}", .path, .message)]
struct Argv0ProgramError {
    path: PathBuf,
    message: &'static str,
    #[source]
    source: Option<io::Error>,
}

impl Argv0ProgramError {
    fn err(self) -> Result<(), Argv0ProgramError> {
        Err(self)
    }
}

struct PathBufError<'a> {
    path: &'a Path,
}

impl<'a> PathBufError<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
        }
    }
    
    fn path(&self) -> PathBuf {
        self.path.to_path_buf()
    }
    
    fn using_message(&self, message: &'static str) -> Argv0ProgramError {
        Argv0ProgramError {
            path: self.path(),
            message,
            source: None,
        }
    }
    
    fn using_source(&self, source: io::Error) -> Argv0ProgramError {
        Argv0ProgramError {
            path: self.path(),
            message: "",
            source: Some(source),
        }
    }
}

impl Argv0ProgramType {
    /// Detect the type of argv0 in `python`'s directory.
    /// `path` is already in `python`'s directory.
    /// First, check if it's an existing executable file.
    /// Then, check if it's a script by looking for a shebang #!.
    ///
    /// If it's not a script, then we assume it's a binary executable and we execute it as argv0.
    /// This includes the normal argv0 == "python" case.
    ///
    /// If it's a script, look for "python" in the shebang line.
    /// If it's a Python script, execute argv0 as "python" normally,
    /// with the script path inserted as argv1 so python can run it.
    /// It it's not a Python script, then just execute it as argv0,
    /// letting the OS run its shebang program.
    fn detect(path: &Path) -> Result<Self, Argv0ProgramError> {
        let error = PathBufError::new(path);
        let with_src = |msg| error.using_message(msg).err();
        let with_err = |e| error.using_source(e);
        
        if !path.exists() {
            with_src("does not exist")?;
        } else if !path.is_file() {
            with_src("not a file")?;
        } else if !path.is_executable() {
            with_src("not executable")?;
        }
        // checked the file already, so shouldn't have errors reading it,
        // so I'm not adding any context to the default anyhow::Error
        let file = File::open(path).map_err(with_err)?;
        let mut reader = BufReader::new(file);
        let mut shebang = [0_u8; 2];
        reader.read(&mut shebang).map_err(with_err)?;
        let is_script = &shebang == b"#!";
        let exe_type = if !is_script {
            Binary
        } else {
            // need to read shebang first before first line
            // b/c if there's no shebang and it's binary,
            // it might be UTF-8, so String decoding will fail
            let mut first_line = String::new();
            reader.read_line(&mut first_line).map_err(with_err)?;
            let is_python_script = ["python", "pip"]
                .iter()
                .any(|word| first_line.contains(word));
            if is_python_script {
                PythonScript
            } else {
                Script
            }
        };
        
        Ok(exe_type)
    }
}

impl Argv0Program {
    fn new(python_path: PathBuf) -> Result<Self, Argv0ProgramError> {
        let symlinked_path = || -> Option<PathBuf> {
            let argv0 = env::args_os().next()?;
            let argv0_name = Path::new(argv0.as_os_str()).file_name()?;
            let path_buf = python_path.parent()?.join(Path::new(argv0_name));
            Some(path_buf)
        };
        let path = symlinked_path()
            .unwrap_or_else(|| python_path.to_path_buf());
        let exe_type = Argv0ProgramType::detect(path.as_path())?;
        Ok(Self {
            python_path,
            path,
            exe_type,
        })
    }
    
    /// The path to use as argv0.
    fn argv0(&self) -> &Path {
        let Self {
            python_path,
            path,
            exe_type,
        } = self;
        match exe_type {
            Binary => path,
            PythonScript => python_path,
            Script => path,
        }.as_path()
    }
    
    /// The python script path, if it's valid.
    fn python_script(&self) -> Option<&Path> {
        Some(self.path.as_path())
            .filter(|_| self.exe_type == PythonScript)
    }
    
    fn to_command(&self) -> Command {
        let mut args = env::args_os();
        let mut cmd = Command::new(self.argv0());
        if let Some(_) = args.next() {}
        if let Some(script) = self.python_script() {
            cmd.arg(script.as_os_str());
        }
        cmd.args(args);
        cmd
    }
}

impl Display for Argv0Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let file_name = self.path().file_name().unwrap().apply(Path::new);
        let python_name = self.python_path().file_name().unwrap().apply(Path::new);
        // TODO only works in rustc 1.55, which isn't stable yet (as of 7/27/2021)
        // let [file_name, python_name] = [self.path(), self.python_path()]
        //     .map(|path| path.file_name().unwrap().apply(Path::new));
        let is_python = file_name == python_name;
        if is_python || self.exe_type == PythonScript {
            write!(f, "{}", python_name.display())?;
            if !is_python {
                write!(f, " ")?;
            }
        }
        if !is_python {
            write!(f, "{}", file_name.display())?;
        }
        Ok(())
    }
}

/// Most of the same extension methods as [`std::os::unix::process::CommandExt`],
/// except this is also implemented on `cfg(not(unix))`,
/// either with a fallback (`exec`) or not at all (`arg0`).
trait CommandExt2 {
    fn exec(&mut self) -> io::Error;
    
    fn arg0<S: AsRef<OsStr>>(&mut self, _arg: S) -> &mut Command;
}

#[cfg(not(unix))]
impl CommandExt2 for Command {
    fn exec(&mut self) -> io::Error {
        match self.status() {
            Ok(status) => status
                .code()
                .unwrap_or_default()
                .apply(std::process::exit),
            Err(e) => return e,
        }
    }
    
    fn arg0<S: AsRef<OsStr>>(&mut self, _arg: S) -> &mut Command {
        self
    }
}

#[cfg(unix)]
impl CommandExt2 for Command {
    fn exec(&mut self) -> io::Error {
        std::os::unix::process::CommandExt::exec(self)
    }
    
    fn arg0<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        std::os::unix::process::CommandExt::arg0(self, arg)
    }
}

/// Run the current `python` (as determined by `pyenv`) with the given args.
/// If --path is the only arg, print `python`'s path.
/// If --prefix is the only arg, print `python`'s directory,
/// the same as `python -c 'import sys; print(sys.prefix)'`.
/// These are the only differences from actual `python`,
/// and they don't clash with any of `python`'s actual options.
fn main() -> anyhow::Result<()> {
    let python = Python::new().context("python not found")?;
    let program = python
        .python()
        .path()
        .to_path_buf()
        .apply(Argv0Program::new)?;
    let parent_level: Option<usize> = match env::args()
        .nth(1)
        .unwrap_or_default()
        .as_str() {
        "--path" => Some(0),
        "--dir" => Some(1),
        "--prefix" => Some(2),
        "--which" => {
            println!("`{}` using {}", program, python);
            return Ok(());
        }
        _ => None,
    };
    match parent_level {
        None => program
            .to_command()
            .exec()
            .apply(Err)
            .context("failed to run python subprocess")?,
        Some(level) => {
            let mut dir = program.path();
            for current_level in 0..level {
                dir = dir.parent()
                    .with_context(|| format!(
                        "python --path doesn't have {} parent directories",
                        current_level,
                    ))?;
            }
            println_bytes(dir);
        }
    }
    Ok(())
}
