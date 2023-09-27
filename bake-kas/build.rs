use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
/// Gets an environment variable owned by cargo.
///
/// Environment variables set by cargo are expected to be valid UTF8.
fn cargo_env_var(var: &str) -> Option<String> {
    std::env::var_os(var).map(|os_string| os_string.to_str().unwrap().into())
}

/// Gets the path to the virtualenv, if one is active.
fn get_venv_path() -> Option<PathBuf> {
    match (env_var("VIRTUAL_ENV"), env_var("CONDA_PREFIX")) {
        (Some(dir), None) => Some(PathBuf::from(dir)),
        (None, Some(dir)) => Some(PathBuf::from(dir)),
        (Some(_), Some(_)) => {
            warn!(
                "Both VIRTUAL_ENV and CONDA_PREFIX are set. The build script will ignore both of \
                 these for locating the Python interpreter until you unset one of them."
            );
            None
        }
        (None, None) => None,
    }
}

/// Creates a virtualenv in the target directory.
fn create_venv(interpreter: &Path, target_dir: &Path) -> Result<PathBuf> {
    let venv_path = target_dir.join("venv");
    let status = Command::new(interpreter)
        .arg("-m")
        .arg("venv")
        .arg(&venv_path)
        .status()?;
    if !status.success() {
        return Err("venv creation failed".into());
    }

    Ok(venv_path)
}

/// Attempts to locate a python interpreter. Locations are checked in the order listed:
/// 1. If in a virtualenv, that environment's interpreter is used.
/// 2. `python`, if this is functional a Python 3.x interpreter
/// 3. `python3`, as above
fn find_interpreter() -> Result<PathBuf> {
    if let Some(venv_path) = get_venv_path() {
        match cargo_env_var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
            "windows" => Ok(venv_path.join("Scripts\\python")),
            _ => Ok(venv_path.join("bin/python")),
        }
    } else {
        println!("cargo:rerun-if-env-changed=PATH");
        ["python", "python3"]
            .iter()
            .find(|bin| {
                if let Ok(out) = Command::new(bin).arg("--version").output() {
                    // begin with `Python 3.X.X :: additional info`
                    out.stdout.starts_with(b"Python 3") || out.stderr.starts_with(b"Python 3")
                } else {
                    false
                }
            })
            .map(PathBuf::from)
            .ok_or_else(|| "no Python 3.x interpreter found".into())
    }
}

/// Return the path to the target directory.
///
/// The `OUT_DIR` env variable contains  the folder in which all output and intermediate artifacts
/// should be placed. This folder is inside the build directory for the package being built, and it
/// is unique for the package in question.
///
/// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
fn out_dir() -> PathBuf {
    PathBuf::from(cargo_env_var("OUT_DIR").unwrap())
}

fn main() {
    // Check if the build requirements are met.
    let python = match find_interpreter() {
        Ok(python) => python,
        Err(err) => panic!("Python interpreter not found: {}", err),
    };

    let venv = match get_venv_path() {
        Some(venv_path) => venv_path,
        None => match create_venv(&python, &out_dir()) {
            Ok(venv_path) => venv_path,
            Err(err) => panic!("venv creation failed: {}", err),
        },
    };

    // Upgrade virtualenv pip
    let status = Command::new(venv.join("bin/pip"))
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
    let status = Command::new(venv.join("bin/pip"))
        .arg("install")
        .arg("pyyaml==5.4.1")
        .arg("--no-build-isolation")
        .env("VIRTUAL_ENV", &venv)
        .status()
        .unwrap();
    if !status.success() {
        panic!("pip install pyyaml failed: {}", status);
    }

    // Install kas
    let status = Command::new(venv.join("bin/pip"))
        .arg("install")
        .arg("kas==4.0")
        .env("VIRTUAL_ENV", &venv)
        .status()
        .unwrap();
    if !status.success() {
        panic!("pip install kas failed: {}", status);
    }

    // The build script inherently does not need to re-run under any circumstance
    println!("cargo:rerun-if-changed=build.rs");
}
