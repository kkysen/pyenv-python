use std::path::Path;
use crate::pyenv::command::Command::{Root, Prefix, Shell, Version, VersionFile, VersionName, VersionOrigin, Global, Local, Exec, Shims, Which};
use crate::pyenv;

pub enum Command {
    Root,
    Prefix { version: Option<String>, virtualenv: bool },
    Shell,
    Version,
    VersionFile { dir: Option<Path> },
    VersionName,
    VersionOrigin,
    Global,
    Local,
    Exec,
    Shims { short: bool },
    Which { command: Option<String> },
}

impl Command {
    pub fn run(&self) {
        match self {
            Root => {}
            Prefix { version, virtualenv } => {}
            Shell => {}
            Version => {}
            VersionFile { dir } => {}
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
