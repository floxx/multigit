//! Tests of the shell scripts shipped in the release tarball and used by the
//! release workflow. They run in temporary directories only.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Lay out a fake unpacked tarball: install.sh, desktop file, icon, binary.
fn fake_tarball(dir: &Path) {
    let linux = root().join("packaging/linux");
    fs::copy(linux.join("install.sh"), dir.join("install.sh")).unwrap();
    fs::copy(
        linux.join("io.github.paxel.multigit.desktop"),
        dir.join("io.github.paxel.multigit.desktop"),
    )
    .unwrap();
    fs::copy(root().join("assets/icon_256.png"), dir.join("icon_256.png")).unwrap();
    fs::write(dir.join("multigit"), "#!/bin/sh\necho fake\n").unwrap();
}

#[test]
fn install_script_puts_binary_entry_and_icon_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    let unpacked = tmp.path().join("multigit-0.1.0");
    let home = tmp.path().join("home dir");
    fs::create_dir_all(&unpacked).unwrap();
    fs::create_dir_all(&home).unwrap();
    fake_tarball(&unpacked);

    let out = Command::new("sh")
        .arg(unpacked.join("install.sh"))
        .env("HOME", &home)
        .env_remove("XDG_DATA_HOME")
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let bin = home.join(".local/bin/multigit");
    assert_eq!(fs::read_to_string(&bin).unwrap(), "#!/bin/sh\necho fake\n");
    let entry =
        fs::read_to_string(home.join(".local/share/applications/io.github.paxel.multigit.desktop"))
            .unwrap();
    assert!(
        entry.contains(&format!("\nExec=\"{}\"\n", bin.display())),
        "{entry}"
    );
    assert!(entry.contains("X-Multigit-Managed=true"));
    assert_eq!(
        fs::read(home.join(".local/share/icons/hicolor/256x256/apps/multigit.png")).unwrap(),
        fs::read(root().join("assets/icon_256.png")).unwrap()
    );
}

fn release_notes(version: &str, changelog: &Path) -> std::process::Output {
    Command::new("sh")
        .arg(root().join("packaging/release-notes.sh"))
        .arg(version)
        .arg(changelog)
        .output()
        .unwrap()
}

#[test]
fn release_notes_extract_the_version_section() {
    let tmp = tempfile::tempdir().unwrap();
    let changelog = tmp.path().join("CHANGELOG.md");
    fs::write(
        &changelog,
        "# Changelog\n\nintro\n\n## [0.2.0]\n\n### Added\n- New thing.\n\n---\n\nHistory link.\n",
    )
    .unwrap();
    let out = release_notes("0.2.0", &changelog);
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "\n### Added\n- New thing.\n"
    );
    assert!(!release_notes("9.9.9", &changelog).status.success());
}

#[test]
fn current_changelog_has_notes_for_cargo_version() {
    let version = env!("CARGO_PKG_VERSION");
    let out = release_notes(version, &root().join("CHANGELOG.md"));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
