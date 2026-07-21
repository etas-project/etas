# Syntax Principles

## 1. Etas Source Grammar

This section sketches the current user-facing Etas source syntax. It is intentionally separate from the internal core calculus in [Formal Core, Static Analyses, and PL Context](08-formal-core-static-analyses-and-pl-context.md).

String escapes and numeric literal suffixes are omitted here.

Source examples use **four spaces per indentation level**. Tabs are not used for indentation.

Comments are lexical trivia. They are ignored by parsing, do not enter the typed AST or AIR, and do not affect effects, policies, limits, traces, or runtime behavior.

Etas supports line comments and block comments:

```etas
// Line comments run until the end of the line.

let draft = Writer.run(topic); // Trailing comments are allowed.

/*
Block comments can span multiple lines.
They are useful for documentation near declarations.
*/
```

Block comments are not nested in the MVP. If a block comment contains `/*`, it is treated as ordinary text until the first following `*/`.

Comments must not be used as semantic annotations. A trace spec, effect/action boundary, approval requirement, or loop budget must be expressed in source constructs:

```etas
// This comment does not enforce a budget.
while needs_revision(draft) {
    draft = Rewriter.run(draft);
}

// This source construct does.
while needs_revision(draft)
    limit Iterations(3), Tokens(30_000)
{
    draft = Rewriter.run(draft);
}
```

### 1.1 Modules And Items

```ebnf
Program       ::= ModuleDecl? ImportDecl* Item*

ModuleDecl    ::= "module" Path ";"
ImportDecl    ::= Visibility? "import" ImportTree ";"
ImportTree    ::= Path ImportTail?
ImportTail    ::= "as" Ident
                | ".*"
                | ".{" ImportTreeList "}"
ImportTreeList
              ::= ImportTreeEntry ("," ImportTreeEntry)* ","?
ImportTreeEntry
              ::= Ident ImportTail?

Item          ::= Annotation* Visibility? ExportableItemDecl
                | Annotation* ImplDecl

Annotation    ::= "@" Path AnnotationArgs?
AnnotationArgs
              ::= "(" AnnotationArgList? ")"
AnnotationArgList
              ::= AnnotationArg ("," AnnotationArg)* ","?
AnnotationArg ::= Expr
                | Ident "=" Expr

ExportableItemDecl
              ::= AliasDecl
                | TypeDecl
                | EnumDecl
                | SpecDecl
                | TopLevelLetDecl
                | EffectDecl
                | ToolDecl
                | AgentDecl
                | ProtocolDecl
                | FlowDecl
```

Import examples:

```etas
import std.io;
import std.io as io;
import std.io.println;
import std.io.println as log;
import std.io.{print, println, eprintln};
import std.io.{println as log, read_line};
import std.io.*;
public import std.prelude.*;
import company.memory.ProjectMemory;
import company.memory.ProjectMemory as PM;
```

Top-level declarations default to `private`. `public import` is a re-export.
Wildcard imports are allowed, but they only import public names. If multiple
wildcard imports provide the same unqualified name, the use is ambiguous unless
the program adds an explicit import or uses a qualified path.

`Path` in a module or import declaration is a logical module path, not a raw
filesystem path. The package resolver maps logical paths to files by convention
such as `src/foo/bar.es` or `src/foo/bar/mod.es`, then records stable
package/module/item identities for later phases.

Annotations are item-level static metadata. They are written before the item
they describe:

```etas
@model("gpt-5.5-thinking")
@tools([repo.read, latex.check])
@limits([ContextTokens(32_000), Attempts(2)])
agent Reviewer(input: Draft) -> Review {
    return perform infer<Review>(build_review_prompt(input));
}

@deprecated("use std.fs.read_bytes")
flow old_read(path: Path) -> bytes {
    return std.fs.read_bytes(path);
}

@derive([Schema, PromptEncode])
type Review = {
    summary: string,
    issues: Array<string>,
}
```

Annotation arguments must be statically evaluable: literals, paths, arrays,
records, named arguments, and compile-time constants are allowed; expressions
that perform effects or depend on runtime state are rejected. A small set of
compiler-known annotations may affect checking or lowering, such as `@model`,
`@tools`, `@limits`, `@derive`, `@test`, `@deprecated`, `@trace`, and
`@optimization`. User-defined annotations are metadata by default: they can be
read by package, documentation, test, and IDE tooling, but they do not change
type checking, effect inference, trace-spec checks, or runtime authority unless the
SPEC explicitly promotes them to compiler-known annotations.

Annotation reflection is static-first. Package, build, test, documentation, and
IDE tools may read annotation metadata from compiler outputs. The MVP should not
expose unrestricted runtime reflection over value layouts, private fields,
effect summaries, or trace-spec internals.

Annotations never grant authority. For example, `@tools([email.send])` only
defines the model-visible tool surface. The tool's effect summary, the active
effect boundary, active trace specs, handlers, sandbox, limits, and runtime checks still
decide whether a concrete tool call can execute.

### 1.2 Types

