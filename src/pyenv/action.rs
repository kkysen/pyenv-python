use crate::pyenv::command::Command;
use crate::pyenv::cache::CacheBehavior;
use crate::pyenv::action::Action::{Delegate, Intercept};
use crate::pyenv::cache::CacheBehavior::{Cache, Ignore, Invalidate};
use crate::pyenv::cache::CacheType::{Help, Versions};
use crate::pyenv::command::Command::{Local, Shell, Which, Shims, Exec, VersionOrigin, VersionName, VersionFile, Version, Prefix, Root, Global};
use std::path::Path;
use std::env;

pub enum Action {
    /// Intercept the command completely.
    Intercept(Command),
    
    /// Delegate the command.
    /// Caching depends on [`CacheBehavior`].
    Delegate(CacheBehavior),
}

impl Action {
    pub fn from_args() -> Action {
        let help = Delegate(Cache(Help));
        let mut args = env::args();
        if args.len() < 2 {
            return help;
        }
        let cmd = match args.nth(1) {
            Some(cmd) => cmd.as_str(),
            None => return help,
        };
        let arg = args.next();
        if arg.contains("--help") {
            return help;
        }
        match cmd {
            "root" => Intercept(Root),
            "prefix" => Intercept(Prefix { version: arg, virtualenv: false }),
            "virtualenv-prefix" => Intercept(Prefix { version: arg, virtualenv: true }),
            "version" => Intercept(Version),
            "version-file" => Intercept(VersionFile { dir: arg.map(Path::new) }),
            "version-name" => Intercept(VersionName),
            "version-origin" => Intercept(VersionOrigin),
            "exec" => Intercept(Exec),
            "shims" => Intercept(Shims { short: arg.contains("--short") }),
            "which" => Intercept(Which { command: arg }),
            
            "shell" if arg.is_none() => Intercept(Shell),
            "global" if arg.is_none() => Intercept(Global),
            "local" if arg.is_none() => Intercept(Local),
            
            _ => Delegate(match cmd {
                "shell"
                | "global"
                | "local"
                | "rehash"
                | "completions"
                | "version-file-read"
                | "version-file-write"
                => Ignore,
                
                "--version"
                | "commands"
                | "help"
                | "hooks"
                | "init"
                | "virtualenv-init"
                => Cache(Help),
                
                "versions"
                | "virtualenvs"
                | "whence"
                | "virtualenv" if arg.contains("--version")
                => Cache(Versions),
                
                "activate"
                | "deactivate"
                | "install"
                | "uninstall"
                => Invalidate(Versions),
                
                "update"
                => Invalidate(Help),
                
                _ => Ignore, // TODO should default be Invalidate(Versions)?
            }),
        }
    }
    
    pub fn run(&self) {
        match self {
            Intercept(cmd) => cmd.run(),
            Delegate(cache) => cache.run(),
        }
    }
}
