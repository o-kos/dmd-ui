# Issue #1: Define the issue-driven workflow and the workspace skeleton

Resolves #1.

## Overview

Establish the development process and the minimum Rust workspace that makes its
validation gate real. Nothing can currently be built, checked or reviewed: the
repository holds only a README, a license and ignore rules.

This change is the bootstrap. It is written on a branch created before the workflow it
defines exists, and it is the last change permitted to do so.

## Context

The interface is the product; `dmd` is the layer beneath it and is also a complete
command-line tool. The workspace therefore starts from `dmd` and the shared protocol
types. The interface crate follows once its toolkit is chosen.

Two properties must hold from the first commit, because retrofitting either one is far
more expensive than starting with it:

- A replay-only build must need no C++ toolchain and no signal core. This is what lets
  the interface be developed and tested on its own, and it is the one invariant that CI
  can prove on every push.
- A disabled `native` feature must run no build script, not merely hide calls behind
  `cfg`. An optional dependency introduced with `dep:` is not built at all when its
  feature is off, which gives this property structurally rather than by discipline.

## Decisions

- `AGENTS.md` carries the architectural invariants; `CLAUDE.md` is a symbolic link to it
  so the two can never disagree.
- Clippy strictness lives in `[workspace.lints]`, not on the command line, so a plain
  `cargo clippy` is as strict locally as in CI.
- The workspace declares `default-members` without the native crates, so a bare
  `cargo build` or `cargo test` cannot reach them.
- The native build links a prebuilt static library located through environment
  variables. The repository owns the C ABI contract; it does not build, vendor or
  describe whatever implements it.
- The `pre-push` hook refuses a direct push to `main`. Branch protection is unavailable
  on this repository's plan, so the local hook is the only enforcement point, and a
  merge performed on the server is unaffected by it.
- The hook's content checks are generic: absolute paths, paths escaping the working
  tree, hosts and URLs outside a short allowlist, and e-mail addresses. Any project- or
  host-specific list lives outside the repository and is reached only through an
  optional local policy hook, because such a list discloses exactly what it exists to
  keep out.
- Repository content is written in English.
- Commit messages, Pull Requests, issues and release notes never state or imply that an
  AI tool was used.

## Rejected alternatives

- Requiring `ci/full` as a merge check from this change: the workflow does not exist
  until this change lands, so the requirement would block its own Pull Request. It is
  configured immediately afterwards.
- Keeping a project-specific denylist in the hook: a curated list of the names being
  kept out is itself the disclosure, and the gate would match its own pattern file.
- Creating the interface crate now: its toolkit is undecided, and an empty crate would
  fix CI's system dependencies before that decision is made.

## Implementation steps

- [x] Add `AGENTS.md` with the invariants and conventions, and `CLAUDE.md` as a symlink.
- [x] Add `CONTRIBUTING.md` with the full lifecycle, the validation gate, the external
      review procedure and the plan rules.
- [x] Add `docs/plans/README.md`, `docs/plans/TEMPLATE.md` and `docs/plans/completed/`.
- [x] Add the Issue and Pull Request templates.
- [x] Add `.githooks/pre-push` with the local gate, the `main` refusal, the generic
      content checks and the optional local policy hook.
- [x] Add the Cargo workspace: `rust-toolchain.toml`, `clippy.toml`, workspace lints and
      `default-members`.
- [x] Add `crates/dmd-protocol` and `crates/dmd` so the gate has something to check.
- [x] Add `crates/dmd-native` and `crates/dmd-core-sys` as optional crates, with the C
      ABI header, the linking build script and its diagnostic for missing variables.
- [x] Add the CI workflow publishing `ci/quick` on Draft and `ci/full` otherwise, plus
      the dependency-source policy test.
- [x] Add `CHANGELOG.md`.
- [ ] Complete validation.
- [ ] Move this plan to `docs/plans/completed/` before final review.

## Review follow-up

Keep this plan in place and stop after local commits; do not push.

- [ ] Gate the core linking script behind a default-off feature, propagate native
      activation, and test workspace-wide and explicit crate selection without a core.
- [ ] Reject empty and invalid local policy settings and cover them in hook tests.
- [ ] Clarify the disclosure boundary in the contributor and agent instructions.
- [ ] Require scope and constraints in the bug form.
- [ ] Shorten crate directory names while preserving package and binary names.
- [ ] Run the requested validation, including tests with inherited `GIT_DIR`.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --locked`
- [ ] `cargo test --locked`
- [ ] `cargo build --release --locked`, after the checks above pass
- [ ] `cargo build -p dmd --no-default-features` with no C++ toolchain present
- [ ] `cargo build -p dmd --features native` without the core variables fails naming
      only those variables
- [ ] No build script runs while `native` is disabled
- [ ] The dependency-source policy test rejects a path or git source outside the
      workspace
- [ ] The hook refuses a direct push to `main` and fails on a planted generic violation

## Post-completion

- Require `ci/full` on Pull Requests once the workflow exists on `main`.
- Decide how `main` is protected, given that this repository's plan offers neither
  branch protection nor rulesets.