```ebnf
AliasDecl     ::= "alias" Ident TypeParams? "=" TypeExpr ";"

TypeDecl      ::= "type" Ident TypeParams? ";"
                | "type" Ident TypeParams? "=" TypeExpr ";"
                | "type" Ident TypeParams? "=" RecordType

EnumDecl      ::= "enum" Ident TypeParams? "{" EnumVariant* "}"
EnumVariant   ::= Ident ("(" TypeList? ")")? ";"

SpecDecl     ::= "spec" Ident TypeParams? SpecKindAnn? SpecDeclTail
SpecKindAnn  ::= ":" SpecKind
SpecKind     ::= "type" | "callable" | "trace"
SpecDeclTail ::= SpecEntailment? (";" | SpecBlock)
                | TypeExpr "=>" TypeExpr EffectSuffix? ";"
                | "=" SpecExpr ";"
SpecEntailment
              ::= "~" BoundList
SpecBlock    ::= "{" SpecItem* "}"
SpecItem     ::= Visibility? SpecFlowSig
SpecFlowSig  ::= "flow" Ident TypeParams? ParamList ReturnType ";"

SpecExpr     ::= SpecOr
SpecOr       ::= SpecAnd ("|" SpecAnd)*
SpecAnd      ::= SpecUnary ("&" SpecUnary)*
SpecUnary    ::= ("+" | "-") ActionPattern
                | SpecTemporal
SpecTemporal ::= SpecPrimary ((">>" | "<<") SpecPrimary)?
SpecPrimary  ::= SpecRef
                | ActionPattern
                | "(" SpecExpr ")"
ActionPattern::= Path StaticEffectArgs?

ImplDecl      ::= TypeImplDecl
                | SpecImplDecl
                | AgentImplDecl
TypeImplDecl  ::= "impl" ImplTarget "{" ImplItem* "}"
SpecImplDecl ::= "impl" TypeExpr SpecSatisfaction (";" | "{" SpecImplItem* "}")
AgentImplDecl ::= "impl" "agent" Path SpecSatisfaction? (";" | "{" AgentImplItem* "}")
ImplTarget    ::= Path TypeArgs?
ImplItem      ::= Visibility? FlowDecl
                | EffectActionDecl
SpecImplItem ::= Visibility? FlowDecl
AgentImplItem ::= Visibility? FlowDecl

TypeExpr      ::= HandlerType
                | ArrowType

ArrowType     ::= PrimaryType "->" ArrowType EffectSuffix?
                | PrimaryType

HandlerType   ::= "!" "[" EffectList HandlerTail? "]"
HandlerTail   ::= HandlerProduced? HandlerResult?
HandlerProduced
              ::= "=>" HandlerOutput
HandlerOutput ::= EffectList | "[" "]"
HandlerResult ::= "for" TypeExpr

PrimaryType   ::= PrimitiveType
                | ExistentialType
                | Path TypeArgs?
                | RecordType
                | TupleType
                | "(" TypeExpr ")"

ExistentialType
              ::= "?" "~" BoundList

PrimitiveType ::= "bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                | "f32" | "f64" | "char" | "string" | "bytes" | "unit" | "never"

RecordType    ::= "{" FieldDecl* "}"
FieldDecl     ::= Visibility? Ident ":" TypeExpr ","?
Visibility    ::= "private" | "public"

TupleType     ::= "(" TypeList ")"
TypeList      ::= TypeExpr ("," TypeExpr)* ","?
TypeParams    ::= "<" TypeParam ("," TypeParam)* ","? ">"
TypeParam     ::= Ident TypeParamBound?
                | "effect" Ident TypeParamBound?
TypeParamBound::= "~" BoundList
BoundList     ::= Bound ("+" Bound)*
Bound         ::= SpecRef
TypeArgs      ::= "<" TypeList ">"

SpecSatisfaction
              ::= "~" SpecRef
SpecRef      ::= Path TypeArgs?
DeclarationConformance
              ::= "~" DeclarationConformanceRef
DeclarationConformanceRef
              ::= Path TypeArgs?
                | "(" SpecExpr ")"

EffectSuffix  ::= "!" EffectRow
EffectRow     ::= "[" EffectList? "]"
EffectList    ::= EffectRef ("," EffectRef)* ","?
EffectRef     ::= Path StaticEffectArgs?
StaticEffectArgs
              ::= "<" StaticEffectArgList? ">"
StaticEffectArgList
              ::= StaticEffectArg ("," StaticEffectArg)* ","?
StaticEffectArg
              ::= TypeExpr
                | Literal
                | StableResourcePath
                | ConstPath
                | "_"
```

Delimiter roles are intentionally separated:

- `<...>` is used for type-level, spec-level, effect-level, and action-level
  parameterization, including `Result<T, E>`, `Stage<I, O, E>`,
  `Error<E>`, and `Fs.read<R>`.
- `![...]` is the only bracketed form for effect rows and handler types.
- `[...]` remains expression syntax for arrays, indexing, slicing, and value
  lists passed to annotations such as `@tools([repo.read])`.

Effect/action `<...>` arguments are static selectors, not runtime argument
captures. They may name types, stable resource paths, literals, compile-time
constants, or `_`. A local variable such as `path`, `stream`, `topic`, or
`req.host` is rejected in an effect row or action reference unless it resolves
as a compile-time constant. Runtime values are carried by the action call
payload and recorded in trace events.

`alias A = B;` is a transparent abbreviation and does not create a new type
identity. `type A = B;` creates a nominal type with representation `B`. `type
A;` creates a bodyless nominal type used either as a zero-runtime marker or as
an opaque runtime handle:

```etas
alias JsonText = string;

type UserId = string;
type ReportsRoot;
type TcpStream;
```

Assignment between `type UserId = string` and `string` requires an explicit
constructor, accessor, or conversion flow. `ReportsRoot` is a marker if it is
only used in type indices and spec relations; `TcpStream` is an opaque runtime
handle if values of that type are returned by APIs and passed at runtime.

`spec` declares static constraints. Etas MVP has three spec kinds:
`type`, `callable`, and `trace`. Type specs constrain types and type-level
entities such as resource markers and effect rows. Callable specs constrain
flows, tools, and generated agent methods. Trace specs constrain requested
action traces, authority, and temporal relations. `:` is reserved for type
annotations and optional spec-kind annotations, while `~` means spec satisfaction
or spec entailment. `callable` and `trace` are contextual spec-kind words in the
MVP: they are only reserved after `spec Name:` and are not counted as global
source keywords.

Type specs can be marker specs, relation specs, behavioral specs, or specs
that entail other specs:

```etas
public spec ByteStream;
public spec Region;
public spec Within<Parent>;
public spec ReadableStream ~ ByteStream;
public spec WritableStream ~ ByteStream;
public spec ReadWriteStream ~ ReadableStream + WritableStream;

public spec PromptEncode {
    flow encode(self) -> PromptPart;
}

impl TlsStream ~ ByteStream;
impl ReportsRoot ~ Region;
impl ReportsRoot ~ Within<WorkspaceRoot>;

impl Report ~ PromptEncode {
    flow encode(self) -> PromptPart {
        return PromptPart.data(self);
    }
}
```

Callable specs are bodiless computation-shape specs:

```etas
public spec Stage<I, O, effect E>: callable I => O ![E];
public spec Pure<I, O>: callable I => O ![];
public spec ReportWriter: callable Brief => Draft ![WriterEffects];
```

Names used by a callable spec body are not implicitly generic. Generic variables
must be declared in `<...>`; otherwise names such as `Brief`, `Draft`, and
`WriterEffects` must resolve to existing types or effect rows.

If a callable spec omits `![...]`, it imposes no effect constraint. The
implementation still has its own inferred effect row, but the spec does not
restrict it. To require an effect-free computation, write `![]` explicitly, as
in `Pure<I, O> I => O ![]`.

