use std::{path::Path, process::Command};

#[test]
fn workspace_and_explicit_core_selection_need_no_native_core() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    // A separate target avoids locking the Cargo process running this test.
    let target = root.join("target/native-isolation");
    for args in [
        vec!["check", "--workspace", "--all-targets", "--locked"],
        vec!["test", "--workspace", "--no-run", "--locked"],
        vec!["build", "-p", "dmd-core-sys", "--locked"],
        vec!["build", "-p", "dmd", "--features", "native"],
    ] {
        let output = Command::new(env!("CARGO"))
            .current_dir(root)
            .args(&args)
            .arg("--target-dir")
            .arg(&target)
            .env_remove("DMD_CORE_LIB_DIR")
            .env_remove("DMD_CORE_LIB_NAME")
            .env_remove("DMD_CORE_EXTRA_LIBS")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if args.contains(&"native") {
            assert!(
                !output.status.success(),
                "native build unexpectedly succeeded"
            );
            assert!(
                stderr.contains("DMD_CORE_LIB_DIR must be set; DMD_CORE_LIB_NAME defaults to dmd_core; DMD_CORE_EXTRA_LIBS is optional"),
                "{stderr}"
            );
        } else {
            assert!(output.status.success(), "{args:?}:\n{stderr}");
        }
    }
}
