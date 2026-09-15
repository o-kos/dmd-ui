#[cfg(not(feature = "native"))]
fn main() {}

#[cfg(feature = "native")]
fn main() {
    for name in [
        "DMD_CORE_LIB_DIR",
        "DMD_CORE_LIB_NAME",
        "DMD_CORE_EXTRA_LIBS",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let directory = variable("DMD_CORE_LIB_DIR").filter(|value| !value.trim().is_empty());
    let Some(directory) = directory else {
        panic!(
            "DMD_CORE_LIB_DIR must be set; DMD_CORE_LIB_NAME defaults to dmd_core; DMD_CORE_EXTRA_LIBS is optional"
        );
    };
    let name = variable("DMD_CORE_LIB_NAME").unwrap_or_else(|| "dmd_core".into());
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
    if let Some(extra) = variable("DMD_CORE_EXTRA_LIBS") {
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

#[cfg(feature = "native")]
fn variable(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => panic!("{name} must be valid Unicode"),
    }
}