Trace specs are compile-time spec expressions over action patterns:

```etas
public spec SafeEmail: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & -Secret.read<_>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

Spec bounds appear on type parameters with `~`:

```etas
flow render<T ~ PromptEncode>(value: T) -> PromptPart;
flow read_report<R ~ Region + Within<ReportsRoot>>(path: WorkspacePath<R>) -> bytes;
flow wrap<T, effect E>(f: () -> T ![E]) -> T ![E];
```

Flow declarations, tool declarations, and generated agent entrypoints may
satisfy callable specs. The `agent` declaration itself is a nominal component,
not a flow value. Its generated `Name.run` entrypoint is the value that
participates in callable specs such as `Stage<I, O, E>`. If the spec arguments
are inferable from the declaration signature, the implementation may write only
the spec name:

```etas
flow Normalize(text: string) -> string ~ Pure {
    return std.text.trim(text);
}
```

`Normalize` is checked as `Pure<string, string>`; users do not need to write the
inferred arguments. If inference is ambiguous, the compiler requires an
explicit instantiation such as `~ Stage<Brief, Draft, WriterEffects>`.

Spec operators are kind-checked. `&` may combine specs of the same kind. `|` is
reserved for trace specs in the MVP. Prefix `+`, prefix `-`, `>>`, and `<<` only
construct trace specs; applying them to type specs or callable specs is rejected.

Spec satisfaction does not create subtyping. A value of type `TlsStream` is not
assigned to a concrete `ByteStream` value; it is accepted by APIs that quantify
over `S ~ ByteStream`. Likewise, `WorkspacePath<DraftsRoot>` is not a subtype
of `WorkspacePath<ReportsRoot>`; it is accepted only by generic APIs whose
bounds are satisfied by imported spec evidence such as `impl DraftsRoot ~
Within<ReportsRoot>;`.

When a type satisfies multiple behavioral specs with overlapping method names,
source code can select a specific spec method with `::`:

```etas
let trusted = prompt::PromptEncode.encode();
let bytes = packet::WireEncode.encode();
```

`value::Spec.method(args)` means "call `method` using the implementation evidence
for `value ~ Spec`". The receiver is the value before `::`; it is not repeated as
the first positional argument. This form is for behavioral spec methods.
Callable specs such as `Stage<I, O, E>` are called through the flow, tool, or
generated agent entrypoint value itself.

Existential spec objects use `? ~ Spec` in type position. They hide the concrete
implementation type while preserving spec evidence:

```etas
let stream: ? ~ ByteStream = tls;
let streams: Array<? ~ ByteStream> = [tcp, tls, file];

flow read_one(stream: ? ~ ByteStream) -> bytes {
    return std.stream.read_until_limit(stream, ByteLimit(4096), Timeout.ms(1000));
}
```

The first `?` is a type-position existential marker. It is unrelated to the
postfix expression form `e?`, which captures `Error<E>` into a `Result<T, E>`.
`?` cannot appear alone; it must be followed by `~` and one or more type specs,
for example `? ~ ReadableStream + WritableStream`.

Array and list literals remain homogeneous. The example above has element type
`? ~ ByteStream`; each element is packed into the existential type because the
expected array type is explicit. Without that context, `[tcp, tls, file]` is
rejected rather than inferred as a hidden common supertype.

Arrow types are the source-level notation for flow values. The compiler normalizes them to the internal `Flow<I, O, E>` representation. `A -> B -> C` is right-associative and means `A -> (B -> C)`. An explicit effect row is written after the output type with `![...]`:

```etas
Url -> Page
Url -> Page ![Network]
Topic -> Draft ![Http.request<_>]
(A, B) -> C
```

An effect row accepts a tag, an action, or a parameterized action instance:

```etas
![Network]
![AcademicSearch.search]
![Http.request<"github.com">]
![ProjectWorkspace.write<"reports/**">]
```

`Network` is the broad form and covers actions under the `Network` effect root,
including imported package-defined actions that extend it. Etas does not
support `Network.*` in effect rows; use `Network` for the whole tag or an action
pattern such as `Http.request<_>` for a narrower boundary. Effect rows are positive
summaries only, so `deny` is not valid inside `![...]`; rejection belongs in
trace specs.

`![...]` without `=>` in a computation type is an effect row. In a type annotation by itself, `![...]` may also describe a handler effect transformer:

```text
![H]             == ![H => _]
![H for R]       == ![H => _ for R]
![H => E]        == ![H => E]
![H => E for R]  == ![H => E for R]
```

`H` is the handled action/effect set. `_` means the handler's produced effects
are inferred from handler arms. `E` is an explicit upper bound on those produced
effects. `for R` pins the answer type of the handled expression when a handler
arm uses `finish` instead of resuming:

```etas
![Approval]
![Approval => Console.stdout_write]
![Error<AppError> for Report]
![Error<AppError> => [] for Report]
```

The `flow` keyword is used for named declarations, not for type expressions.

```ebnf
EffectDecl    ::= "effect" Ident TypeParams? ("extends" Effect)? (";" | EffectBlock)
EffectBlock   ::= "{" EffectActionDecl* "}"
EffectActionDecl
              ::= "action" Ident TypeParams? ParamList ReturnType ";"
```

### 1.3 Top-Level Bindings

```ebnf
TopLevelLetDecl
              ::= "let" Ident TypeAnnot? "=" Expr ";"
TypeAnnot     ::= ":" TypeExpr
```

Top-level `let` is restricted. It may define immutable constants and named
runtime resource handles created by compiler-known standard resource
constructors such as `std.memory.region<...>`. Top-level `var` is not part of
the language, and top-level `let` initializers must not perform ordinary runtime
effects such as tool calls, agent calls, time reads, or memory reads/writes.
Handler literals are allowed in top-level `let` declarations because they create
handler values; they do not execute the handled computation.

Top-level `let` declarations are module items. If marked `public`, a resource
handle can be imported by another module. Import aliases are name-resolution
aliases only; they do not allocate a new resource handle or create a new memory
backend identity.

### 1.4 Flows

```ebnf

FlowDecl      ::= "flow" Ident TypeParams? ParamList ReturnType? FlowClause* FlowBody FlowHandler?
FlowBody      ::= Block
                | "=" Expr ";"?
FlowHandler   ::= "with" HandlerArg
ReturnType    ::= "->" TypeExpr EffectSuffix?

