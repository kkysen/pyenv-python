use std::path::PathBuf;
use crate::pyenv::cache::CacheType::{Help, Versions};
use std::{io, fs, env};
use std::fs::DirEntry;
use std::io::{Error, ErrorKind};
use crate::pyenv_path;
use std::process::{Command, Output, exit};
use crate::pyenv::cache::CacheBehavior::{Cache, Ignore, Invalidate};
use std::collections::HashMap;
use std::env::ArgsOs;
use std::collections::hash_map::Entry;

pub enum CacheType {
    Help,
    Versions,
}

impl CacheType {
    pub fn invalidates(&self) -> &[CacheType] {
        match self {
            Help => &[Help, Versions],
            Versions => &[Versions],
        }
    }
}

pub enum CacheBehavior {
    Cache(CacheType),
    Ignore,
    Invalidate(CacheType),
}

fn run_as_pyenv() -> Output {
    let path = pyenv_path().expect("pyenv not found");
    let output = Command::new(path)
        .args(env::args_os().skip(1))
        .output().expect("couldn't create subprocess for pyenv");
    output
}

impl CacheBehavior {
    pub fn run(self) {
        let mut cache = Cache::default(); // TODO load from cache file and lock file
        let output = match self {
            Cache(cache_type) => {
                let output = cache
                    .get(&cache_type)
                    .or_insert_with(run_as_pyenv);
                (*output).clone()
            }
            Ignore => run_as_pyenv(),
            Invalidate(cache_type) => {
                cache.invalidate(&cache_type);
                run_as_pyenv()
            }
        };
        // TODO save to cache file and release lock file
        let Output {status, stdout, stderr} = output;
        eprint_bytes(stderr);
        print_bytes(stdout);
        exit(status.code().unwrap_or_default());
    }
}

// skip storing environ, too, b/c too big
type CommandCache = HashMap<ArgsOs, Output>;
type CommandEntry<'a> = Entry<'a, ArgsOs, Output>;

struct Cache {
    help: CommandCache,
    versions: CommandCache,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            help: Default::default(),
            versions: Default::default(),
        }
    }
}

impl Cache {
    
    fn cache_for(&mut self, cache_type: &CacheType) -> &mut CommandCache {
        match cache_type {
            Help => &mut self.help,
            Versions => &mut self.versions,
        }
    }
    
    pub fn get(&mut self, cache_type: &CacheType) -> CommandEntry {
        self.cache_for(cache_type).entry(env::args_os())
    }
    
    fn invalidate_only(&mut self, cache_type: &CacheType) {
        self.cache_for(cache_type).clear();
    }
    
    pub fn invalidate(&mut self, cache_type: &CacheType) {
        for sub_cache_type in cache_type.invalidates() {
            self.invalidate_only(sub_cache_type);
        }
    }
}
