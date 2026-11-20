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
On Linux, `std install` saves the solved Effect analysis for the matching
executable. `etas run` reuses it and checks the execution contracts on load.
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