ParamList     ::= "(" ParamListInner? ")"
ParamListInner::= Param ("," Param)* ","?
Param         ::= Ident ":" TypeExpr

FlowClause    ::= DeclarationConformance
```

`flow` is the ordinary user-defined callable declaration in the source language. The compiler infers the return type, effect row, and determinism class: `Deterministic` or `NonDeterministic`. This applies to every flow, including `main`. Local declarations usually omit `![...]`; when a flow writes an explicit effect row, that row is checked as an upper-bound constraint on the inferred body effects. Etas-implemented `tool` declarations also have bodies, but they are model-callable boundaries with extra schema, trace-spec, effect/action, and output-safety rules. Exported APIs may write explicit effects or rely on generated package metadata.

`~` is the declaration conformance operator. On declarations, the right-hand side
is resolved by spec kind:

```text
T ~ ByteStream                 => type spec conformance
flow f(...) ~ StageSpec        => callable spec conformance
flow f(...) ~ SafetyTraceSpec  => trace spec conformance over requested actions
flow f(...) ~ (+A & -B)        => inline trace spec conformance
```

Specs are kinded. Etas MVP has three spec kinds:

| Kind | Constrains | Examples |
|---|---|---|
| `type` | Types, type-level markers, resource markers, and effect-row/static entities | `ByteStream`, `Region`, `Within<Root>`, `ReadOnlyEffects` |
| `callable` | Flow, tool, and agent-method call shape | `Stage<I, O, E>`, `Pure<I, O>` |
| `trace` | Requested-action traces, authority, and temporal constraints | `SafeEmail`, `+Email.send`, `Approval.request >> Email.send` |

Effect-row specs are `type` specs because effect rows are static entities rather
than runtime values. Protocols are not a spec kind in the MVP; the optional
`protocol` declaration remains a separate advanced construct.

Spec expressions are kind-checked. `&` composes specs of the same kind. `|` is
reserved for trace specs in the MVP. Prefix `+`, prefix `-`, `>>`, and `<<`
construct only trace specs:

```text
+A      allow action pattern A
-A      deny action pattern A
A >> B  require A before B
A << B  require A after B, equivalent to B >> A
```

If a path can name more than one conformance kind in scope, the compiler rejects
the declaration as ambiguous and asks for a qualified path or a clearer name.

An expression-bodied flow is shorthand for a block whose final expression is
the same expression:

```etas
flow SearchOnce() -> Answer =
    handle Search() with ChooseFirst;
```

This declares a named flow. A bare top-level `handle Search() with ChooseFirst`
expression is not a module item and is not executed during module loading.

The return type may be omitted. In that case, the compiler infers it from explicit `return` statements and from the final expression, if present:

```etas
flow normalize_title(title: string) {
    trim(lowercase(title))
}
```

This is equivalent to:

```etas
flow normalize_title(title: string) -> string {
    return trim(lowercase(title));
}
```

Return type inference uses this rule:

```text
ReturnCandidates(flow body) =
  all `return expr` expressions
  + the final expression, if present
  + `unit` for each `return;`

Expressions of type `never` do not constrain the inferred return type.
All remaining candidates must unify.
If the candidate set is empty, the inferred return type is `unit`.
```

For example:

```etas
flow choose_writer(fast: bool) {
    if fast {
        return FastWriter;
    }

    return CarefulWriter;
}
```

is inferred as:

```text
flow choose_writer(bool) -> Outline -> Draft
```

An `impl` block can define inherent methods for an ordinary type, action
signatures for an effect tag, or evidence that a type satisfies a type spec.
The compiler must kind-check the target before checking the block body:

```text
impl TypeName { ... }              => only `flow` methods are allowed.
impl EffectName { ... }            => only `action` signatures are allowed.
impl TypeName ~ SpecName { ... }  => only required type-spec `flow` methods are allowed.
```

These forms must not be mixed. A type `impl` cannot contain `action`, an effect
`impl` cannot contain ordinary `flow` methods, and a type-spec `impl` cannot
declare extra methods outside the spec contract. When an `impl` targets an
ordinary type, its methods have bodies. When an `impl` targets an effect tag,
its `action ...;` entries have no bodies. Marker type-spec implementations may
use the short form `impl TypeName ~ SpecName;`. Callable specs are not
implemented with top-level `impl`; flows, agents, and tools satisfy them
directly in their declarations with `~ SpecName`. Effect actions can also be
declared directly inside an `effect` block. An effect action can be invoked with
`perform` and intercepted by `handle`.

### 1.4 Tools, Agents, Persistent Resources

```ebnf
ToolDecl      ::= "tool" Path ParamList "->" TypeExpr EffectSuffix? ToolClause* ToolBody
ToolBody      ::= Block | ";"
ToolClause    ::= DeclarationConformance

AgentDecl     ::= "agent" Ident AgentDeclTail
AgentDeclTail ::= ";"
                | ParamList "->" TypeExpr EffectSuffix? AgentClause* Block
AgentClause   ::= DeclarationConformance
```

Persistent agent state is represented with compiler-known standard/runtime support types such as `MemoryRegion<S>` and `Store<K, V>`, usually bound to an immutable top-level resource handle:

```etas
type ProjectMemorySchema = MemoryRegion<{
    Papers: Store<PaperId, PaperRecord>,
    Decisions: Store<DecisionId, DecisionRecord>,
}>;

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );
```

Store API calls are ordinary calls whose signatures carry parameterized, region-sensitive effects such as `Memory.read<ProjectMemory.Papers>` and `Memory.write<ProjectMemory.Decisions>`. The legal memory places, containment relation, and read/write inference rules are defined in [Effect System and Inference](06-effect-system-and-inference.md#11-parameterized-effects). Backend binding, namespace, retention, and migration policy belong in the manifest/runtime configuration, not in source syntax.

Agent runtime metadata uses annotations before the declaration. For example,
`@model("gpt-5.5")`, `@tools([latex.check])`, and
`@limits([ContextTokens(32_000)])` are compiler-known annotations; `model`,
`tools`, and `limits` are not keywords. The bare form `agent Name;` declares a
nominal agent component with identity, metadata, tool surface, trace-spec context, trace, and
runtime configuration, but no default callable entrypoint.

The ergonomic form:

```etas
agent Name(input: I) -> O ![E] {
    ...
}
```

is sugar for declaring the nominal agent and implementing its default `run`
method. The body is checked like a flow body, with one extra operation available:
`perform infer(prompt)`. The explicit `![E]`, when present, is an upper-bound
contract for ordinary escaping effects of the generated `run` method.

The `@tools(...)` annotation lists model-callable tool declarations. It may reference
Etas-implemented `tool` declarations and imported symbols whose package or
compiler metadata marks them as model-callable tools. It must not reference
ordinary `flow` declarations directly. To expose Etas logic to a model, wrap
the logic behind a `tool` body and keep helper flows private if they should not
be model-callable.

The desugared shape is:

```etas
agent Name;

