use manganis_cli_support::{AssetManifestExt, ManganisSupportGuard};
use manganis_common::{AssetManifest, AssetType, Config};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{ChildStdout, Stdio};

use cargo_metadata::Message;
use std::process::Command;

// return the location of the executable generated by cargo
fn get_executable_location(cargo_output: std::io::BufReader<ChildStdout>) -> PathBuf {
    let executable = cargo_metadata::Message::parse_stream(cargo_output).find_map(|x| {
        if let Ok(Message::CompilerArtifact(artifact)) = x {
            artifact.executable
        } else {
            None
        }
    });
    let executable = executable.expect("Failed to find the output binary path. This may happen if you build a library instead of an application");

    executable.into_std_path_buf()
}

#[test]
fn collects_assets() {
    // This is the location where the assets will be served from
    let assets_serve_location = "/assets";

    // First set any settings you need for the build
    Config::default()
        .with_assets_serve_location(assets_serve_location)
        .save();

    // Next, tell manganis that you support assets
    let _guard = ManganisSupportGuard::default();

    // Find the test package directory which is up one directory from this package
    let mut test_package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    test_package_dir.push("test-package");

    println!("running the CLI from {test_package_dir:?}");

    // Then build your application
    let mut command = Command::new("cargo")
        .args([
            "build",
            "--message-format=json-render-diagnostics",
            "--release",
        ])
        .current_dir(test_package_dir)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let path = get_executable_location(reader);

    // Then collect the assets
    let assets = AssetManifest::load(&path);

    let all_assets = assets.assets();

    println!("{:#?}", all_assets);

    let locations = all_assets
        .iter()
        .filter_map(|a| match a {
            AssetType::File(f) => Some(f.location()),
            _ => None,
        })
        .collect::<HashSet<_>>();

    assert_eq!(locations.len(), 16);
}
