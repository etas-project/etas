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
The SDK stores a shared declaration/type object, one body object per module,
and a semantic dependency index. HIR nodes and their checked Type/Effect facts
have stable IDs and are stored once. Each module indexes its definition bodies;
startup decodes the definitions in the selected closure. Original source is retained for diagnostics
and reflection. `etas run` takes the transitive closure of its checked calls and
methods, reads only the corresponding module objects, then links those saved
facts without parsing or solving the standard library.

A derived object-pack cache combines the selected official objects. Its key
includes the SDK manifest, entry set, compiler and runtime ABI. It lives outside
the SDK (`$XDG_CACHE_HOME/etas/std-objects`, or `~/.cache/etas/std-objects`);
`ETAS_STD_CACHE_ROOT` overrides that directory. Deleting it is supported: the next
run links the canonical objects directly and rebuilds the pack. Cached bytes are
verified against the official object hashes. Installation does not precompute
closure images or populate this cache.

Public calling signatures are normalized and imported into a frozen type
universe during installation. Ordinary runs restore those solved facts and bind
the caller's source symbols; a portable signature cache handles other package
slots. Reflection revision coordinates are decoded when requested. Source method
behavior graphs and flow summaries are stored separately; the checker loads
those selected by checked calls, function values and witnesses, together with
their semantic dependency closures. Ordinary runs use installed module origins
and source-body exposure directly. The retained source archive parses a file
only when a diagnostic or reflection operation requests its AST. SDK type
descriptor fingerprints are also prepared during installation; startup computes
fingerprints for the caller's new types. Standalone source applications reuse
the public interface's installation-time validation. Unchanged imported type
bases keep their shared frozen index. Linked module views retain the selected
bodies' Effect captures and environments. Installed ABI records use a byte index
by TypeId; runtime decodes each requested record once and shares that result.
The installation receipt binds the complete SDK manifest to the CLI's compiled
build stamp, avoiding a repeated scan of the executable. Code and dependency
changes regenerate this stamp.

Reflection, directly imported std values and caller-performed actions currently
load all canonical module objects. Programs with residual
monitors retain proof rebinding;
unpaired SDKs use the portable checking path. Runtime permissions, budgets and
trace enforcement still apply.

Measure startup in fresh processes with project caching disabled:

```sh
python3 scripts/ci/benchmark_std_startup.py --binary /path/to/installed/bin/etas --receipt-dir /tmp/etas-startup
```

The benchmark checks a `std.text.join` program and records wall time, stage
timings, loaded body size, selected behavior count and standard-source parse
count, including SDK cache misses and hits. `--sdk-cache miss` deletes the derived
cache before each process; `--sdk-cache hit` primes it once. Pass `--baseline /path/to/older/bin/etas` to compare versions, or `--cpu N` to pin the samples to one CPU. The default target is a
median below 500 ms across 20 runs per SDK cache mode; pass `--max-ms 100` for a 100 ms budget.
Stage timings can include nested work.

A local Linux release comparison interleaved 20 fresh processes per SDK cache
mode on CPU 42, with project caching off:

| Derived object-pack cache | CLI 0.20.0 | CLI 0.19.1 baseline |
| --- | ---: | ---: |
| Deleted before each process | 295.8 ms | 243.8 ms |
| Primed before measurement | 290.6 ms | 238.7 ms |

These are wall-time medians for the join program above. CLI 0.19.1 uses installed
closure images in both cohorts and has no derived object-pack cache. Every
0.20.0 sample parsed zero standard-library sources; object-pack misses and hits
were verified separately. Checked object assembly took about 43.2 ms without a
pack and 41.8 ms with one. Stage timings can include nested work.

The complete 0.20.0 SDK occupies 32.37 MB, compared with 145.38 MB for the old
617-image release. Its 51 module references resolve to 50 unique canonical
objects, with no precomputed execution-closure images or unreferenced objects.
Standard-library sources, public interfaces and the global interpreter layout
are included in those SDK totals. The shared format currently adds about
50 ms to this short program's startup compared with the old precomputed images;
the 100 ms budget remains unmet.

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

- [Local checkout integration and verification](docs/development/local-checkouts.md)
- [Examples](examples/)
- [Language specification](docs/design/README.md)
- [Standard-library version and lockfile contract](docs/design/15-package-management.md#71-using-the-installed-standard-library)
- [VS Code extension](https://github.com/etas-project/etas-ide)
- [Etas Development Kit](https://github.com/etas-project/etas-edk)

## Citation

If you use Etas in research, please cite
[ETAS: An Effect-Typed Language for Agent Systems](https://arxiv.org/abs/2607.17780).
Machine-readable citation metadata, including the preferred paper citation, is
available in [`CITATION.cff`](CITATION.cff).

## License

[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)