impl agent Name ~ Stage<I, O, E> {
    flow run(input: I) -> O ![E] {
        ...
    }
}
```

`impl agent` may also define inherent methods or implement additional specs for
the same nominal agent component. The agent component itself is not a flow value;
its methods are ordinary callable values. A method named `run` is the default
pipeline target used by `x ~> AgentName`.

Inside an agent body or `impl agent` method, `perform infer<T>(prompt)` asks the
current agent runtime to perform a model inference and decode the result as `T`.
The `<T>` may be omitted when the expected type is known from a type annotation,
assignment target, return type, or surrounding expression. The shorthand is
agent-scoped: it is rejected in ordinary `flow` and `tool` bodies because it
needs the current agent identity, method name, model configuration, tool surface,
schema, limits, trace context, and trace-spec context.

```etas
@model("gpt-5.5-thinking")
@tools([latex.check])
@limits([ContextTokens(32_000)])
agent Reviewer(input: Draft) -> Review {
    let prompt = Prompt.new()
        .system(Trusted("Review the draft for correctness and risk."))
        .data(input);

    return perform infer<Review>(prompt);
}
```

`tool ... { ... }` declares a model-callable boundary implemented in Etas. Its
body is checked like a flow body, but with additional tool-boundary rules:
parameters and output must be schema-encodable for model tool calling, inferred
effects must fit the explicit effect boundary and trace-spec constraints,
omitted tool boundaries default to no effects, and forbidden output types such as secrets
must not escape. `flow` remains the ordinary Etas callable and cannot be placed
directly in an agent `@tools(...)` list.

`tool ...;` is a bodyless tool signature. It is used in package interfaces,
generated package metadata, compiler-known standard-library metadata, and
precompiled packages. It is imported and type-checked like an ordinary tool
symbol, but implementation source should provide a body unless the package
manifest or compiler/runtime registry supplies the implementation metadata.

Runtime-provided primitive symbols and host-provided package bindings are not
declared with a source-level keyword. They are imported like ordinary package
symbols, and their compiler/runtime metadata supplies type, effect row,
determinism class, schema information, and host binding information. If such a
symbol declares `![Command.run<_>]`, the command action must carry a sandbox
profile; standard library wrappers use `DefaultCommandSandbox` unless a stricter
profile is passed.

```etas
tool write_report(path: Path, content: string) -> unit ![ProjectWorkspace.write<"reports/**">]
{
    perform ProjectWorkspace.write(path, content);
}

tool create_github_issue(repo: Repo, issue: IssueDraft) -> Issue
    ![Http.request<"api.github.com">, GitHub.issue_create<_>]
{
    return perform GitHub.issue_create(repo, issue);
}
```

External service operations should normally be declared as package-defined or
host-defined effect actions and wrapped by package flows. The minimal standard
library only owns generic orthogonal actions such as console I/O, approval,
time, memory, sandboxed command execution, secrets, errors, and internal
agentic inference support. Host/runtime adapter bindings belong in package
metadata or compiler/runtime registries, not in source syntax.

### 1.5 Trace Specs And Protocols

```ebnf
EffectPattern ::= Path EffectPatternArgs?
EffectPatternArgs
              ::= "<" EffectPatternArgList? ">"
EffectPatternArgList
              ::= EffectPatternArg ("," EffectPatternArg)* ","?
EffectPatternArg
              ::= Ident "~" BoundList
                | StaticEffectArg
                | "_"

ProtocolDecl  ::= "protocol" Ident "{" ProtocolMsg* "}"
ProtocolMsg   ::= Path "->" Path ":" TypeExpr ";"
```

`EffectPattern` mirrors the surface shape of `EffectRef`, but its angle-bracket
arguments may also bind constrained pattern variables such as `R ~
Within<ReportsRoot>`. It deliberately has no `.*` form. In a trace spec, write
the tag name to match the whole tag, and write an action or parameterized action
to match a narrower boundary:

```etas
spec HttpReadOnly: trace =
    +Network
    & -Http.request<"POST", _>
    & -Http.upload<_>;
```

Parameterized action patterns may constrain an argument variable with a spec
bound directly in the angle-bracket list. This lets trace-spec matching reuse
the same relation evidence as generic type checking:

```etas
spec ReportsOnly: trace =
    +Fs.read<R ~ Within<ReportsRoot>>;
```

Trace-spec patterns cannot capture runtime variables in `<...>`. To check a runtime
path, URL, account, tenant, or request id, the runtime trace monitor matches
the concrete action payload against the static pattern.

```etas
spec ReportsOnly: trace =
    +Fs.read<ReportsRoot>
    & +ProjectWorkspace.write<"reports/**">;
```

For example, `perform ProjectWorkspace.write(path, body)` records `path` in the
runtime action event. The effect row should say `ProjectWorkspace.write<_>` or a
known static selector such as `ProjectWorkspace.write<"reports/**">`; it should
not say `ProjectWorkspace.write<path>`.

### 1.6 Statements

```ebnf
Block         ::= "{" Stmt* FinalExpr? "}"
FinalExpr     ::= Expr

Stmt          ::= LetStmt
                | VarStmt
                | AssignStmt
                | IfStmt
                | MatchStmt
                | ForStmt
                | WhileStmt
                | RetryStmt
                | ResumeStmt
                | FinishStmt
                | ReturnStmt
                | BreakStmt
                | ContinueStmt
                | ExprStmt

LetStmt       ::= "let" Pattern TypeAnnot? "=" Expr ";"
VarStmt       ::= "var" Pattern TypeAnnot? "=" Expr ";"
AssignStmt    ::= Expr "=" Expr ";"
TypeAnnot     ::= ":" TypeExpr

IfStmt        ::= "if" Expr Block ("else" (IfStmt | Block))?
MatchStmt     ::= "match" Expr "{" MatchArm* "}"
MatchArm      ::= Pattern "=>" (Expr | Block) ","?

