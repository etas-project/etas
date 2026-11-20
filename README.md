# Etas

An effect-typed programming language for agent systems.

Etas brings agent orchestration into a typed language: `agent` describes
model-driven behavior, `flow` expresses computation and coordination, and
`tool` defines interfaces that models may call. Effects and handlers make
external operations explicit rather than hiding them in orchestration code.

The toolchain provides:

- **Safety:** check types and possible actions, with host authority enforced
  separately at execution time.
- **Reliability:** express errors, recovery, and resource limits explicitly.
- **Inspection:** view inferred effects and requested actions before execution.

Experimental research prototype. Syntax and APIs may change; not production-ready.

The CLI checks programs and executes checked HIR directly through
`etas-interpreter`.

## Quickstart

Requires Rust 1.85+ and a native C/C++ build toolchain.

```sh
git clone https://github.com/etas-project/etas.git
cd etas
cargo run --locked --release -p etas_cli -- std install --prefix "${CARGO_HOME:-$HOME/.cargo}"
etas run examples/hello.es
```

No model provider or API key is needed for this example.

The installer places `etas` in Cargo's binary directory and builds the
matching SDK from the standard-library sources pinned by Cargo. The CLI uses
this SDK by default. Pass `--std-provider legacy` to select the legacy registry.
On Linux, `std install` prepares checked SDK data, public query indices and
rootless interpreter layouts for the matching toolchain build. The compiler records
semantic dependencies from solved calls, types, generic bounds and Effect facts.
The installer takes their transitive closures and packages the checked IR and
analysis results directly. Original source is retained for diagnostics and
reflection. `etas run` selects the smallest installed execution bundle covering
its checked calls and methods; declaration and type indices stay shared.
Public calling signatures are normalized during installation. Source method
behavior graphs and flow summaries are stored separately; the checker loads
those selected by checked calls, function values and witnesses, together with
their semantic dependency closures. Ordinary runs use installed module origins
and source-body exposure directly. The retained source archive parses a file
only when a diagnostic or reflection operation requests its AST. SDK type
descriptor fingerprints are also prepared during installation; startup computes
fingerprints for the caller's new types. Standalone source applications reuse
the public interface's installation-time validation.
The installation receipt binds the complete SDK manifest to the CLI's compiled
build stamp, avoiding a repeated scan of the executable. Code and dependency
changes regenerate this stamp.

Reflection, directly imported std values, caller-performed actions and calls
spanning the available bundles use the complete SDK. Programs with residual
monitors retain proof rebinding;
unpaired SDKs use the portable checking path. Runtime permissions, budgets and
trace enforcement still apply.

Measure startup in fresh processes with project caching disabled:

```sh
python3 scripts/ci/benchmark_std_startup.py --binary /path/to/installed/bin/etas --receipt-dir /tmp/etas-startup
```

The benchmark checks a `std.text.join` program and records wall time, stage
timings, loaded body size, selected behavior count and standard-source parse
count. Pass `--baseline /path/to/older/bin/etas` to compare versions, or `--cpu N` to pin the samples to one CPU. The default target is a
median below 500 ms across ten runs; pass `--max-ms 100` for a 100 ms budget.
Stage timings can include nested work.

A local Linux release comparison of 20 fresh processes on CPU 42, with project
caching off, measured a 352.5 ms median for CLI 0.18.0 versus 466.2 ms for
0.17.0 on this join program. The complete SDK contained 617 execution bundles;
the program loaded 15 of 442 callable behaviors and parsed no standard-library
sources. The 100 ms budget remains unmet; public signature loading, type
association, and executable-image/layout decoding remain significant costs.

To inspect a program before running it:

```sh
etas check examples/hello.es
etas effects examples/hello.es
```

`check` reports syntax, type, and effect errors; `effects` shows the program's
inferred effects and requested actions. Model or network examples additionally
need the corresponding host services and permissions configured.

## A Minimal Program

```etas
module hello;

import std.io.println;

flow main(args: Array<string>) -> i32 {
    println("Hello, Etas!");
    return 0;
}
```

Source files use the `.es` extension. A `flow` is a callable computation;
`main` is the program entry and returns an exit status. Effect annotations
are optional here: the compiler infers them from calls such as `println`.

## Next Steps

- [Examples](examples/)
- [Language specification](docs/design/README.md)
- [Building a standard library SDK and upgrading an existing lockfile](docs/design/15-package-management.md#71-using-the-installed-standard-library)
- [VS Code extension](https://github.com/etas-project/etas-ide)
- [Etas Development Kit](https://github.com/etas-project/etas-edk)

## Citation

If you use Etas in research, please cite
[ETAS: An Effect-Typed Language for Agent Systems](https://arxiv.org/abs/2607.17780).
Machine-readable citation metadata, including the preferred paper citation, is
available in [`CITATION.cff`](CITATION.cff).

## License

[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)
