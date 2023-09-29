use std::path::PathBuf;

use bake_kas::kas::KasContextBuilder;

/// Convenience function to get the path to the tests directory.
fn tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// Convenience function to get the path to the tests temporary directory.
fn test_tempdir(dir: &str) -> PathBuf {
    let tmp = tests_dir().join("tmp").join(dir);

    // If the directory doesn't exist, create it.
    if !tmp.exists() {
        std::fs::create_dir_all(&tmp).expect("Failed to create temporary directory");
    }

    tmp
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

    let kas_ctx = KasContextBuilder::new(work_dir).update(true).build();
    let kas_cfg = config;

    //// When
    let result = bake_kas::kas_checkout(kas_ctx, kas_cfg);

    //// Then
    println!("{:?}", result);
    assert!(result.is_ok());
}