ForStmt       ::= "for" Pattern "in" Expr LimitClause* Block
WhileStmt     ::= "while" Expr LimitClause* Block
LimitClause   ::= "limit" LimitList
LimitList     ::= Expr ("," Expr)* ","?

RetryStmt     ::= "retry" RetryClause* Block
RetryClause   ::= "limit" LimitList

ResumeStmt    ::= "resume" Expr? ";"
FinishStmt    ::= "finish" Expr ";"

ReturnStmt    ::= "return" Expr? ";"
BreakStmt     ::= "break" ";"
ContinueStmt  ::= "continue" ";"
ExprStmt      ::= Expr ";"
```

An expression followed by `;` is a statement whose value is discarded. A trailing expression without `;` is the block's value and participates in return type inference for a `flow` declaration without an explicit `-> Type`, together with explicit `return` statements.

Limit dimensions are typed support constructors rather than keywords. For example, `limit Iterations(20), Tokens(20_000)` is parsed as a `limit` clause containing ordinary expressions whose types must satisfy the `Limit` support spec.

MVP error handling uses the `Error<E>.raise` effect action, runtime-scoped `handle`, the postfix `?` effect-to-`Result` lowering operator, the value type `Result<T, E>`, and the ordinary support flow `abort(...) -> never`. `try`, `catch`, and `throw` are not source keywords.

There is no source-level `parallel` statement in the MVP. Concurrent composition is expressed through library calls such as `join([() => { ... }, () => { ... }])`. Tenant context is also not a keyword; it is an ordinary user-defined struct passed through flows, tools, typed memory APIs, and policies.

There is also no source-level `prompt` declaration in the MVP. Prompt construction is expressed through deterministic flows returning the Agent/runtime support type `Prompt`, for example `flow ReviewPrompt(...) -> Prompt { ... }`.

There is no source-level `msg` or `message` declaration in the MVP. Agent communication is expressed with typed Agent/runtime support values such as `Message<T>`, `SessionConfig`, and `Conversation`. `Prompt` is the model-call input package; `Message<T>` is the communication value that can carry sender, receiver, session, role, trust, and provenance metadata through AIR and traces.

There is no special `approve` expression. Human approval is represented by the
`Approval.request` effect action and stdlib wrapper flows. A call such as
`std.ui.approve(...)` is an ordinary flow call whose signature carries
`![Approval.request]`.

The `>>` and `<<` operators are trace-spec temporal operators. They are not
general expression operators. They are only meaningful in spec expressions of
kind `trace`, for example:

```etas
Approval.request >> CompanyEmail.send<WorkAccount>;
Approval.request >> StripePayment.charge<BillingAccount>;
Sanitized << AcademicSearch.search;
HumanReview >> Blog.publish<Site>;
```

Internally, these requirements normalize to trace-spec terms such as
`trace.before(Approval.request, CompanyEmail.send<WorkAccount>)`. This keeps
source syntax compact while preserving a typed representation for static
analysis, monitor generation, manifests, and diagnostics.

### 1.7 Expressions

```ebnf
Expr          ::= Literal
                | Path
                | RecordExpr
                | TupleExpr
                | ArrayExpr
                | ListExpr
                | MapExpr
                | SetExpr
                | RangeExpr
                | CallExpr
                | MethodCallExpr
                | SpecMethodCallExpr
                | PerformExpr
                | HandleExpr
                | HandlerExpr
                | StageComposeExpr
                | PipelineExpr
                | FieldExpr
                | IndexExpr
                | SliceExpr
                | TryExpr
                | UnaryExpr
                | BinaryExpr
                | IfExpr
                | MatchExpr
                | LambdaExpr
                | Block

CallExpr      ::= Expr TypeArgs? "(" ArgList? ")"
MethodCallExpr::= Expr "." Ident TypeArgs? "(" ArgList? ")"
SpecMethodCallExpr
              ::= Expr "::" SpecRef "." Ident "(" ArgList? ")"
PerformExpr   ::= "perform" Effect "." Ident StaticEffectArgs? "(" ArgList? ")"
                | "perform" "infer" TypeArgs? "(" ArgList? ")"
HandleExpr    ::= "handle" Expr "with" HandlerArg
                | Expr "with" HandlerArg
HandlerArg    ::= HandlerBlock | HandlerValueExpr
HandlerValueExpr
              ::= Expr  // must type-check as a handler value; a bare block is invalid
HandlerExpr   ::= "handler" HandlerBlock
HandlerBlock  ::= "{" HandlerArm* "}"
HandlerArm    ::= Effect "." Ident StaticEffectArgs? "(" PatternList? ")" "=>" (ResumeStmt | FinishStmt | HandlerArmBlock)
HandlerArmBlock
              ::= "{" Stmt* FinalNeverExpr? "}"
FinalNeverExpr
              ::= Expr  // must type-check as never
StageComposeExpr
              ::= PipelineStage "|" PipelineStage ("|" PipelineStage)*
PipelineExpr  ::= Expr "~>" PipelineStage ("~>" PipelineStage)*
PipelineStage ::= Expr LimitClause*
LambdaExpr    ::= ParamList "=>" (Expr | Block)
                | Ident "=>" (Expr | Block)

ArgList       ::= Arg ("," Arg)* ","?
Arg           ::= Ident "=" Expr | Expr

RecordExpr    ::= Path? "{" FieldInit* "}"
FieldInit     ::= Ident ("=" Expr)? ","?
TupleExpr     ::= "(" ArgList? ")"
ArrayExpr     ::= "[" CommaExprList? "]"
ListExpr      ::= "[" SemiExprList? "]"
                | Expr "::" Expr
MapExpr       ::= "{" MapEntryList? "}"
MapEntryList  ::= MapEntry ("," MapEntry)* ","?
MapEntry      ::= Expr "=>" Expr
SetExpr       ::= "#{" CommaExprList? "}"
RangeExpr     ::= "[" Expr "," Expr ")"
                | "(" Expr "," Expr "]"

CommaExprList ::= Expr ("," Expr)* ","?
SemiExprList  ::= Expr (";" Expr)* ";"?

FieldExpr     ::= Expr "." Ident
IndexExpr     ::= Expr "[" Expr "]"
SliceExpr     ::= Expr "[" Expr "," Expr ")"
                | Expr "(" Expr "," Expr "]"
TryExpr       ::= Expr "?"
UnaryExpr     ::= UnaryOp Expr
BinaryExpr    ::= Expr BinaryOp Expr
IfExpr        ::= "if" Expr Block "else" Block
MatchExpr     ::= MatchStmt

