# Etas Language Specification

This directory defines the language and its public library contracts.
Compiler architecture, IR design, implementation plans and framework surveys
are outside its scope. A specified feature is not an implementation-status claim.

| Chapter | Contents |
|---|---|
| [Overview](01-overview.md) | Constructs, keywords and language/library boundaries |
| [General Constructs](02-general-programming-constructs.md) | Bindings, values, calls, initialization, modules and imports |
| [Agents And Tools](03-agents-tools-prompts-memory.md) | Active instances, requests, model boundaries, context and resources |
| [Flows And Resource Bounds](04-flows-human-gates-and-protocols.md) | Composition, loops, limits, human gates and protocols |
| [Types And Errors](05-type-system-and-errors.md) | Nominal/structural types, specs, existentials and `?` |
| [Effects And Handlers](06-effect-system-and-inference.md) | Actions, rows, inference, handlers and authority boundaries |
| [Examples](07-examples.md) | Small examples of the normative constructs |
| [Source Grammar](09-syntax-principles.md) | EBNF and syntax references |
| [Safety Contracts](14-safety-analysis-static-dynamic.md) | Trace constraints, scope composition, rejection and dynamic checks |
| [Packages](15-package-management.md) | Manifest, dependency identity, lockfile and project boundaries |
| [Concurrency](16-concurrency.md) | Structured tasks, handlers, cancellation and active requests |
| [Reflection Extension](18-reflection-and-checked-code-generation.md) | Definition access and checked code revision |
| [Standard Library](19-standard-library.md) | Public API tables and behavior contracts |
| [Lexical Capture](20-lexical-capture.md) | Lambda/handler capture, mutation and lifetime |

Normative details live in one chapter and are linked elsewhere. Examples
illustrate contracts; they do not add implicit APIs or permissions.
