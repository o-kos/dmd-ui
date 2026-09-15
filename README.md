# dmd-ui

An application for demodulating recorded signals and inspecting the results.

The interface is the product. Between it and the demodulation libraries sits `dmd`, a
command-line tool that gives the interface one uniform way to reach demodulation
regardless of what is behind it. The interface starts `dmd` for a run and reads its
events; it never loads a demodulation library itself and never links a signal core.

`dmd` is also a complete tool in its own right, usable directly from a terminal for
demodulating, inspecting modules and checking results.

`dmd` resolves what is behind it from the contents of its search path:

- **Native** performs real demodulation through a signal core and dynamically loaded
  demodulation libraries. It is enabled by the `native` Cargo feature.
- **Replay** reproduces prepared recordings of earlier runs. It is always available and
  needs no native toolchain, so the interface can be developed, tested and demonstrated
  without the demodulation stack present.

Both paths share one command-line surface, one event protocol and one set of result
files. The interface cannot tell them apart beyond the data they produce, which is what
makes a recording a usable stand-in for a real run.

## Status

Early bootstrap. Work starts from `dmd` and the shared protocol types; the interface
crate follows once its toolkit is chosen. The event protocol and the recording format
are being specified before implementation. See `docs/plans/` for the active work.

## Building

```sh
# Command-line tool with replay only: no C++ toolchain required
cargo build -p dmd --no-default-features

# Full build, including native demodulation
cargo build -p dmd --features native
```

The full build links a prebuilt signal core; see `CONTRIBUTING.md` for the environment
variables that locate it.

## Contributing

`CONTRIBUTING.md` describes the issue-driven workflow, the validation gate and the
review process. `AGENTS.md` carries the architectural invariants that every change must
preserve.

## License

MIT. See `LICENSE`.