UnaryOp       ::= "!" | "-"
BinaryOp      ::= "==" | "!=" | "<" | "<=" | ">" | ">="
                | "+" | "-" | "*" | "/" | "%"
                | "&&" | "||"
```

`TryExpr` is an expression form. It does not consume or require a semicolon.
Semicolons terminate outer statements such as `let` or `return`; a block's final
`e?` expression is written without `;`.

Collection literal forms are:

```etas
[1, 2, 3]                 // Array<i32>
[1; 2; 3]                 // List<i32>
1 :: 2 :: []              // List<i32>
{"alice" => 10}           // Map<string, i32>
#{alice, bob}             // Set<User>
[0, n)                    // Range, left-closed right-open
(0, n]                    // Range, left-open right-closed
```

The empty sequence literal `[]` needs type context to choose between `Array<T>` and `List<T>`. Empty map, set, and record literals also need type context when the syntax alone is insufficient.

Heterogeneous sequence literals require an explicit existential element type:

```etas
let streams: Array<? ~ ByteStream> = [tcp, tls, file]; // OK
let bad = [tcp, tls, file];                            // rejected
```

The accepted literal is still an `Array<T>` for one `T`; here `T` is the
existential type `? ~ ByteStream`.

`IndexExpr` uses the compiler-known `Index` spec. For sequence-like values such as `Array<T>`, `Slice<T>`, and `bytes`, the index expression may have any concrete integer type that satisfies `Index`; the operation remains bounds-checked and does not create a general implicit numeric conversion. Conceptually, `xs[i]` lowers to an internal checked-index operation that either returns `T` or raises `Error<IndexError>`. Code that wants a value-level failure can write `xs[i]?` and receive `Result<T, IndexError>`. For maps, indexing or lookup is keyed by `K`, not by `Index`.

`SliceExpr` uses range bounds directly after the sequence:

```etas
xs[i, j)                  // left-closed, right-open Slice<T>
xs(i, j]                  // left-open, right-closed Slice<T>
```

Slicing is supported for `Array<T>`, `Slice<T>`, `bytes`, and `Range<I>`. Negative indexes and negative slice bounds are rejected in the MVP.

Record field shorthand is supported. In a record expression, `{ topic, papers }` is equivalent to `{ topic = topic, papers = papers }`. This is especially useful for single-input agents that take structured records:

```etas
agent Writer(input: { topic: string, papers: Array<Citation> }) -> Draft {
    return perform infer<Draft>(WriterPrompt(input));
}

let draft = Writer.run({ topic, papers });
```

Anonymous flow expressions use `=>` instead of a `flow` expression keyword:

```etas
let normalize = (title: string) => trim(lowercase(title));

let branches = [
    () => StepA(input),
    () => {
        let value = StepB(input);
        return validate(value);
    },
];
```

The compiler infers each anonymous flow's input type, output type, effect row, and determinism class from its body and surrounding expected type.

Anonymous flow body effects are latent effects: creating a flow value does not execute the body, but calling, composing, joining, returning, or storing that value must preserve and expose its effect row. The detailed rules are in [Effect System and Inference](06-effect-system-and-inference.md#7-anonymous-flows-and-latent-effects).

Agent calls normally use the compiler-generated `run` entrypoint, for example
`Writer.run(topic)`. The `run` body is ordinary checked code; each
agent-scoped `perform infer<T>(prompt)` inside that body records a requested
action such as `Agentic.infer<Writer.run, Draft>` for analysis, trace, replay,
and runtime support. That requested action does not add an `Agentic` obligation
to the source-level escaping effect row of ordinary callers. Tool calls are
ordinary calls to declared tools, for example `http.get(url)`. Memory reads and
writes are ordinary calls to typed memory APIs on named resource handles, for
example `ProjectMemory.Drafts.get(topic)` and
`ProjectMemory.Drafts.put(topic, draft)`.

The `|` operator composes stages and returns a flow value. It is left-associative. A stage may be an agent, a flow, or a tool:

```etas
let pipeline = Researcher | Writer | Publisher;
```

If:

```text
A ~ Stage<I, M, E1>
B ~ Stage<M, O, E2>
```

then:

```text
A | B : I -> O ![E1 + E2]
```

The result is always a flow value, never an `agent`, because the composed value contains multiple runtime steps, trace events, validations, effects, action boundaries, and trace-spec checks. Effects are the union of the composed stages. Determinism is the maximum of the stage determinism classes:

```text
Deterministic < NonDeterministic
```

The `~>` operator is the pipeline application operator. It is left-associative:

```etas
brief ~> ProductManager ~> Architect ~> Engineer ~> Reviewer
```

lowers to a sequence of calls where the left value is passed as the input to the
right stage. If the right stage is an agent name, the operation lowers to
`Agent.run(left)`. If the right stage is a generated agent entrypoint such as
`Agent.run`, a flow, or a tool, it lowers to the corresponding call. If the right
stage is a composition such as `(Researcher | Writer)`, the value is passed to
that composed flow. Effects are the union of all stages, and each
agent/tool/flow stage still produces normal AIR nodes and trace events.

In short:

```text
A | B        = compose stages, returns flow
x ~> A       = apply value to one stage
x ~> (A | B) = apply value to a composed flow
```

A pipeline stage may include local `limit` clauses:

```etas
let input = { design, prd };
let plan =
    input
    ~> Engineer
        limit Tokens(12_000);
```

The `limit` clause attaches only to the immediately preceding pipeline stage.
Extra context should be passed as ordinary typed input values or selected inside
the agent's context harness. Model, tool, and long-lived trace-spec configuration
belong in the `agent` declaration, a wrapper flow/tool, or package/runtime
metadata. `with` is reserved for handler application and does not configure
pipeline stages.

Effect actions are not ordinary calls. An `effect` declaration defines a tag. If the declaration has a block, `action ...;` entries declare typed operation boundaries owned by that tag. The same action signatures may also be split into an `impl EffectName` block. A `perform` expression invokes one of those action boundaries:

```etas
effect Approval {
    action request(req: ApprovalRequest) -> ApprovalDecision;
}

impl Approval {
    action audit(req: ApprovalRequest, decision: ApprovalDecision) -> unit;
}

