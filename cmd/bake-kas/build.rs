use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

// Show warning. If needed, please extend this macro to support arguments.
macro_rules! warn {
    ($msg: literal) => {
        println!(concat!("cargo:warning=", $msg));
    };
}

/// Gets an external environment variable, and registers the build script to rerun if
/// the variable changes.
fn env_var(var: &str) -> Option<OsString> {
    println!("cargo:rerun-if-env-changed={}", var);
    std::env::var_os(var)
}

/// Gets the path to the virtualenv, if one is active.
fn get_venv_path() -> Option<PathBuf> {
    env_var("VIRTUAL_ENV").map(PathBuf::from)
}

fn main() {
    // Check if the build requirements are met.
    let venv = match get_venv_path() {
        Some(path) => path,
        None => panic!("No VIRTUAL_ENV found"),
    };

    // Install venv dependencies
    let pip = venv.join("bin/pip");

    // Upgrade virtualenv's pip
    let status = Command::new(&pip)
        .arg("install")
        .arg("--upgrade")
        .arg("pip")
        .env("VIRTUAL_ENV", &venv)
        .status()
        .unwrap();
    if !status.success() {
        warn!("pip upgrade failed");
    }

    // Install pyyaml manually to avoid build isolation issue
    let status = Command::new(&pip)
        .arg("install")
        .arg("pyyaml==5.4.1")
        .arg("--no-build-isolation")
        .env("VIRTUAL_ENV", &venv)
        .status()
        .unwrap();
    if !status.success() {
        panic!("pip install 'pyyaml' failed: {}", status);
    }

    // Install kas
    let status = Command::new(&pip)
        .arg("install")
        .arg("kas==4.0")
        .env("VIRTUAL_ENV", &venv)
        .status()
        .unwrap();
    if !status.success() {
        panic!("pip install 'kas' failed: {}", status);
    }
}
