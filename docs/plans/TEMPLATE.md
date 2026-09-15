# Issue #<number>: <title>

Resolves #<number>.

## Overview

Describe the problem and observable outcome. State what is outside scope.

## Context

Explain constraints and architectural invariants relevant to this change.

## Decisions

State each decision and its technical rationale.

## Rejected alternatives

Describe alternatives and why they do not meet this change's needs.

## Implementation steps

- [ ] Implement a concrete, reviewable unit of work.
- [ ] Complete validation.
- [ ] Move this plan to `docs/plans/completed/` before final review.

## Validation

- [ ] Verify each Issue acceptance criterion.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --locked`
- [ ] `cargo test --locked`
- [ ] `cargo build --release --locked`, after the gate passes

## Post-completion

List follow-up Issues or operational steps, if any.
