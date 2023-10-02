use bake_kas::kas::{KasContextBuilder, KasProjectConfig};

mod testlib;

#[test]
fn run_kas_checkout() {
    //// Given
    let work_dir = testlib::test_tempdir("run_kas_checkout");
    let project_config_path = testlib::test_asset("poky.yml");

    let kas_proj_cfg = KasProjectConfig::new(project_config_path, None, None, false);
    let kas_ctx = KasContextBuilder::new(work_dir.clone())
        .with_config(kas_proj_cfg)
        .update(true)
        .build();

    //// When
    let result = bake_kas::kas_checkout(kas_ctx);

    //// Then
    println!("result: {:?}", result);
    assert!(result.is_ok());

    // Assert tempdir contents
    assert!(work_dir.join("build/conf").is_dir());
    assert!(work_dir.join("poky/.git").is_dir());
    assert!(work_dir.join("poky/README.md").is_file());
}