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
    
    pub enum Origin {
        Shell,
        File(PathBuf),
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

pub fn which(command: &Path) -> Option<PathBuf> {}
