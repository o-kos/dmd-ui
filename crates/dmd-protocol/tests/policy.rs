use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned()
}

fn validate_sources(metadata: &Value) -> Result<(), String> {
    let root = Path::new(
        metadata["workspace_root"]
            .as_str()
            .ok_or("missing workspace root")?,
    )
    .canonicalize()
    .map_err(|e| e.to_string())?;
    for package in metadata["packages"].as_array().ok_or("missing packages")? {
        let name = package["name"].as_str().ok_or("missing package name")?;
        match package["source"].as_str() {
            Some(source) if source.starts_with("registry+") => (),
            Some(_) => {
                return Err(format!(
                    "{name}: only registry or workspace path sources are permitted"
                ));
            }
            None => {
                let manifest = Path::new(
                    package["manifest_path"]
                        .as_str()
                        .ok_or("missing manifest path")?,
                )
                .canonicalize()
                .map_err(|e| e.to_string())?;
                if !manifest.starts_with(&root) {
                    return Err(format!("{name}: dependency path escapes the workspace"));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn dependency_sources_stay_in_the_workspace_or_registry() {
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace())
        .args(["metadata", "--locked", "--format-version", "1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
    validate_sources(&metadata).unwrap();
}

#[test]
fn dependency_policy_rejects_external_paths_and_git() {
    let root = workspace();
    let metadata = |source: Value, manifest: PathBuf| {
        serde_json::json!({
            "workspace_root": root.join("crates/dmd-protocol"),
            "packages": [{"name": "fixture", "source": source, "manifest_path": manifest}],
        })
    };
    let external_path = metadata(Value::Null, root.join("Cargo.toml"));
    assert!(
        validate_sources(&external_path)
            .unwrap_err()
            .contains("escapes")
    );
    let git_source = metadata(
        Value::String(["git+", "file:", "fixture"].concat()),
        root.join("Cargo.toml"),
    );
    assert!(
        validate_sources(&git_source)
            .unwrap_err()
            .contains("sources")
    );
    let inside = metadata(Value::Null, root.join("crates/dmd-protocol/Cargo.toml"));
    assert!(validate_sources(&inside).is_ok());
}

#[test]
fn policy_scripts_pass_their_tests() {
    let root = workspace();
    let output_dir = root.join("target/policy-tests");
    std::fs::create_dir_all(&output_dir).unwrap();
    for (source, name) in [
        ("scripts/content_policy.rs", "content-policy"),
        (".github/scripts/ci_status.rs", "ci-status"),
    ] {
        let binary = output_dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        let compile = Command::new("rustc")
            .current_dir(&root)
            .args(["--edition=2024", "-D", "warnings", "--test", source, "-o"])
            .arg(&binary)
            .status()
            .unwrap();
        assert!(compile.success(), "cannot compile {source}");
        assert!(
            Command::new(&binary).status().unwrap().success(),
            "{source} tests failed"
        );
    }
}

#[cfg(unix)]
fn without_git_environment(mut command: Command) -> Command {
    for (name, _) in std::env::vars_os() {
        if name.as_encoded_bytes().starts_with(b"GIT_") {
            command.env_remove(name);
        }
    }
    command
}

#[cfg(unix)]
#[test]
fn push_hook_ignores_inherited_git_environment() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let other = std::env::temp_dir().join(format!(
        "dmd-hook-other-repository-{}-{nonce}",
        std::process::id()
    ));
    let init = without_git_environment(Command::new("git"))
        .args(["init", "--bare"])
        .arg(&other)
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    for contaminated in [false, true] {
        let mut test = without_git_environment(Command::new(std::env::current_exe().unwrap()));
        test.args([
            "--exact",
            "push_hook_enforces_content_and_local_policy",
            "--nocapture",
        ]);
        if contaminated {
            test.env("GIT_DIR", &other);
            for name in [
                "GIT_WORK_TREE",
                "GIT_INDEX_FILE",
                "GIT_OBJECT_DIRECTORY",
                "GIT_ALTERNATE_OBJECT_DIRECTORIES",
                "GIT_COMMON_DIR",
                "GIT_PREFIX",
            ] {
                test.env(name, other.join("unused"));
            }
        }
        let output = test.output().unwrap();
        assert!(
            output.status.success(),
            "hook fixture failed (contaminated={contaminated}):\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
}

#[cfg(unix)]
#[test]
fn push_hook_enforces_content_and_local_policy() {
    use std::os::unix::fs::PermissionsExt;
    let root = workspace();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture =
        std::env::temp_dir().join(format!("dmd-hook-policy-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(fixture.join("scripts")).unwrap();
    std::fs::create_dir_all(fixture.join("bin")).unwrap();
    std::fs::copy(
        root.join("scripts/content_policy.rs"),
        fixture.join("scripts/content_policy.rs"),
    )
    .unwrap();
    let cargo = fixture.join("bin/cargo");
    std::fs::write(&cargo, "exit 0\n").unwrap();
    std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).unwrap();
    let git = |args: &[&str]| {
        let output = without_git_environment(Command::new("git"))
            .current_dir(&fixture)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    git(&["init", "--initial-branch=topic"]);
    git(&["config", "user.name", "Policy Test"]);
    git(&["config", "user.email", &["policy", "invalid"].join("@")]);
    std::fs::write(fixture.join("note.txt"), "Safe content\n").unwrap();
    git(&["add", "note.txt"]);
    git(&["commit", "-m", "Add safe content"]);
    let base = git(&["rev-parse", "HEAD"]);
    let hook = |head: &str, destination: &str| invoke_hook(&fixture, &base, head, destination);
    let clean = hook(&base, "topic");
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stderr)
    );
    assert!(String::from_utf8_lossy(&clean.stderr).contains("no local policy hook configured"));
    let main = hook(&base, "main");
    assert!(!main.status.success());
    assert!(String::from_utf8_lossy(&main.stderr).contains("Pull Request"));
    std::fs::write(
        fixture.join("note.txt"),
        ["", "private", "capture"].join("/"),
    )
    .unwrap();
    git(&["add", "note.txt"]);
    git(&["commit", "-m", "Add a policy fixture"]);
    let bad = hook(&git(&["rev-parse", "HEAD"]), "topic");
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("absolute filesystem path"));
    let local = fixture.join("local-policy");
    std::fs::write(&local, "exit 1\n").unwrap();
    std::fs::set_permissions(&local, std::fs::Permissions::from_mode(0o755)).unwrap();
    git(&["config", "dmd.localPolicy", local.to_str().unwrap()]);
    let failed = hook(&base, "topic");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("local policy hook configured"));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("local policy hook failed"));
}

#[cfg(unix)]
fn invoke_hook(fixture: &Path, base: &str, head: &str, destination: &str) -> std::process::Output {
    let path = std::env::join_paths(
        std::iter::once(fixture.join("bin"))
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let updates = fixture.join("updates");
    std::fs::write(
        &updates,
        format!("refs/heads/topic {head} refs/heads/{destination} {base}\n"),
    )
    .unwrap();
    without_git_environment(Command::new("git"))
        .current_dir(fixture)
        .args([
            "-c",
            &format!("core.hooksPath={}", workspace().join(".githooks").display()),
            "hook",
            "run",
            &format!("--to-stdin={}", updates.display()),
            "pre-push",
        ])
        .env("PATH", &path)
        .output()
        .unwrap()
}
