use std::path::{PathBuf, Path};
use std::ffi::OsString;

mod command;
mod cache;
pub mod action;

pub fn root() -> Option<PathBuf> {
    todo!()
}

pub fn prefix(version: String, virtualenv: bool) -> Option<PathBuf> {
    todo!()
}

pub mod version {
    use std::path::{PathBuf, Path};
    use std::env::current_dir;
    use crate::pyenv::version::Origin::System;
    use std::{io, fs};
    use std::fs::File;
    use std::io::{BufReader, ErrorKind, BufRead};
    
    pub struct VersionFile {
        pub path: PathBuf,
    }
    
    impl VersionFile {
        pub fn read(&self) -> io::Result<String> {
            let file = File::open(&self.path)?;
            let reader = BufReader::new(file);
            let version = reader.lines().next().ok_or(ErrorKind::NotFound)??;
            Ok(version)
        }
        
        pub fn write(&self, name: String) -> io::Result<()> {
            fs::write(&self.path, name)?;
            Ok(())
        }
    }
    
    pub enum Origin {
        Shell,
        File(VersionFile),
        System,
    }
    
    pub struct Version {
        pub name: String,
        pub origin: Origin,
    }
    
    impl Default for Version {
        fn default() -> Self {
            Self {
                name: "system".into(),
                origin: System,
            }
        }
    }
    
    pub fn get_in_dir(dir: &Path, skip_shell: bool) -> Version {
        todo!()
    }
    
    pub fn get(skip_shell: bool) -> Version {
        todo!()
    }
    
    pub fn shell() -> Option<Version> {
        todo!()
    }
    
    pub fn local() -> Option<Version> {
        todo!()
    }
    
    pub fn global() -> Option<Version> {
        todo!()
    }
    
}

pub fn shims_dir() -> Option<PathBuf> {
    todo!()
}

pub fn shims(short: bool) -> Vec<PathBuf> {
    todo!()
}

pub fn which(command: &Path) -> Option<PathBuf> {
    todo!()
}
