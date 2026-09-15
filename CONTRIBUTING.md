# Contributing

## 1. Define the change

Create an Issue using the bug or change form. Describe observable behavior, scope,
constraints and acceptance criteria that a reviewer can verify. Route unrelated bugs,
design questions and improvements to separate Issues; link them without expanding the
current scope silently.

## 2. Plan before implementation

Create a branch from current `main` named `<type>/<issue>-<short-description>` (for
example, `docs/1-project-workflow`). Copy `docs/plans/TEMPLATE.md` to a numbered plan in
`docs/plans/`. Record decisions, rejected alternatives, steps and validation. Commit
that plan before implementation. This bootstrap's pre-existing branch is the sole
exception to that ordering.

Open a Draft Pull Request early. Include a closing keyword such as `Resolves #1`, a
link to the plan and the intended behavior. Keep the description aligned with the final
change. Use English throughout; do not include attribution trailers or generation
notices in commits, Issues, Pull Requests or release notes.

## 3. Implement and validate

Make atomic commits with imperative English messages. Tick each plan step in the same
commit that completes it. Never record commit hashes in plans. Keep the authoritative
plan current when a decision changes, and explain the technical reason.

Install the local gate once per checkout:

```sh
git config core.hooksPath .githooks
```

The pre-push hook refuses direct pushes to `main`: changes reach it through a Pull
Request. It checks outgoing commit messages, filenames and added text for absolute
filesystem paths, paths escaping the tree, e-mail addresses and unapproved hosts or
URLs. The small generic host allowlist lives in `scripts/content_policy.rs`. It is a
text hygiene check, not a secrets scanner. Avoid binary additions that require text
review without a separate review of their contents.

The hook reports whether `dmd.localPolicy` is configured. When configured, it must name
an executable; the hook invokes it with the same arguments and ref-update input as
pre-push, and propagates failure. Keep environment-specific policy outside the tree.
Configure it with `git config dmd.localPolicy <executable>`.

The gate stops at the first failed command and explains the failure:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked
cargo test --locked
```

Use `git push --no-verify` only as an explicit escape hatch. State the reason and the
checks bypassed in the Pull Request. A bypass does not replace validation or review.
The local hook is the enforcement point for direct pushes because branch protection
is unavailable. Server-side merges do not execute it.

Draft Pull Requests run formatting, lints and policy tests on Linux, producing
`ci/quick`. Ready Pull Requests, pushes to `main` and manual runs require Linux,
Windows and macOS tests and release builds, producing `ci/full`. Skipped results do
not count as successes. Configure the required full check once the workflow exists.

## Native builds

The default build and tests require only Rust and the platform's ordinary Rust linker,
with no Python, CMake, C++ compiler or signal core. Optional dependencies keep native
crates out of default builds. Even explicit crate or workspace-wide selection requires
no core: the core linking script is inert unless its default-off `native` feature is
enabled transitively by the CLI native feature.

For a native build, set `DMD_CORE_LIB_DIR` to the directory containing the prebuilt
static library. `DMD_CORE_LIB_NAME` defaults to `dmd_core` (without a filename prefix or
extension). `DMD_CORE_EXTRA_LIBS` is an optional semicolon-separated list of additional
link targets, using Cargo link-library syntax. Then run:

```sh
cargo build -p dmd --features native
```

The ABI contract is `crates/dmd-core-sys/include/dmd_core.h`. Its implementation is not
part of this repository. With `native` disabled, the core linking script performs no
environment reads, probing or linking.

## 4. External review

Once the gate passes, request an independent reviewer. Provide the Issue, plan, diff,
validation results and known limitations. The reviewer checks acceptance criteria,
architecture, error handling, tests and scope. Record actionable findings in the Pull
Request, fix them in focused commits, and rerun affected checks. Route out-of-scope
findings to Issues. Resolve every blocking finding and obtain approval; do not treat a
clean automated run as review. Obtain project-owner agreement for every lint
suppression before pushing it.

## 5. Complete

Move the finished plan to `docs/plans/completed/` before final review and update its
links. Run the gate, then `cargo build --release --locked`. Mark the Pull Request Ready
and wait for explicit success of `ci/full` and final review approval. Merge by squash
only, using an imperative English title and a summary of the resulting behavior.
Confirm the closing keyword closes the Issue, then remove the merged branch.

Maintain `CHANGELOG.md` in Keep a Changelog style under `Unreleased` for user-visible
changes. Internal workflow plumbing does not earn an entry.