let decision = perform Approval.request(req);
```

Effect actions are runtime-observable operation boundaries. The minimal standard
library keeps only generic orthogonal actions such as console I/O, approval,
time, memory, secrets, errors, sandboxed command execution, and internal
agentic inference support. Domain and platform actions such as web search,
email, project workspace writes, database updates, and payment charges are
package-defined or host-defined effect actions. Etas-implemented `tool`
declarations wrap those actions with validation, sanitization, approval, limits,
or narrower model-callable schemas. Host/runtime adapters are described by
package metadata or compiler/runtime registries.

`handle` scopes can intercept performed actions from their body. `with` applies
a handler value to an expression. Anonymous handlers are introduced with the
`handler` expression keyword:

```etas
let HumanApproval = handler {
    Approval.request(req) => {
        resume Accepted;
    }
};
```

`handler` is not a declaration keyword; it introduces a handler literal
expression. Handler values can be bound to variables, passed as arguments,
returned from flows, and stored in records like other first-class values.

```etas
let name =
    std.io.read_line() with {
        Error<IOError>.raise(err) => {
            finish "anonymous";
        }
    };
```

`expr with h` is equivalent to `handle expr with h`. A flow-level trailing
handler applies to the whole flow body:

```etas
flow main(args: Array<string>) -> i32 {
    std.io.println("hello");
    return 0;
} with {
    Error<IOError>.raise(err) => {
        finish 1;
    }
}
```

Inside `with`, a block whose entries are handler arms is a handler block. Outside
`with`, the bare form `{ Action(...) => ... }` is not a handler expression.
Write `handler { ... }` when creating an anonymous handler value. This avoids
ambiguity with record, map, and block expressions while keeping handler
application concise.

Handler arms must complete explicitly. `resume value;` supplies the return value
of the current effect action and continues the handled expression. `finish
value;` ends the handled expression and makes the whole `expr with ...` result
be `value`. `return` keeps its ordinary flow-level meaning and is not a handler
completion form; a `return` statement directly inside a handler arm is rejected
unless it belongs to a nested flow declaration or flow literal.

`resume` and `finish` are control-flow keyword statements, not standard-library
functions. `resume` is valid only inside a handler arm for an action whose
return type is not `never`, and the continuation may be resumed at most once.
`finish` is valid only inside a handler arm and its expression must have the
answer type of the handled expression. A handler arm may also end with a
non-returning expression such as `abort(...)`; `never` is the bottom type and is
accepted for any required handler-arm answer.

An ordinary final expression is not a handler fallback. `Action(...) => expr` is
rejected, and a handler arm block whose final expression has a normal value type
is rejected. Use `finish expr;` for answer completion, `resume expr;` for action
resumption, or a `never` expression such as `abort(...)` for non-returning
completion.

These handlers are intentionally more restricted than Koka-style algebraic effect handlers. They are runtime-scoped action handlers for approval, recovery, fallback, trace, and replay; they are not an MVP mechanism for generators, arbitrary coroutines, stored continuations, or multi-shot nondeterministic search.

### 1.8 Names And Patterns

```ebnf
Path          ::= Ident ("." Ident)*
PathList      ::= Path (";" Path)* ";"?

Pattern       ::= Ident
                | "_"
                | LiteralPattern
                | TuplePattern
                | RecordPattern
                | VariantPattern

LiteralPattern
              ::= "true"
                | "false"
                | IntegerLiteral
                | StringLiteral
                | CharLiteral

TuplePattern  ::= "(" Pattern ("," Pattern)* ","? ")"
RecordPattern ::= Path? "{" PatternField* "}"
PatternField  ::= Ident (":" Pattern)? ","?
VariantPattern::= Path "(" PatternList? ")"
PatternList   ::= Pattern ("," Pattern)* ","?
```

Literal patterns match by value and do not bind names. For example, `(true, x)` is a tuple pattern whose first element is the boolean literal `true` and whose second element binds `x`. Boolean literals in pattern position are never variable bindings. MVP literal patterns include booleans, integers, strings, and chars; floating-point literal patterns are intentionally omitted because equality around `NaN` and precision should not be hidden inside pattern syntax.

## 2. Key Design Principles

### 2.1 Agents Are Not Deterministic Flows

Agents are non-deterministic, effectful, trace-producing computations.

### 2.2 Tools Are Not Ordinary Flows

Tools cross trust boundaries and must be governed by effects, action boundaries, and policies.

### 2.3 Prompts Are Not Messages

Prompts are typed model-call input packages. Messages are typed communication values that flow between agents, humans, tools, persistent stores, and runtime control points.

### 2.4 Prompts Are Not Strings

Prompts are typed templates with trust labels and explicit channels.

Agent input values are not converted to prompts through subclassing or `toString`. Prompt construction requires input types to satisfy `PromptEncode`, and agent output validation uses schema/decoder support such as `Schema<T>` and `ResponseDecode`. This keeps structure, trust labels, provenance, channel placement, and trace data visible to the compiler and runtime.

### 2.5 Values Are Not Shared Mutable Objects

Records, tuples, arrays, lists, maps, and sets use value semantics. Passing them to another flow, agent, or tool does not let that callee mutate the caller's value. The implementation may share structure internally, but shared mutable object identity is not part of the source-language semantics.

Persistent mutation must go through typed runtime resources such as `Store<K, V>`, and local mutation must remain explicit through `var` rebinding or visible mutable APIs.

### 2.6 Messages Are Not Raw Strings

Agent-to-agent communication should use typed `Message<T>` values when conversation continuity, handoff, sender/receiver identity, session linkage, trust, or provenance matters.

### 2.7 Persistent State Is Not Magic Context

Persistent state is typed, scoped, versioned, and auditable.

### 2.8 Conversations Are Not Always Protocols

Most conversation state is runtime message/session semantics. Protocol declarations are optional advanced contracts for checking allowed message order.

### 2.9 Reasoning Is Separate from Acting

LLMs may reason, but only the runtime may authorize external actions.

### 2.10 Static Analysis Is Essential

Agent flows should be analyzable before execution.

### 2.11 Traces Are First-Class

Every important event should be logged, replayable, and abstractable.

### 2.12 Non-Deterministic Loops Must Be Controlled

Loops that may call agents, tools, or typed memory APIs are part of the flow semantics. They must either be statically bounded or carry explicit `limit` clauses so the compiler can reason about cost and the runtime can enforce typed limits such as `Iterations(...)`, `Tokens(...)`, `ContextTokens(...)`, `Cost(...)`, and `WallTime(...)`.
