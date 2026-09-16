# Issue #3: Rename the product to Wireroom

Resolves #3.

## Overview

The repository is named `wireroom`, but its content still presents the product under
its old working title or leaves it unnamed. The product is **Wireroom**: an operator's
console for tuning to a recorded signal, setting processing parameters, running
demodulation and watching progress as a log and in separate panes for decoded text,
bits, callsigns and the phase constellation. After this change the README and the
contributor and agent instructions name it, record the binary names, and the repository
carries the application icon as vector sources.

Outside scope: renaming the `dmd` binary, the `dmd-*` crates, the `DMD_CORE_*`
variables, the `dmd.localPolicy` setting or the C ABI header; creating the interface
crate; raster icon sizes and platform bundles.

## Context

`dmd` is the command-line layer beneath the interface and keeps its name. The
interface crate does not exist yet, so its binary name can only be recorded as a
decision for when the crate is created. The pre-push content policy rejects URLs whose
host is not on a short allowlist, and every standalone SVG file declares its namespace
as a URL on the W3C host.

## Decisions

- The interface binary will be `wr`; the command-line tool stays `dmd`. A published
  package would be named `wireroom`, because the `wr` crate name is taken.
- The icon is four constellation points inside a carrier-lock loop with an arrowhead:
  cyan points and a coral loop, each with a dark navy outline, drawn with round-capped
  strokes on a 100-unit grid. It ships as `assets/icons/wireroom.svg` and a single-colour
  `assets/icons/wireroom-mono.svg` that uses `currentColor`, for monochrome contexts
  such as a system tray.
- Small sizes get their own optical sources, `assets/icons/wireroom-16.svg` and
  `assets/icons/wireroom-mono-16.svg`. Round points of the main drawing merge into one
  blob at 16 pixels, so the small sources place four pixel-aligned squares, each two
  pixels wide and two pixels from its neighbour at that size, inside a thinner loop.
  They drop the point outlines that blur at that size and replace the arrowhead chevron
  with a filled triangle.
- Icons live under a repository-level `assets/icons/` directory until the interface
  crate exists and takes ownership of them.
- The host of the SVG namespace joins the content policy's host allowlist. It belongs
  to a published standard that the disclosure rule explicitly excludes, and a valid
  standalone SVG cannot omit its namespace. Markup closing tags are recognised as tags
  only when the same checked text opened that element earlier; otherwise they remain
  subject to the absolute-path check.

## Rejected alternatives

- Omitting the `xmlns` declaration to satisfy the content policy: a standalone SVG
  without its namespace does not render as an image.
- Renaming the `dmd-*` crates to `wireroom-*`: they belong to the command-line layer,
  whose name does not change.
- Committing raster sizes now: they need a separate review of binary contents and
  belong with the interface crate's packaging.

## Implementation steps

- [x] Allow the SVG namespace host and recognise markup closing tags in the content
      policy, with unit tests.
- [x] Add the colour and monochrome icon sources under `assets/icons/`.
- [x] Name the product Wireroom and record the `wr` and `dmd` binaries in `README.md`,
      `AGENTS.md` and `CONTRIBUTING.md`, and add an `Unreleased` entry to `CHANGELOG.md`.
- [x] Complete validation.
- [x] Move this plan to `docs/plans/completed/` before final review.

## Review round 1

- [x] Recognise a closing tag only after an opening tag for the same element in the
      same checked text, so a single-component path in angle brackets is rejected.
- [x] State the `wr` and `dmd` binary names in `CONTRIBUTING.md`.
- [x] Record the completed validation in this plan.

## Review round 2

- [x] Rewrite the earlier test additions so every commit of this change passes the
      final content policy, not only the commits of the push that introduced them.
- [x] Add optical small-size icon sources that keep the four points distinct at 16
      pixels, instead of deferring that acceptance criterion.
- [x] Record both corrections in this plan.

## Validation

- [x] Verify each Issue acceptance criterion.
- [x] Searching tracked content for the old product title finds no match outside
      `docs/plans/completed/`.
- [x] All four icon sources render with `rsvg-convert` at 16, 24 and 64 pixels and
      contain no external references or embedded rasters. The main sources read clearly
      at 24 and 64 pixels; the small-size sources keep the four points distinct at 16
      pixels.
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --locked`
      (warnings are denied in `[workspace.lints]`)
- [x] `cargo clippy -p dmd --features native --all-targets --locked` with an empty core
      directory (see `CONTRIBUTING.md` for exact commands)
- [x] `cargo test --locked`
- [x] `cargo test -p dmd-native --locked`
- [x] The content policy accepts every outgoing commit
- [x] `cargo build --release --locked`, after the gate passes

## Post-completion

- Generate raster sizes and platform icon bundles from these sources when the interface
  crate is created, taking 16-pixel rasters from the small-size sources.
