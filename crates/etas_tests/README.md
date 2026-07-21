# etas_tests

`etas_tests` is the workspace acceptance-test package for Etas.

It follows the boundary in `Docs/Architect/etas-syntax-design.md`:

- `etas_syntax` owns lexing, parsing, source spans, syntax diagnostics, parsed
  AST nodes, recovery nodes, and AST dump support.
- Syntax-specific AST golden tests stay in `crates/etas_syntax/tests/`.
- `etas_tests` exercises public cross-crate behavior only: parser API, HIR
  lowering, CLI commands, and future AIR/runtime public APIs.
- Tests must not depend on parser implementation details such as `chumsky`
  types or error values.

Fixture convention:

- `fixtures/syntax/valid/` contains syntax-only positive parser inputs.
- `fixtures/syntax/valid/modules/` contains dedicated Module/Import syntax
  inputs for plain imports, item imports, aliases, grouped imports, wildcard
  imports, public re-exports, and import-only program prefixes.
- `fixtures/syntax/recovery/` contains incomplete or malformed syntax inputs
  that should still produce a parse value plus syntax diagnostics.
- `fixtures/hir/positive/` and `fixtures/hir/negative/` contain inputs for
  HIR lowering, symbol, scope, and name-resolution behavior.
- `fixtures/hir/golden/` contains expected HIR dump output.
- `fixtures/cli/positive/` and `fixtures/cli/negative/` contain command-surface
  inputs used to validate exit codes and output routing.
- `fixtures/compiler/algorithms/positive/` contains general-purpose language
  algorithm programs. This corpus must stay at or above 30 positive algorithm
  fixtures. Source files use `.es`; matching `.io.txt` files record the
  command, stdin, stdout, and expected exit status.
- `fixtures/compiler/functional/positive/` contains functional-language
  feature programs outside the algorithm corpus. This corpus must stay at
  exactly 10 positive fixtures unless the test is updated, and covers anonymous
  flows, higher-order flow parameters, returned flows, closure capture,
  currying, folds, filters, mapping, thunk forcing, and stage composition.
  Source files use `.es`; matching `.io.txt` files record the command, stdin,
  stdout, and expected exit status.
- `fixtures/compiler/type_system/positive/` and
  `fixtures/compiler/type_system/negative/` contain focused compiler fixtures
  for the current Type System SPEC. This corpus covers transparent `alias`
  declarations, generic alias expansion, nominal `type` identity, explicit
  nominal construction, bodyless host-returned types, alias cycles, and record
  opacity. Negative fixtures have matching `.diagnostics.txt` diagnostic
  contracts.
- `fixtures/compiler/algorithms/negative/` contains semantic/compiler negative
  algorithm programs. Matching `.diagnostics.txt` files record the expected
  diagnostic contract.
- `fixtures/compiler/effects/positive/` and
  `fixtures/compiler/effects/negative/` contain Effect/Handler compiler
  fixtures written against the current SPEC: positive effect rows use `![...]`
  upper bounds, handlers match concrete actions, and handler-produced effects
  are checked through `etas check`; negative fixtures have matching
  `.diagnostics.txt` diagnostic contracts. This corpus must stay at exactly 115
  positive `.es` source files and exactly 115 negative `.es` source files
  unless the test is updated. Project import fixtures are layered directories:
  `main.es` is the end-to-end regression entry, while `support.es` is imported
  module coverage. The effect harness therefore also fixes the end-to-end
  regression case count at 107 positive entries and 107 negative entries. Each
  source file must have at least 100 lines, include a lambda expression, define
  unique flows, keep every flow body at or above 10 lines, and contain at least
  three inter-flow call links.
- `fixtures/compiler/handlers/positive/` and
  `fixtures/compiler/handlers/negative/` contain focused Handler compiler
  fixtures written against the current Effect System SPEC. This corpus must
  stay at exactly 12 positive `.es` files and exactly 16 negative `.es` files
  unless the test is updated. Positive fixtures cover handler type forms,
  inline handler blocks, reusable handler values, trailing `with` syntax,
  first-class handler parameters and returns, produced effect rows, nested
  handlers, `Error[IndexError]` fallback, and top-level handler-value bindings.
  Negative fixtures cover invalid `resume` placement, multiple resume,
  resuming never actions, captured resume continuations, bare handler arm
  blocks, malformed handler action arms, produced effect row escape, handled
  row mismatch, top-level `handle` expressions, and `with` values that are not
  handlers. They also include regression coverage for `finish` placement and
  type checking, `return` inside handler arms, and rejection of the obsolete
  anonymous application form `expr with handler { ... }`; direct anonymous
  application must be `expr with { ... }`. Negative `.diagnostics.txt` files use the
  `diagnostic-code | stable message fragment` contract.
- `fixtures/compiler/policies/positive/` and
  `fixtures/compiler/policies/negative/` contain Policy compiler fixtures using
  current source forms such as `allow Web;`, `deny Web.post[_];`, and
  `require Approval.request before Email.send[WorkAccount];`. Positive fixtures
  must satisfy their policy through `etas check`; negative fixtures have
  matching `.diagnostics.txt` diagnostic contracts. This corpus must stay at
  exactly 15 positive `.es` files and exactly 15 negative `.es` files unless
  the test is updated. Each fixture follows the same complexity contract as
  Effect/Handler fixtures: at least 100 lines, at least one lambda expression,
  unique flows, every flow body at or above 10 lines, and at least three
  inter-flow call links.
