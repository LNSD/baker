use bake_kas::kas::KasContextBuilder;

mod testlib;

#[test]
fn run_kas_checkout() {
    //// Given
    let work_dir = testlib::test_tempdir("run_kas_checkout");
    let config = testlib::test_asset("poky.yml");

    let kas_ctx = KasContextBuilder::new(work_dir.clone())
        .update(true)
        .build();
    let kas_cfg = config;

    //// When
    let result = bake_kas::kas_checkout(kas_ctx, kas_cfg);

    //// Then
    assert!(result.is_ok());

    // Assert tempdir contents
    assert!(work_dir.join("build/conf").is_dir());
    assert!(work_dir.join("poky/.git").is_dir());
    assert!(work_dir.join("poky/README.md").is_file());
}
