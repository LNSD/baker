use std::path::{Path, PathBuf};

/// Convenience function to get the path to the tests directory.
pub fn tests_dir() -> PathBuf {
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
pub fn test_tempdir(dir: &str) -> TempDir {
    let tmpdir_path = tests_output_dir().join("tmp").join(dir);

    // If the temporary directory doesn't exist, create it.
    let tmpdir = TempDir::from(tmpdir_path);
    tmpdir.init().expect("Failed to create temporary directory");
    tmpdir
}

/// Convenience function to get the path to a test asset.
pub fn test_asset(asset: &str) -> PathBuf {
    tests_dir().join("assets").join(asset)
}
