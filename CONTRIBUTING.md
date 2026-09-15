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

The implementer works in a dedicated Git worktree checked out on the Issue branch,
outside the main checkout. The main checkout stays on `main`.

Open a Draft Pull Request early. Include a closing keyword such as `Resolves #1`, a
link to the plan and the intended behavior. Keep the description aligned with the final
change. Use English throughout.

## 3. Implement and validate

Do not name or imply other repositories, internal hosts, organisations or filesystem
paths outside this repository, or describe prior or parallel implementations. Public
developer infrastructure, published standards and conventions, and tool-mandated
filenames are outside this disclosure rule's scope.

Make atomic commits with imperative English messages. Tick each plan step in the same
commit that completes it. Never record commit hashes in plans. Keep the authoritative
plan current when a decision changes, and explain the technical reason.

Never state or imply in commit messages, Pull Requests, issues or release notes that
an AI tool was used during development. Do not add co-author trailers or generation
notices. Contributor and agent instructions describing the tooling are outside this
rule's scope.

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
an executable file; empty or invalid values fail the gate. The hook invokes it with
the same arguments and ref-update input as pre-push, and propagates failure. Keep environment-specific policy outside the tree.
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

The ABI contract is `crates/core-sys/include/dmd_core.h`. Its implementation is not
part of this repository. With `native` disabled, the core linking script performs no
environment reads, probing or linking.

## 4. External review

### Roles and invocation settings

Use these fixed roles. Pass the model and reasoning effort explicitly on every
invocation; never rely on machine-local defaults. Apply the listed access mode too.

| Role | Performed by | Model | Reasoning effort and access |
| --- | --- | --- | --- |
| Planning and intent review | Claude | Opus | high |
| Dispatch, Git and Pull Request routine | Claude | Sonnet | medium |
| Reconnaissance | Codex | `gpt-6-astra` | medium, read-only |
| Implementation | Codex | `gpt-6-astra` | high, workspace-write |
| Mechanical code review | Codex | `gpt-5.6-sol` | high, read-only |

Select the mechanical reviewer from the implementer, using the following table. This
selection overrides the usual mechanical-review role when Claude implements the
change, and stays fixed across all rounds of the same review.

| Implementer | Reviewer | CLI model | Effort |
| --- | --- | --- | --- |
| Codex GPT-6 Astra | Codex GPT-5.6 Sol | `gpt-5.6-sol` | high |
| Claude | Codex GPT-6 Astra | `gpt-6-astra` | high |

### First tier: mechanical code review

Once the validation gate passes, write `review-prompt.md` for the particular change.
Name the exact diff to inspect (base and head, plus any uncommitted changes included)
and the Issues it closes. Provide the plan, validation results and known limitations.
Tell the reviewer to read `AGENTS.md` and `CONTRIBUTING.md` first. Define the review
categories for the change: rank rarely executed code whose failures are expensive,
such as workflows, hooks and release scripts, first; then cover architecture, error
handling, tests and scope. Exclude checks already covered by the automated gate.
Require an explicit "nothing found" for each category without a substantive finding;
do not ask for a quota of findings or accept invented findings to fill a category.

Set `review_model` from the reviewer-selection table and invoke the reviewer read-only:

```sh
codex exec -s read-only --model "${review_model:?set from the table}" \
  -c 'model_reasoning_effort="high"' \
  -C "$(git rev-parse --show-toplevel)" "$(cat review-prompt.md)" < /dev/null
```

Stdin must be closed with `< /dev/null`; without it the command waits for input
forever. Do not use `codex review --base <branch>`: it cannot take a custom prompt.

The orchestrator returns findings to the implementer as one structured list per
round, with category, severity, location, reasoning and requested correction. Agents
do not negotiate with each other; the orchestrator decides which findings to accept
or decline. Fix accepted findings in focused commits and rerun affected checks. Route
out-of-scope findings to separate Issues.

Repeat with the same reviewer until a round returns nothing substantive. Each later
prompt names what was fixed and what was declined, explains each decision, and asks
the reviewer to challenge the reasoning behind every decline. A reviewer that never
disagrees is worth nothing.

### Second tier: intent review

After the mechanical review is clean, the orchestrator (Claude Opus, explicitly at
high reasoning effort) reviews the final diff once against the approved plan, owner
decisions and every invariant. This tier asks whether the result is what was requested
and preserves every invariant; code correctness was covered by the first tier.
Return any findings as one structured list to the implementer under the same
orchestrator decision rule. Allow at most two correction rounds in this tier, then
escalate unresolved findings to the project owner.

### Owner review

When asking the owner to review, summarise the automatic review: its findings, which
were accepted and how they were addressed, which were declined and why, and whether
the final round was clean. Resolve every blocking finding and obtain approval; a
clean automated gate is not review. Obtain project-owner agreement for every lint
suppression before pushing it.

## 5. Complete

Move the finished plan to `docs/plans/completed/` before final review and update its
links. Run the gate, then `cargo build --release --locked`. Mark the Pull Request Ready
and wait for explicit success of `ci/full` and final review approval. Merge by squash
only, using an imperative English title and a summary of the resulting behavior.
Confirm the closing keyword closes the Issue, then remove the merged branch.

Maintain `CHANGELOG.md` in Keep a Changelog style under `Unreleased` for user-visible
changes. Internal workflow plumbing does not earn an entry.
