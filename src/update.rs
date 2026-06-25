use anyhow::{Result, bail};

pub fn target_triple() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        _ => "unsupported",
    }
}

pub fn asset_name(version: &str, target: &str) -> String {
    format!("ccs-{version}-{target}.tar.gz")
}

pub fn run_update() -> Result<()> {
    let target = target_triple();
    if target == "unsupported" {
        bail!(
            "self-update is not available for {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }

    let status = self_update::backends::github::Update::configure()
        .repo_owner("reedchan7")
        .repo_name("ccs")
        .bin_name("ccs")
        .target(target)
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    println!("Updated to {}", status.version());
    Ok(())
}
