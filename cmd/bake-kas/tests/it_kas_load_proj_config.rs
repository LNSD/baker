use assert_matches::assert_matches;

use bake_kas::kas::ProjectConfig;

#[test]
fn load_proj_config() {
    //// Given
    const TEST_PROJ_CONFIG: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/assets/poky.yml"
    ));

    //// When
    let result = serde_yaml::from_str::<ProjectConfig>(TEST_PROJ_CONFIG);

    //// Then
    assert!(result.is_ok());

    // Assert config values
    let config = result.unwrap();

    assert_eq!(config.header.version, "14");
    assert_eq!(config.target, ["zlib-native"]);

    // Assert repos
    assert_eq!(config.repos.len(), 1);
    assert_matches!(config.repos.get("poky"), Some(Some(repo)) => {
        assert_eq!(repo.url.as_deref(), Some("https://git.yoctoproject.org/poky.git"));
        assert_eq!(repo.commit.as_deref(), Some("387ab5f18b17c3af3e9e30dc58584641a70f359f"));

        // Assert layers
        assert_eq!(repo.layers.len(), 2);
        assert_matches!(repo.layers.get("meta"), Some(None));
        assert_matches!(repo.layers.get("meta-poky"), Some(None));
    });
}
