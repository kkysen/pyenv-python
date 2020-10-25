#![forbid(unsafe_code)]

use pyenv_python::python_path;
use std::process::{exit, Command};
use std::os::unix::process::CommandExt;
use std::{env, io};
use anyhow::Context;
use std::path::{Path, PathBuf};
use thiserror::Error;
use is_executable::IsExecutable;
use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use print_bytes::println_bytes;
use crate::Argv0ProgramType::{Binary, PythonScript, Script};

#[derive(Eq, PartialEq)]
enum Argv0ProgramType {
    Binary,
    PythonScript,
    Script,
}

struct Argv0Program {
    python_path: PathBuf,
    path: PathBuf,
    exe_type: Argv0ProgramType,
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
    
    fn with_message(&self, message: &'static str) -> Argv0ProgramError {
        Argv0ProgramError {
            path: self.path(),
            message,
            source: None,
        }
    }
    
    fn with_source(&self, source: io::Error) -> Argv0ProgramError {
        Argv0ProgramError {
            path: self.path(),
            message: "",
            source: Some(source),
        }
    }
}

impl Argv0ProgramType {
    /// Detect the type of argv0 in `python`'s directory.
    /// [`path`] is already in `python`'s directory.
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
        let with_src = |msg| error.with_message(msg).err();
        let with_err = |e| error.with_source(e);
        
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
        let mut shebang = [0 as u8; 2];
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
            let is_python_script = ["python", "pip"].iter().any(|word| first_line.contains(word));
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
            let argv0 = env::args_os().nth(0)?;
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
        let Self { python_path, path, exe_type } = self;
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
        let mut cmd = Command::new(self.argv0());
        if cfg!(unix) {
            if let Some(arg0) = env::args_os().nth(0) {
                cmd.arg0(arg0);
            }
        }
        if let Some(script) = self.python_script() {
            cmd.arg(script.as_os_str());
        }
        cmd.args(env::args_os().skip(1));
        cmd
    }
    
    /// Run, allowing for architectural abstraction over Command.
    /// Either [`exec::Command`] should be used on unix,
    /// or [`std::process::Command`] as a backup.
    fn run<F>(&self, run: F) -> anyhow::Result<()>
        where F: FnOnce(Command) -> anyhow::Result<()> {
        run(self.to_command())
    }
}

#[cfg(unix)]
fn exec(mut cmd: Command) -> anyhow::Result<()> {
    let error = cmd.exec();
    Err(error)?;
    exit(1);
}

#[cfg(not(unix))]
fn exec_cmd(mut cmd: std::process::Command) -> anyhow::Result<()> {
    let status = cmd
        .status().context("failed to run python subprocess")?
        .code().unwrap_or_default();
    exit(status);
}

/// Run the current `python` (as determined by `pyenv`) with the given args.
/// Uses [`exec::Command`] on unix and [`std::process::Command`] elsewhere.
/// If --path is the only arg, print `python`'s path.
/// If --prefix is the only arg, print `python`'s directory,
/// the same as `python -c 'import sys; print(sys.prefix)'`.
/// These are the only differences from actual `python`,
/// and they don't clash with any of `python`'s actual options.
fn main() -> anyhow::Result<()> {
    let python_path_buf = python_path().context("python not found")?;
    let python_path = python_path_buf.as_path();
    let parent_level: Option<usize> = match env::args()
        .nth(1)
        .unwrap_or_default()
        .as_str() {
        "--path" => Some(0),
        "--dir" => Some(1),
        "--prefix" => Some(2),
        _ => None,
    };
    match parent_level {
        None => Argv0Program::new(python_path_buf)?.run(exec)?,
        Some(level) => {
            let mut dir = python_path;
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
