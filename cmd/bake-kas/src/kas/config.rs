use std::path::{Path, PathBuf};

use pyo3::prelude::*;

use crate::kas::ProjectConfig;

pub(crate) mod project;

#[derive(Debug)]
#[pyclass]
pub struct KasProjectConfig {
    pub config_path: PathBuf,
    pub target: Option<String>,
    pub task: Option<String>,
    pub update: bool,

    // Internal representation of the config
    inner: Option<Py<PyAny>>,
}

#[pymethods]
impl KasProjectConfig {
    #[getter]
    pub fn get_config(&self) -> Option<&Py<PyAny>> {
        self.inner.as_ref()
    }
}

impl KasProjectConfig {
    pub fn new(
        config_path: PathBuf,
        target: Option<String>,
        task: Option<String>,
        update: bool,
    ) -> Self {
        Self {
            config_path,
            target,
            task,
            update,
            inner: None,
        }
    }
}

pub fn load_config(config_path: &Path) -> ProjectConfig {
    let cfg_file = std::fs::File::open(config_path).expect("Failed to open config file");
    serde_yaml::from_reader(cfg_file).expect("Failed to parse config file")
}
