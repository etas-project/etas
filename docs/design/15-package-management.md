# Package Management

## 1. Goal

Package management is part of the Etas language specification at the project
and tooling boundary. It should not add source-language keywords. Source files
use logical `module` and `import` paths; package manifests, lockfiles, package
metadata, and registry configuration decide where those logical paths come from.

This separation keeps source code stable across local packages, registries,
GitHub dependencies, private registries, generated modules, and compiler-known
standard-library modules.

## 2. Source Language Boundary

Etas source code imports logical module paths:

```etas
module app.main;

import std.io.*;
import company.agents.Writer;
import web.search.SearchClient;
```

The source language does not support raw dependency locations:

```etas
// Not source syntax.
import "https://github.com/org/package";
import "../local/file.es";
```

Dependency sources, versions, aliases, registry URLs, Git revisions, and local
paths belong in `etas.toml` and `etas.lock`, not in `.es` source files.

## 3. Package Layout

The default package layout is:

```text
my-agent-app/
    etas.toml
    etas.lock
    src/
        app/main.es
        app/pipeline.es
    tests/
    examples/
```

If `src/app/main.es` declares:

```etas
module app.main;
```

then the default resolver maps the logical module path to one of:

```text
src/app/main.es
src/app/main/mod.es
```

If both files exist and both define `module app.main`, the package is ambiguous
and must be rejected.

## 4. Package Manifest

An Etas package is described by `etas.toml` at the package root.

```toml
[package]
name = "research-assistant"
version = "0.1.0"
edition = "2026"
license = "MIT"
description = "A multi-agent research assistant"
repository = "https://github.com/org/research-assistant"

[source]
root = "src"

[dependencies]
std = { version = "0.2" }
company_agents = { package = "company-agents", version = "1.2", import = "company.agents" }
web_search = { package = "web-search", version = "0.4", import = "web.search" }
local_prompts = { path = "../local-prompts", import = "prompts" }
github_tools = { git = "https://github.com/org/etas-tools.git", tag = "v0.3.0", import = "tools" }

[[bin]]
name = "research-assistant"
module = "app.main"
flow = "main"
```

Manifest fields:

| Field | Meaning |
|---|---|
| `[package].name` | Registry package name. |
| `[package].version` | Package version using SemVer. |
| `[package].edition` | Language edition used for parsing and semantic rules. |
| `[source].root` | Source root for the current package. Defaults to `src`. |
| `[dependencies]` | Package dependencies from registry, local path, Git, or built-in source. |
| `[[bin]]` | Runnable entry points. Each entry names a module and a flow. |
| `[reflection].retain_bodies` | Optional list of fully qualified items in this package whose source bodies are retained; defaults to `[]`. |
| `[reflection].export_bodies` | Optional list of public items in this package whose bodies may be reflected across packages; implies retention and defaults to `[]`. |

