# Issue #4: Document the server-side protection of main

Resolves #4.

## Overview

`CONTRIBUTING.md` describes branch protection as unavailable, states that the local
`pre-push` hook is the only enforcement point for direct pushes to `main`, and that the
required full check is still to be configured. None of this holds any more. This change
brings the contributor and agent instructions in line with the protection that is
actually configured on `main`.

Outside scope: changing the protection settings, rewriting the completed bootstrap
plan, and the product rename tracked in #3.

## Context

The bootstrap plan recorded server-side protection as unavailable. After it merged,
`main` turned out to be protected already: a Pull Request is required, history must be
linear, conversations must be resolved, and force pushes and deletion are forbidden.
The required `ci/full` check was then added, bound to GitHub Actions and requiring a
branch that is up to date with `main`. Administrators are not included in the
protection, so an administrator can still push to or merge into `main` directly.

## Decisions

- `CONTRIBUTING.md` describes the server-side rules as the enforcement of the merge
  path, next to the existing description of the hook and the CI tiers, and names
  `ci/full` as the required check together with its up-to-date requirement.
- The hook stays documented as an early, local copy of the gate. It keeps refusing
  direct pushes to `main`, because administrators are not included in the server-side
  protection and the hook is then the only barrier against an accidental direct push.
- `AGENTS.md` gains one statement that a merge waits for the required `ci/full` check,
  so the agent instructions carry the rule without repeating the full description.
- The completed bootstrap plan stays unchanged: it records what was decided at the
  time, and this plan records the correction.

## Rejected alternatives

- Dropping the hook's refusal of direct pushes to `main`: with administrators outside
  the protection, the owner could push to `main` by accident with nothing to stop it.
- Including administrators in the protection as part of this change: it alters a
  repository setting rather than documentation, and the owner decides it separately.
- Editing the completed bootstrap plan: completed plans are a durable record of
  decisions as they were made, not current documentation.

## Implementation steps

- [x] Describe the server-side protection of `main`, the required `ci/full` check and
      the administrator exception in `CONTRIBUTING.md`, and describe the hook as the
      local copy of the gate.
- [x] State in `AGENTS.md` that a merge waits for the required `ci/full` check.
- [ ] Complete validation.
- [ ] Move this plan to `docs/plans/completed/` before final review.

## Validation

- [ ] Verify each Issue acceptance criterion.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --locked`
      (warnings are denied in `[workspace.lints]`)
- [ ] `cargo clippy -p dmd --features native --all-targets --locked` with an empty core
      directory (see `CONTRIBUTING.md` for exact commands)
- [ ] `cargo test --locked`
- [ ] `cargo test -p dmd-native --locked`
- [ ] The content policy accepts every outgoing commit
- [ ] `cargo build --release --locked`, after the gate passes

## Post-completion

- The owner decides whether administrators are included in the protection of `main`.
