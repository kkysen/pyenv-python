use std::path::Path;
use std::env;
use std::process::exit;

#[cfg(unix)]
fn exec(path: &Path) {
    Err(exec::execvp(path, env::args_os()))
        .expect("failed to exec");
}

#[cfg(not(unix))]
fn exec(path: &Path) {
    use std::process::Command;
    let status = Command::new(path)
        .args(env::args_os().skip(1))
        .status()
        .expect("failed to run subprocess")
        .code().unwrap_or_default();
    exit(status);
}
