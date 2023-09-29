use std::path::{Path, PathBuf};

use bake_kas::kas::KasContextBuilder;

/// Convenience function to get the path to the tests directory.
fn tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// Convenience function to get the path to the tests output directory.
fn tests_output_dir() -> PathBuf {
    PathBuf::from(env!("OUT_DIR")).join("tests")
}

pub struct TempDir(PathBuf);

impl TempDir {
    /// Create the temporary directory if it doesn't exist.
    pub fn init(&self) -> std::io::Result<()> {
        if !self.0.exists() {
            return std::fs::create_dir_all(&self.0);
        }

        Ok(())
    }
}

impl From<PathBuf> for TempDir {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::ops::Deref for TempDir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TempDir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if self.0.exists() {
            std::fs::remove_dir_all(&self.0).expect("Failed to remove temporary directory");
        }
    }
}

/// Convenience function to get the path to the tests output temporary directory.
fn test_tempdir(dir: &str) -> TempDir {
    let tmpdir_path = tests_output_dir().join("tmp").join(dir);

    // If the temporary directory doesn't exist, create it.
    let tmpdir = TempDir::from(tmpdir_path);
    tmpdir.init().expect("Failed to create temporary directory");
    tmpdir
}

/// Convenience function to get the path to a test asset.
fn test_asset(asset: &str) -> PathBuf {
    tests_dir().join("assets").join(asset)
}

#[test]
fn run_kas_checkout() {
    //// Given
    let work_dir = test_tempdir("run_kas_checkout");
    let config = test_asset("poky.yml");

    let kas_ctx = KasContextBuilder::new(work_dir.clone())
        .update(true)
        .build();
    let kas_cfg = config;

    //// When
    let result = bake_kas::kas_checkout(kas_ctx, kas_cfg);

    //// Then
    assert!(result.is_ok());

    assert!(work_dir.join("build/conf").is_dir());
    assert!(work_dir.join("poky/.git").is_dir());
    assert!(work_dir.join("poky/README.md").is_file());
}
