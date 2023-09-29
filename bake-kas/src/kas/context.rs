use std::collections::BTreeMap;
use std::path::PathBuf;

use pyo3::prelude::*;

#[derive(Debug)]
#[pyclass]
pub struct KasContext {
    pub kas_work_dir: PathBuf,
    pub build_dir: PathBuf,
    pub kas_repo_ref_dir: Option<PathBuf>,
    pub force_checkout: Option<bool>,
    pub update: Option<bool>,
    pub environment: BTreeMap<String, String>,

    #[pyo3(get, set)]
    pub config: Option<Py<PyAny>>,

    // Required by: https://github.com/siemens/kas/blob/4edb347c920467f031f9ec2ddeda23db641a38bd/kas/libcmds.py#L329
    #[pyo3(get, set)]
    pub missing_repo_names: Option<Py<PyAny>>,
    // Required by: https://github.com/siemens/kas/blob/4edb347c920467f031f9ec2ddeda23db641a38bd/kas/libcmds.py#L330
    #[pyo3(get, set)]
    pub missing_repo_names_old: Option<Py<PyAny>>,
}

#[pymethods]
impl KasContext {
    /// The path to the build directory.
    #[getter]
    fn build_dir(&self) -> String {
        self.build_dir.to_str().unwrap().to_string()
    }

    /// The path to the kas work directory.
    #[getter]
    fn kas_work_dir(&self) -> String {
        self.kas_work_dir.to_str().unwrap().to_string()
    }

    /// The reference directory for the repo.
    #[getter]
    fn kas_repo_ref_dir(&self) -> Option<String> {
        self.kas_repo_ref_dir
            .as_ref()
            .map(|p| p.to_str().unwrap().to_string())
    }

    #[getter]
    fn force_checkout(&self) -> Option<bool> {
        self.force_checkout
    }

    #[getter]
    fn update(&self) -> Option<bool> {
        self.update
    }

    #[getter]
    fn environment(&self) -> BTreeMap<String, String> {
        self.environment.clone()
    }

    #[getter]
    fn environ(&self) -> BTreeMap<String, String> {
        self.environment()
    }
}

pub struct KasContextBuilder {
    kas_work_dir: PathBuf,
    build_dir: PathBuf,
    kas_repo_ref_dir: Option<PathBuf>,
    force_checkout: Option<bool>,
    update: Option<bool>,
    environment: BTreeMap<String, String>,
}

impl KasContextBuilder {
    pub fn new(work_dir: PathBuf) -> Self {
        let work_dir = if work_dir.is_absolute() {
            work_dir
        } else {
            work_dir
                .canonicalize()
                .expect("work_dir must be a valid path")
        };

        Self {
            kas_work_dir: work_dir.clone(),
            build_dir: work_dir.join("build"),
            kas_repo_ref_dir: None,
            force_checkout: None,
            update: None,
            environment: BTreeMap::new(),
        }
    }

    pub fn with_build_dir(mut self, build_dir: PathBuf) -> Self {
        let build_dir = if build_dir.is_absolute() {
            build_dir
        } else {
            build_dir
                .canonicalize()
                .expect("build_dir must be a valid path")
        };

        self.build_dir = build_dir;
        self
    }

    pub fn with_repo_ref_dir(mut self, repo_ref_dir: PathBuf) -> Self {
        let repo_ref_dir = if repo_ref_dir.is_absolute() {
            repo_ref_dir
        } else {
            repo_ref_dir
                .canonicalize()
                .expect("repo_ref_dir must be a valid path")
        };

        self.kas_repo_ref_dir = Some(repo_ref_dir);
        self
    }

    pub fn force_checkout(mut self, force_checkout: bool) -> Self {
        self.force_checkout = Some(force_checkout);
        self
    }

    pub fn update(mut self, update: bool) -> Self {
        self.update = Some(update);
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> KasContext {
        KasContext {
            kas_work_dir: self.kas_work_dir,
            build_dir: self.build_dir,
            kas_repo_ref_dir: self.kas_repo_ref_dir,
            force_checkout: self.force_checkout,
            update: self.update,
            environment: self.environment,

            config: None,
            missing_repo_names: None,
            missing_repo_names_old: None,
        }
    }
}