- `fixtures/compiler/projects/positive/` contains multi-file application
  workspaces. Each project has an `etas.toml`, an `etas.lock`, a `src/` tree,
  an `app.main` entry module, and cross-module imports. Project manifests must
  follow the Package Management SPEC: `[package]` name/version/edition,
  `[source] root = "src"`, `[dependencies] std = { version = "0.1" }`, and a
  `[[bin]]` entry selecting `app.main.main`. Source files import only logical
  module paths; raw URLs and filesystem paths belong in package metadata. This
  corpus must stay at exactly 25 project-level fixtures unless the test is
  updated: ordinary application projects plus package dependency fixtures for
  exported `alias` and nominal `type` behavior, and effectful application
  projects covering `std.io`, `Console`, checked-index `Error[IndexError]`,
  `Agentic` effect rows, and `agent` declarations.
- `fixtures/compiler/incremental/` contains large project-level compiler
  workspaces for frontend incremental compilation acceptance. This corpus must
  contain at least three independent projects, each with at least 1000
  non-empty, non-comment `.es` source lines under `src/`, and edit variants for
  body-only reuse, import graph invalidation, module part merging, dependency
  import resolution, diagnostics replacement, and snapshot/delta refresh.
- `fixtures/compiler/projects/negative/` contains multi-file project fixtures
  for module graph, visibility, duplicate declaration, import-resolution, and
  Package Management diagnostics. Package-level negative fixtures cover raw
  dependency imports in source, manifest entry validation, manifest source-root
  handling, ambiguous dependency import roots, and project-level effect row
  escape diagnostics through public `etas check`.
- `fixtures/compiler/support/` contains public helper modules imported by
  compiler fixtures, so import/name-resolution tests do not rely on dangling
  helper names.

The syntax fixture corpus is intentionally broad and must stay at or above 100
`.es` files. `syntax_boundary.rs` enforces that count and parses the whole
syntax corpus.

Compiler algorithm fixtures follow the current command-line entry shape from
the language SPEC:

```etas
flow main(args: Array[string]) -> i32
```

They also follow the current module/import layout:

```etas
module tests.compiler.algorithms.example;

import std.collections.List;
import std.io.{read_all, println};

flow main(args: Array[string]) -> i32
```

Imports must appear immediately after the optional `module` declaration and
before any item declaration. Algorithm fixtures intentionally exercise explicit
item imports, grouped imports, and support-module imports so compiler tests cover
the SPEC's module boundary instead of relying on implicit global names.

Functional compiler fixtures use the same command-line and module/import
contract as algorithm fixtures, but their coverage target is Flow's
functional-language surface: flow value types such as `i32 -> i32`, anonymous
flows with `=>`, higher-order calls, returned closures, and pipeline
composition with `|`.

Test-file convention:

- `syntax_boundary.rs` uses only `etas_syntax` public APIs and checks syntax
  behavior without semantic assertions.
- `hir_boundary.rs` starts at AST-to-HIR lowering and checks symbols, scopes,
  resolution, name diagnostics, and HIR dumps without type/effect/AIR claims.
- `cli_boundary.rs` checks command routing, stdout/stderr policy, output
  layering, and stable exit codes without owning compiler semantics.
- `compiler_algorithms.rs` checks common algorithm fixture coverage,
  command-line I/O metadata, current entry-shape conformance, module/import
  ordering, absence of agent calls, syntax validity for positive algorithms, and
  targeted metadata for negative algorithm regressions.
- `compiler_functional.rs` checks functional fixture coverage, command-line I/O
  metadata, current entry-shape conformance, module/import ordering, absence of
  agent/prompt/refinement syntax, syntax validity, and the expected
  higher-order functional forms.
- `compiler_effect_policy.rs` checks current Effect/Handler and Policy fixture
  coverage, rejects obsolete source syntax such as source-level `Capability`,
  source-level `Sandbox`, old bracketed policy forms, and `prompt` keyword
  usage, enforces fixture complexity and function similarity bounds, validates
  public `etas effects --format json` summaries for positive effect entries,
  and runs every end-to-end fixture entry through public `etas check` compiler
  behavior.
- `compiler_handlers.rs` checks focused Handler fixtures through public
  `etas check`, rejects fake std/effect registries and obsolete try/catch
  syntax, verifies that fixture names match the handler SPEC surface they claim
  to cover, and requires negative diagnostics to include targeted codes plus
  stable message fragments.
- `compiler_projects.rs` checks project-level application fixtures as full
  workspaces through `etas check --all`, including manifest and lockfile shape,
  multi-file module graphs, cross-module imports, public APIs, package
  management rules, project-level effect propagation, and project-level
  diagnostics.
- `compiler_incremental_fixtures.rs` checks the large incremental compiler
  fixtures through public CLI workspace checks and `etas_frontend::FrontendSession`
  incremental changes, without depending on frontend private modules.
