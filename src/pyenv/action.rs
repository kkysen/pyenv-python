use crate::pyenv::command::Command;
use crate::pyenv::cache::CacheBehavior;
use crate::pyenv::action::Action::{Delegate, Intercept};
use crate::pyenv::cache::CacheBehavior::{Cache, Ignore, Invalidate};
use crate::pyenv::cache::CacheType::{Help, Versions};
use crate::pyenv::command::Command::{Local, Shell, Which, Shims, Exec, VersionOrigin, VersionName, VersionFile, Version, Prefix, Root, Global, VersionFileRead, VersionFileWrite, VersionFileRead};
use std::path::{Path, PathBuf};
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
            None => return help,
            Some(cmd) => cmd.as_str(),
        };
        let arg1 = args.next();
        let arg2 = args.next();
        if arg1.map(|it| it.as_str()).contains(&"--help") {
            return help;
        }
        
        enum ControlFlow {
            Break,
            Continue,
        }
        
        use ControlFlow::{Break, Continue};
        
        try_to_command = || -> Result<Command, ControlFlow> Ok(match cmd {
            "root" => Root,
            "prefix" => Prefix { version: arg1, virtualenv: false },
            "virtualenv-prefix" => Prefix { version: arg1, virtualenv: true },
            "version" => Version,
            "version-file" => VersionFile { dir: arg1.map(Path::new) },
            "version-file-read" => VersionFileRead { path: arg1.ok_or(Break).map(PathBuf::new)? },
            "version-name" => VersionName,
            "version-origin" => VersionOrigin,
            "exec" => Exec,
            "shims" => Shims { short: arg1.contains("--short") },
            "which" => Which { command: arg1 },
            _ if arg1.is_none() => match cmd {
                "shell" => Shell,
                "global" => Global,
                "local" => Local,
                _ => return Err(Continue),
            },
            _ => return Err(Continue),
        });
        
        let to_cache_behavior = || -> CacheBehavior match cmd {
            "shell"
            | "global"
            | "local"
            | "rehash"
            | "completions"
            | "version-file-read"
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
            | "virtualenv" if arg1.contains("--version")
            => Cache(Versions),
            
            "activate"
            | "deactivate"
            | "install"
            | "uninstall"
            => Invalidate(Versions),
            
            "update"
            => Invalidate(Help),
            
            _ => Ignore, // TODO should default be Invalidate(Versions)?
        };
        
        match try_to_command() {
            Ok(cmd) => Intercept(cmd),
            Err(Break) => help,
            Err(Continue) => Delegate(to_cache_behavior()),
        }
    }
    
    pub fn run(self) {
        match self {
            Intercept(cmd) => cmd.run(),
            Delegate(cache) => cache.run(),
        }
    }
}
