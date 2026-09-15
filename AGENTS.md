# Project context

## Purpose and architecture

The interface is the product: an application for demodulating recorded signals and
inspecting results. It starts `dmd` and consumes its events. `dmd` is also a complete
command-line tool. Only the optional native layer reaches the signal core through the
C ABI. Everything implementing that ABI is outside this repository's scope. The
interface toolkit is undecided; do not create an interface crate yet.

## Workspace

- `crates/protocol`: shared protocol boundary; currently only its version.
- `crates/cli`: command-line surface and backend selection.
- `crates/native`: safe resource ownership over the C ABI.
- `crates/core-sys`: C header, matching raw declarations and static linking.
- `docs/plans`: active plans; `docs/plans/completed`: finished plans.
- `.github/scripts`: CI decision logic; `scripts`: local policy checks.

The event protocol, recording format and result layout belong to later work.

## Commands

Use the toolchain pinned by `rust-toolchain.toml`.

```sh
cargo build
cargo run -p dmd -- --help
cargo run -p dmd -- --backend replay list
cargo fmt --all -- --check
cargo clippy --all-targets --locked
cargo test --locked
cargo build --release --locked
cargo build -p dmd --no-default-features
cargo build -p dmd --features native
```

The CLI currently reports that each command is not implemented. Native builds require
`DMD_CORE_LIB_DIR`; `DMD_CORE_LIB_NAME` defaults to `dmd_core`, and
`DMD_CORE_EXTRA_LIBS` optionally lists additional link targets separated by semicolons.

## Architectural invariants

- `native` is not a default feature. Optional dependencies isolate default builds;
  the core linking script is inert without its own `native` feature, even when selected
  explicitly or through `--workspace`.
- Replay-only builds and tests require no Python, CMake or C++ toolchain.
- There is no silent fallback. A core that fails to load or a corrupt recording is an
  error, never a quiet switch to the other backend.
- Sample positions are frames of the source file. For IQ, one frame is an I/Q pair.
- Paths from recordings and result identifiers stay inside the output directory.
  Never delete an arbitrary directory recursively.
- No STL types or C++ exceptions cross the C ABI. Memory is released by its allocator's
  side, and Rust RAII closes resources. Panics must not unwind across FFI boundaries.
- Replay-only options are never silently ignored during a native run.
- Event protocol, recording format and C ABI versions are independent.
- Use `thiserror` in libraries and `anyhow` in binaries. Non-test code must not
  `unwrap` or `expect` on external data.
- Every lint suppression requires project-owner agreement before pushing, uses
  `#[expect(..., reason = "...")]`, and is a last resort after refactoring.

## Engineering and workflow

Write all repository content in English.

Do not name or imply other repositories, internal hosts, organisations or filesystem
paths outside this repository, or describe prior or parallel implementations. Public
developer infrastructure, published standards and conventions, and tool-mandated
filenames are outside this disclosure rule's scope.

Dependencies may use the package registry or workspace paths, never external path or
git sources.
Maintain individually selected workspace lints and inherit them in every crate.

Start with an Issue containing verifiable acceptance criteria, then a branch named
`<type>/<issue>-<short-description>`. Commit a plan before implementation and open a
Draft Pull Request early with a closing keyword. Complete each plan checkbox in the
commit that completes its work; never record commit hashes. Commit atomic changes with
English imperative messages and no attribution trailers or generation notices.

Install `.githooks` with `git config core.hooksPath .githooks`. Run the validation gate
before review. Route out-of-scope findings to separate Issues. Complete external review,
move the plan to `completed/`, rebuild release, mark Ready, and merge by squash only.
See `CONTRIBUTING.md` for the complete lifecycle and bypass rules.
