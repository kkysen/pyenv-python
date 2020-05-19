use std::path::{Path, PathBuf};
use crate::pyenv::command::Command::{Root, Prefix, Shell, Version, VersionFile, VersionFileRead, VersionName, VersionOrigin, Global, Local, Exec, Shims, Which};
use crate::pyenv;

pub enum Command {
    Root,
    Prefix { version: Option<String>, virtualenv: bool },
    Shell,
    Version,
    VersionFile { dir: Option<PathBuf> },
    VersionFileRead { path: PathBuf },
    VersionName,
    VersionOrigin,
    Global,
    Local,
    Exec,
    Shims { short: bool },
    Which { command: Option<String> },
}

impl Command {
    pub fn run(self) {
        match self {
            Root => {}
            Prefix { version, virtualenv } => {}
            Shell => {}
            Version => {}
            VersionFile { dir } => {}
            VersionFileRead { path } => {
                let version_file = pyenv::version::VersionFile {path};
                version_file.read();
            }
            VersionName => {}
            VersionOrigin => {}
            Global => {}
            Local => {}
            Exec => {}
            Shims { short } => {}
            Which { command } => {}
        }
    }
}
