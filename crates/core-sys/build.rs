#[cfg(not(feature = "native"))]
fn main() {}

#[cfg(feature = "native")]
fn main() {
    use std::env;

    for name in [
        "DMD_CORE_LIB_DIR",
        "DMD_CORE_LIB_NAME",
        "DMD_CORE_EXTRA_LIBS",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let directory = env::var("DMD_CORE_LIB_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let Some(directory) = directory else {
        panic!(
            "DMD_CORE_LIB_DIR must be set; DMD_CORE_LIB_NAME defaults to dmd_core; DMD_CORE_EXTRA_LIBS is optional"
        );
    };
    let name = env::var("DMD_CORE_LIB_NAME").unwrap_or_else(|_| "dmd_core".into());
    assert!(
        !name.trim().is_empty() && !name.contains(['\n', '\r']),
        "DMD_CORE_LIB_NAME must be nonempty and single-line"
    );
    assert!(
        !directory.contains(['\n', '\r']),
        "DMD_CORE_LIB_DIR must be single-line"
    );
    println!("cargo:rustc-link-search=native={directory}");
    println!("cargo:rustc-link-lib=static={name}");
    if let Ok(extra) = env::var("DMD_CORE_EXTRA_LIBS") {
        assert!(
            !extra.contains(['\n', '\r']),
            "DMD_CORE_EXTRA_LIBS must be single-line"
        );
        for library in extra
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            println!("cargo:rustc-link-lib={library}");
        }
    }
}
