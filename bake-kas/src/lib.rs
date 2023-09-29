use std::path::PathBuf;

use pyo3::prelude::PyModule;
use pyo3::{Py, PyAny, Python};

use crate::kas::KasContext;

pub mod kas;

mod scripts {
    pub const KAS_VERSION: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/scripts/kas_version.py"
    ));

    pub const KAS_EXEC: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/kas_exec.py"));

    pub const KAS_CHECKOUT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/scripts/kas_checkout.py"
    ));
}

pub fn kas_version() -> String {
    Python::with_gil(|py| -> String {
        let version_fn: Py<PyAny> = PyModule::from_code(py, scripts::KAS_VERSION, "", "")
            .unwrap()
            .getattr("version")
            .unwrap()
            .into();
        let version = version_fn.call0(py);

        version.expect("kas version not found").extract(py).unwrap()
    })
}

pub fn kas_exec(argv: impl Into<Vec<String>>) -> Result<(), String> {
    let argv = argv.into();
    Python::with_gil(|py| -> Result<(), String> {
        let exec_fn: Py<PyAny> = PyModule::from_code(py, scripts::KAS_EXEC, "", "")
            .unwrap()
            .getattr("kas_exec")
            .unwrap()
            .into();
        let result = exec_fn.call1(py, (argv,));

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    })
}

pub fn kas_checkout(ctx: KasContext, config: PathBuf) -> Result<(), String> {
    Python::with_gil(|py| -> Result<(), String> {
        let checkout_fn: Py<PyAny> = PyModule::from_code(py, scripts::KAS_CHECKOUT, "", "")
            .unwrap()
            .getattr("kas_checkout")
            .unwrap()
            .into();
        let result = checkout_fn.call1(py, (ctx, config));

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    })
}
