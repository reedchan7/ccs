use ccs::update::asset_name;

#[test]
fn builds_linux_amd64_asset_name() {
    assert_eq!(
        asset_name("v1.2.3", "x86_64-unknown-linux-gnu"),
        "ccs-v1.2.3-x86_64-unknown-linux-gnu.tar.gz"
    );
}

#[test]
fn builds_macos_arm64_asset_name() {
    assert_eq!(
        asset_name("v1.2.3", "aarch64-apple-darwin"),
        "ccs-v1.2.3-aarch64-apple-darwin.tar.gz"
    );
}