Reflection lists contain logical item names, not paths. Entries must resolve to
items owned by the package; `export_bodies` cannot expose a private item or a
dependency's unexported source. `public` visibility alone does not export a body.
See [reflection configuration](18-reflection-and-checked-code-generation.md#33-code-representation-and-visibility)
for the configuration example and runtime-revision rules.

A dependency key is a local manifest name. It does not need to match the
registry package name or import root:

```toml
[dependencies]
company_agents = { package = "company-agents", version = "1.2", import = "company.agents" }
```

This lets source code import:

```etas
import company.agents.Writer;
```

If `import` is omitted in a dependency entry, the dependency key is used as the
default import root.

## 5. Dependency Sources

Etas should support these dependency source forms:

```toml
[dependencies]
std = { version = "0.2" }
local = { path = "../local-package", import = "local" }
tools = { git = "https://github.com/org/tools.git", rev = "abc123", import = "tools" }
tools_tag = { git = "https://github.com/org/tools.git", tag = "v0.3.0", import = "tools" }
tools_branch = { git = "https://github.com/org/tools.git", branch = "main", import = "tools" }
private = { registry = "company", package = "agent-tools", version = "2.0", import = "company.tools" }
```

Rules:

- registry dependencies resolve through a configured registry index;
- path dependencies resolve relative to the package manifest;
- Git dependencies must be locked to an exact revision in `etas.lock`;
- built-in packages such as `std` may be compiler-known, but they still expose
  normal package metadata;
- source code only sees logical import roots, not dependency source forms.

The [compiler-recognized std contracts](01-overview.md#4-language-and-library-boundaries)
bind to resolved standard package/item identities, not local import roots or
unqualified names. Aliases and re-exports preserve recognition; a different
package or local declaration with the same spelling does not acquire it.
Standard declarations and algorithms may be Etas source. Compiler recognition
does not require a bodyless host implementation for every std export.

## 6. Version Requirements

Etas package versions use SemVer. The package manager should support:

```toml
[dependencies]
std = { version = "0.2" }          # compatible with 0.2.x
foo = { version = "^1.2.0" }       # >=1.2.0, <2.0.0
bar = { version = "=1.2.3" }       # exactly 1.2.3
baz = { version = ">=1.2,<1.5" }
```

The exact selected versions are recorded in `etas.lock`.

## 7. Lockfile

`etas.lock` is generated by tooling. Application packages should commit it so
builds are reproducible. Library packages may commit it, but public dependency
constraints still come from `etas.toml`.

Example:

```toml
version = 1

[[package]]
name = "std"
version = "0.1.4"
import_root = "std"
source = "builtin"
checksum = "builtin:std"

[[package]]
name = "company-agents"
version = "1.2.3"
import_root = "company.agents"
source = "registry+https://packages.etas.dev"
checksum = "blake3:..."
dependencies = ["std"]

[[package]]
name = "web-search"
version = "0.4.1"
import_root = "web.search"
source = "git+https://github.com/org/web-search.git?rev=abc123"
checksum = "blake3:..."
dependencies = ["std"]
```

Each lockfile entry records the resolved `import_root` that source imports use
for that package in this package graph. The value comes from the dependency
entry's `import` field, or from the dependency key when `import` is omitted.

The frontend cache key must include the resolved lockfile entries and package
metadata fingerprints for all dependencies that affect type checking, effect
checking, policy checking, generated tool schemas, and public API resolution.

### 7.1 Using the installed standard library

The selected standard library must satisfy the manifest version requirement
and locked identity. Its recognized symbols and public contracts must match
the selected execution target; mismatch is a diagnostic, not a silent fallback.

Changing a standard-library dependency requires explicit lock resolution.
Installation paths, artifact formats, caches and provider-selection commands
are toolchain documentation, not language semantics.

## 8. Exported Contracts

Dependencies checked from source or metadata must expose the same public
semantics. File formats and compiler pipelines are outside this SPEC.

| Contract | Required Information |
|---|---|
| Identity and visibility | Package identity/version, logical modules, public items and expanded re-exports. |
| Types and callables | Type/spec signatures, generic constraints, parameter names/defaults, receiver modes and transitive suspension facts. Unknown suspension is not non-suspension. |
| Constructors | Field order, types/visibility, resolved `@inject` action identities/static selectors and omitted-field behavior; imports do not rebind provider names. |
| Active agents | State schema, dispatch/role evidence, scoped-reference/capture bounds and temporal/state-dependence contracts. |
| Effects and tools | Escaping and requested-action summaries, model schemas, required concrete bindings and trace constraints. |
| Defaults | Definition-site bindings and revision identity; callers do not reinterpret default expressions. |
| Reflection | Body retention/export visibility and source/dependency fingerprints. Retained private bodies remain private. |
| Standard symbols | Recognized symbol identities and compatible public contracts; package metadata cannot self-assign intrinsic status or authority. |

Inferred public signatures are permitted, but the resulting contracts must be
available to dependents. `requested_actions` and `default_actions` describe
analysis metadata, not source declaration clauses.

## 9. Import Resolution

Resolving:

```etas
import company.agents.Writer;
```

uses the package graph:

```text
1. Search current package modules.
2. Search dependency import roots from etas.toml.
3. Choose the longest matching import prefix.
4. Reject ambiguous matches.
5. Resolve the result to PackageId + ModuleId + ItemId.
```

For:

```toml
[dependencies]
company_agents = { package = "company-agents", version = "1.2", import = "company.agents" }
```

the import lowers to a stable identity:

```text
PackageId(company-agents@1.2.3) + ModuleId(company.agents) + ItemId(Writer)
```

Item identity is independent of physical source location.

## 10. Workspace

A multi-package repository may define an `etas.workspace.toml`:

```toml
[workspace]
members = [
    "packages/core",
    "packages/web",
    "examples/*"
]

[workspace.dependencies]
std = { version = "0.2" }
```

Workspace configuration is tooling-level metadata. It does not change source
language semantics. A workspace dependency gives member packages a shared
dependency constraint when a member chooses to inherit it.

## 11. Registry And User Configuration

User-level package manager configuration belongs outside project source, for
example:

```text
~/.etas/config.toml
```

Example:

```toml
[registries.default]
index = "https://packages.etas.dev/index"

[registries.company]
index = "https://packages.company.com/etas/index"

[cache]
root = "~/.etas/cache"
```

Authentication tokens must not be written into `etas.toml` or `etas.lock`.
They should be loaded from environment variables, the system keychain, hosted
runtime configuration, or Etas secret references.

## 12. Tool Bindings

A package may expose a bodyless tool signature:

```etas
public tool search(query: SearchQuery) -> SearchResults ![AcademicSearch.search];
```

The signature is a typed source-level contract. A runnable application must still
resolve it to a concrete host binding through package metadata or runtime
configuration:

```toml
[bindings.tools]
"web.search.search" = { kind = "mcp", server = "browser", tool = "search" }
```

Rules:

- a tool with an Etas body does not need an external binding;
- a bodyless tool in a dependency may be type-checked from package metadata;
- a final runnable package must resolve all bodyless tool signatures reachable
  from its entry points;
- tool bindings must declare the effect/action row they may perform, and that
  row must be included in the source-level tool signature;
- binding configuration is not source syntax.

## 13. Project Checking

A package is checked using its manifest, locked dependencies, source root and
selected entries. Source or metadata dependencies have identical obligations;
checking does not execute the selected entry.

### 13.1 Module Membership

The source root defaults to `src`:

```toml
[source]
root = "src"
```

Within that root, source files define logical modules:

```text
src/app/main.es      -> module app.main;
src/app/main/mod.es  -> module app.main;
src/app/main/cli.es  -> module app.main;
```

Rules:

- every package source file must declare exactly one `module`;
- the declared module path must match the path selected by the resolver;
- multiple source files may contribute to the same logical module;
- `src/app/main.es` and `src/app/main/mod.es` are competing canonical roots
  for `module app.main`, so using both in the same package is rejected;
- all parts of the same logical module share one top-level namespace, so
  duplicate top-level declarations in the merged module are rejected;
- imports are local to the source file that declares them; `public import` from
  any part contributes to the merged module's public API;
- relative filesystem imports are rejected;
- generated or virtual modules are allowed only when the package resolver maps
  them to stable logical module identities.

### 13.2 Entry Points

Runnable targets are declared in `etas.toml`:

```toml
[[bin]]
name = "research-assistant"
module = "app.main"
flow = "main"
```

The selected entry must resolve to a flow compatible with:

```etas
flow main(args: Array<string>) -> i32
```

The compiler may infer the return type and effect row of `main`, but the final
entry signature must have an `i32` return type. The inferred effect/action row of
the entry becomes the authority summary for deployment and runtime grant checks.

A package may define multiple runnable entries. Each entry is checked
independently, but they share the same package graph and module index.

### 13.3 Library And Executable Boundaries

| Target | Required Checks |
|---|---|
| Library | Name resolution, visibility, type/effect/spec contracts and exported constructor/default/capture contracts; an entry point is not required. |
| Executable | Library checks plus a valid selected entry and compatible concrete definitions/bindings for reachable bodyless tools. Required actions remain subject to execution authority. |
| Isolated file | May provide local diagnostics, but cannot establish project-wide imports, entry points or external bindings without the package environment. |

Dependency import roots and ambiguity follow section 9. The SPEC does not
prescribe compiler phase order, intermediate representations or artifact formats.
