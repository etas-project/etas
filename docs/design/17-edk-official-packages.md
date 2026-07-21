# EDK Official Packages

## 1. Goal

EDK is the official Etas tool-kit distribution built on top of the minimal
standard library. It is analogous to the role a JDK plays for Java users: a
versioned, batteries-included set of common libraries, tools, schemas, mocks,
examples, and tests that make ordinary programs practical
without expanding the language core.

EDK is not a source-language feature. It adds no keywords and no special import
rules. Etas programs consume EDK through ordinary package management and
ordinary imports.

EDK should live as a separate official GitHub repository, for example
`etas-edk`. Its packages should be authored as ordinary Etas packages and
checked by the same frontend, type checker, effect checker, interpreter, and
package tooling as user code. EDK must not rely on compiler-private privileges.

The intended layering is:

| Layer | Content | Core language? |
|---|---|---|
| Core prelude / `std` | Minimal orthogonal support: `Console`, `Error`, `Clock`, `Memory`, `Secret`, `Command`, `Approval`, `Agentic`, collections, `Result`, `Option`, package/module basics, and low-level substrate modules such as `std.net`, `std.stream`, `std.tls`, `std.fs`, codecs, crypto, secrets, and browser protocol transport | Yes |
| EDK official packages | Common production integrations: HTTP, web search, workspace files, email, databases, vector stores, browser automation, PDF/docs, Git/GitHub, Slack/Notion, evaluation harnesses | No |
| Third-party packages | Community packages, enterprise adapters, private business APIs, cloud-specific integrations, generated OpenAPI/MCP bindings | No |

The design rule is:

```text
std = minimal, stable language/runtime foundation
EDK = official reusable integration packages written in Etas
app/package code = domain-specific policy, orchestration, and business logic
```

This keeps the core language small while preserving a good user experience.
Users should not need to define common tools from scratch, but the language
should also avoid committing every external service to the core standard
library.

The design target is full Etas implementation of EDK packages. If a common EDK
tool cannot be implemented in Etas except by adding a high-level private host
binding, that is evidence of a language or `std` substrate gap.

## 2. Relationship To Effects And Actions

EDK packages define ordinary effects, actions, tools, flows, policies, and
support types. Their effects are package-defined authority boundaries, not core
standard-library effects.

For example, EDK may provide:

```etas
module edk.email.smtp;

effect EdkEmail extends Network {
    action send<A>(account: EmailAccount, draft: EmailDraft) -> EmailReceipt;
}

public let EdkEmailDefault: ![EdkEmail.send<_> => EdkSmtp.send<_>, Error<EmailError>] = handler {
    EdkEmail.send(account, draft) => {
        let stream = Smtp.connect(account);
        let message = Smtp.encode(draft);
        let receipt = Smtp.send(stream, message);
        resume receipt;
    }
};

public flow send(account: EmailAccount, to: EmailAddress, draft: EmailDraft)
    -> EmailReceipt ![Error<EmailError>]
{
    return (perform EdkEmail.send(account, draft.with_to(to))) with EdkEmailDefault;
}
```

The compiler treats `EdkEmail.send<_>` exactly like any other effect
action for inference, policy checks, handler dispatch, trace, replay, and
deployment manifests. The only difference from an application-defined action is
ownership and distribution: EDK owns the public API, schemas, Etas
implementation, mock implementation, tests, and compatibility contract.
The internal substrate actions used by the package handler are checked when
compiling EDK itself. Downstream code sees the EDK requested-action/trace
contract, while escaping actions remain caller/application responsibility.

EDK should not reintroduce raw capability strings. Authority remains expressed
through effect actions and trace specs:

```etas
spec SafeMail: trace =
    +Approval.request
    & +EdkEmail.send<WorkAccount>
    & (Approval.request >> EdkEmail.send<WorkAccount>);
```

## 3. Import And Package Model

EDK packages are imported through ordinary package dependencies.

```toml
[dependencies]
std = { version = "0.1" }
edk_http = { package = "edk-http", version = "0.1", import = "edk.http" }
edk_workspace = { package = "edk-workspace", version = "0.1", import = "edk.workspace" }
edk_email = { package = "edk-email", version = "0.1", import = "edk.email" }
```

Source files use normal imports:

```etas
import std.io.*;
import edk.http.client;
import edk.workspace.files;
import edk.email.smtp;
```

The package manager resolves versions, lockfile entries, checksums, substrate
requirements, mock metadata, and package API metadata as described in
[Package Management](15-package-management.md). EDK does not require privileged
source syntax.

## 4. Repository Layout

EDK should be developed as a separate repository and a package workspace. The
repository should be public when the language is ready for external users, but
the design should not depend on being inside the compiler repository.

Recommended layout:

```text
etas-edk/
    README.md
    etas.toml
    etas.lock
    packages/
        edk-http/
            etas.toml
            src/edk/http/client.es
            src/edk/http/json.es
            tests/
        edk-web/
            etas.toml
            src/edk/web/search.es
            src/edk/web/fetch.es
            tests/
        edk-workspace/
            etas.toml
            src/edk/workspace/files.es
            src/edk/workspace/path.es
            tests/
        edk-email/
        edk-db/
        edk-vector/
        edk-browser/
        edk-git/
        edk-github/
        edk-pdf/
        edk-docs/
        edk-eval/
    examples/
        report-publisher/
        research-assistant/
        support-handoff/
    tests/
        golden-effects/
        golden-traces/
        diagnostics/
        interpreter-smoke/
    std-requirements/
        substrate-gaps.md
        accepted-primitives.md
```

The root `etas.toml` should be a workspace manifest:

```toml
[workspace]
members = [
  "packages/edk-http",
  "packages/edk-web",
  "packages/edk-workspace",
  "packages/edk-email",
  "packages/edk-db",
  "packages/edk-vector",
  "packages/edk-browser",
  "packages/edk-git",
  "packages/edk-github",
  "packages/edk-pdf",
  "packages/edk-docs",
  "packages/edk-eval",
]

[workspace.dependencies]
std = { version = "0.1" }
```

Each package should be usable independently through normal package management.
The workspace is for coordinated development, testing, release, and compatibility
checking.

The repository should not contain compiler-private implementation hooks. If a
EDK package needs behavior that cannot be written in Etas using public `std`
APIs, the package should record a substrate gap in `std-requirements/` and the
language/std design should be revisited.

## 5. First EDK Package Set

The first EDK should cover common agent-system integration needs without
becoming a business application framework.

| Package | Example modules | Typical effects/actions | Purpose |
|---|---|---|---|
| `edk.http` | `edk.http.client`, `edk.http.json` | `EdkHttp.request<M, H>` | Typed HTTP requests, JSON helpers, auth-neutral client wrappers |
| `edk.web` | `edk.web.search`, `edk.web.fetch`, `edk.web.crawl` | `EdkWeb.search`, `EdkWeb.fetch<domain>` | Search, fetch, crawl, and return `Untrusted<...>` content by default |
| `edk.workspace` | `edk.workspace.files`, `edk.workspace.path` | `EdkWorkspace.read<P>`, `EdkWorkspace.write<P>`, `EdkWorkspace.list<P>` | Project-local file access with path scopes and policy-visible writes |
| `edk.email` | `edk.email.smtp`, `edk.email.provider` | `EdkEmail.send<A>`, `EdkEmail.read<A>` | Email sending and reading behind account-scoped authority |
| `edk.db` | `edk.db.sql`, `edk.db.pool` | `EdkDb.query<D>`, `EdkDb.exec<D>` | SQL/database access with datasource-scoped effects |
| `edk.vector` | `edk.vector.store`, `edk.vector.embed` | `EdkVector.search<S>`, `EdkVector.write<S>` | Vector search, retrieval, and embedding-store integration |
| `edk.browser` | `edk.browser.page`, `edk.browser.session` | `EdkBrowser.navigate<D>`, `EdkBrowser.click<D>`, `EdkBrowser.read<D>` | Browser automation with domain/session-scoped authority |
| `edk.git` | `edk.git.repo`, `edk.git.diff` | `EdkGit.read<R>`, `EdkGit.write<R>` | Repository status, diff, patch, commit helpers |
| `edk.github` | `edk.github.issue`, `edk.github.pr` | `EdkGitHub.issue_create<R>`, `EdkGitHub.pr_comment<R>` | GitHub API wrappers and tool schemas |
| `edk.pdf` | `edk.pdf.extract`, `edk.pdf.render` | `EdkPdf.read<P>` | PDF text extraction, page rendering, citation-friendly metadata |
| `edk.docs` | `edk.docs.markdown`, `edk.docs.html`, `edk.docs.docx` | `EdkDocs.convert<F>` | Document conversion and structured document helpers |
| `edk.eval` | `edk.eval.golden`, `edk.eval.trace` | `EdkEval.read<E>`, `EdkEval.write<E>` | Evaluation harnesses, golden outputs, trace-based tests |

The naming above is illustrative. The important requirements are:

- EDK package actions are namespaced and package-owned, for example
  `EdkWorkspace.write<P>` rather than a core `Workspace.write<P>`;
- public package APIs are written in Etas;
- high-level packages compose lower-level EDK packages when possible, for
  example `edk.web` should build on `edk.http`, and provider-specific email
  APIs should build on `edk.http` or `edk.email.smtp` when practical;
- a package that cannot be implemented without a new `std` primitive should
  produce a substrate-gap entry rather than hiding the missing primitive behind a
  high-level host binding.

## 6. API Shape

Each EDK integration should expose three layers when useful:

| Layer | Example | Audience |
|---|---|---|
| Effect/action declaration | `EdkWorkspace.write<P>` | Compiler, policy, handler, trace, runtime |
| Default action handler | `let EdkWorkspaceDefault = handler { ... }` | Reference semantics written in Etas |
| Flow wrapper | `edk.workspace.write(path, body)` | Normal Etas code |
| Tool wrapper | `tool repo.write(...) -> ... ![EdkWorkspace.write<P>]` | Model-callable surfaces |

Example:

```etas
module edk.workspace.files;

effect EdkWorkspace extends FileIO {
    action read<P>(path: Path) -> bytes;
    action write<P>(path: Path, body: bytes) -> unit;
}

public let EdkWorkspaceDefault:
    ![EdkWorkspace.read<_>, EdkWorkspace.write<_> => Fs.read<_>, Fs.write<_>, Error<IOError>]
    = handler {
    EdkWorkspace.read(path) => {
        let bytes = std.fs.read_bytes(path);
        resume bytes;
    }

    EdkWorkspace.write(path, body) => {
        std.fs.write_bytes(path, body);
        resume;
    }
};

public flow read(path: Path) -> bytes ![Error<IOError>]
{
    return (perform EdkWorkspace.read(path)) with EdkWorkspaceDefault;
}

public flow write(path: Path, body: bytes) -> unit ![Error<IOError>]
{
    (perform EdkWorkspace.write(path, body)) with EdkWorkspaceDefault;
}

public tool write_text(path: Path, body: string) -> unit
    ![Error<IOError>]
{
    write(path, body.to_bytes());
}
```

The flow wrapper is for ordinary source code. The tool wrapper is for model
surfaces. A model should only see tools explicitly listed in an `agent`
declaration, even when the underlying flow is imported.

The package implementation handler may use lower-level `std` substrate actions.
Those internal effects are part of the EDK package's own implementation check.
The user-facing wrapper does not let `EdkWorkspace.write<_>` escape; it keeps
that action in requested-action and trace facts while the explicit handler
application removes it from the public effect row.

There is no package metadata fallback such as `actions.default_handlers`.
Escaping actions always mean "the caller or application must provide a handler".
If no handler is active, the interpreter reports an unhandled action. EDK default
behavior is ordinary source code: the public API explicitly applies its package
handler with `with`.

## 7. Etas-First Implementation And Substrate

EDK packages should implement their public tools and flows in Etas source when
the semantics are expressible in the language. This is a design constraint, not
only an implementation preference. EDK is the main dogfooding surface for
checking whether Etas can build realistic agent-system libraries.

High-level EDK APIs should not be thin bodyless host bindings by default:

```text
Bad EDK shape:
  edk.http.request = opaque host function
  edk.email.send   = opaque host function
  edk.db.query     = opaque host function

Preferred EDK shape:
  edk.http         = Etas HTTP implementation over std byte streams / sockets
  edk.email        = Etas SMTP/provider protocol wrapper over edk.http or std streams
  edk.db           = Etas driver/protocol wrapper over std streams where practical
  edk.workspace    = Etas path and file helpers over std filesystem primitives
```

The unavoidable host boundary belongs below EDK, in a minimal `std` substrate.
Examples of substrate capabilities that EDK may need are:

| Substrate area | Purpose |
|---|---|
| Byte streams | Read/write byte-oriented streams used by files, sockets, TLS, pipes, and protocol parsers |
| Filesystem primitives | Open, read, write, list, stat, and atomic replace with policy-visible path scopes |
| Network sockets | TCP/UDP connection, read, write, close, and timeout primitives |
| TLS / crypto primitives | TLS transport, hashes, HMAC/signatures, secure randomness, and certificate validation |
| Encoding and compression | UTF-8, base64, URL encoding, JSON bytes, gzip/deflate, and similar reusable codecs |
| Clock and timers | Time reads, deadlines, sleeps, and timeout support |
| Secrets | Host-mediated secret reads without exposing raw environment variables |
| Command sandbox | Explicit sandboxed command execution for cases that are intentionally process-based |

These substrate operations are still ordinary effect actions with policy,
limits, trace, replay, and handler behavior. The difference is that they are
low-level, orthogonal runtime primitives. EDK builds higher-level packages from
them.

If implementing a EDK package reveals that the substrate is missing a necessary
primitive, the correct response is to improve the `std` substrate design, not to
add a package-private high-level host escape hatch.

A substrate gap entry should record:

| Field | Meaning |
|---|---|
| Package | Which EDK package exposed the gap |
| Attempted implementation | What Etas code or design was attempted |
| Missing primitive | The smallest low-level `std` operation that appears necessary |
| Desired signature | Proposed type, effect/action row, and error behavior |
| Safety impact | Policy, trace, sandbox, secret, or trust implications |
| Alternative considered | Why existing `std` or EDK APIs were insufficient |

Example:

```text
Package: edk-http
Attempted implementation: HTTPS client over std.stream and std.net.tcp
Missing primitive: TLS client session with certificate validation
Desired signature:
  std.tls.connect(stream: TcpStream, server_name: Host, config: TlsConfig)
      -> TlsStream ![Tls.handshake<_>, Error<TlsError>]
Safety impact:
  host name and certificate validation must be policy-visible and traceable
Alternative considered:
  implementing TLS fully in EDK is possible long-term, but first EDK needs a
  minimal cryptographic substrate and secure randomness
```

EDK package metadata must still publish:

- public type signatures;
- effect and action contracts;
- tool schemas;
- determinism classification;
- trust/provenance of returned content;
- idempotency and replay metadata when applicable;
- mock binding identifiers for tests.

For example, `edk.web.search` should return untrusted content by default:

```etas
public tool search(query: SearchQuery) -> Array<Untrusted<SearchResult>>
    ![EdkWeb.search, Error<SearchError>]
{
    return perform EdkWeb.search(query);
}
```

EDK should also provide mocks so compiler, interpreter, and application tests
can run without real network, email, database, browser, or payment access:

```toml
[bindings.profile.test]
edk.web.search = "mock:edk.web.search"
edk.email.smtp = "mock:edk.email.smtp"
edk.workspace.files = "mock:edk.workspace.files"
```

Mocks may replace low-level substrate actions or high-level EDK flows in test
profiles, but production EDK package behavior should remain an Etas
implementation over the public `std` substrate. Mock behavior is not part of the
language semantics, but it is part of the EDK compatibility contract.

## 8. Implementation Completeness Rule

EDK should use a strict completeness rule:

```text
Every public EDK flow/tool must either:
  1. have an Etas body, or
  2. be a low-level std substrate primitive outside EDK.

Every user-facing EDK flow/tool that is meant to run by default must apply its
package implementation handler explicitly in Etas source. Every action that
escapes a EDK API is intentionally abstract and must be handled by the caller or
application; otherwise execution fails with an unhandled action diagnostic.
```

This means EDK should reject high-level public APIs such as these:

```etas
// Not acceptable as the main EDK implementation.
public tool http.request(req: HttpRequest) -> HttpResponse
    ![EdkHttp.request<_, _>];

public tool email.send(draft: EmailDraft) -> EmailReceipt
    ![EdkEmail.send<WorkAccount>];
```

Instead, the EDK package should expose package actions and implement its
user-facing APIs by explicitly applying package handlers written in Etas over
lower-level public primitives:

```etas
effect EdkHttp extends Network {
    action request<M, H>(req: HttpRequest) -> HttpResponse;
}

public let EdkHttpDefault:
    ![
        EdkHttp.request<_, _>
        => Net.tcp_connect<_, _>,
           Tls.handshake<_>,
           Stream.write<_>,
           Stream.read<_>,
           Error<HttpError>
    ] = handler {
    EdkHttp.request(req) => {
        let tls = EdkHttp.open_tls(req);
        let wire = EdkHttp.to_wire_request(req);
        EdkHttp.write_wire_request(tls, wire);
        let raw = EdkHttp.read_wire_response(tls, req.limit, req.timeout);

        match std.http.codec.decode_response_head(raw) {
            Ok(head) => {
                resume EdkHttp.from_wire_response(head, raw);
            }
            Err(err) => {
                perform Error<HttpError>.raise(HttpError.Codec(err));
            }
        }
    }
};

public flow request(req: HttpRequest) -> HttpResponse
    ![Error<HttpError>]
{
    return (perform EdkHttp.request(req)) with EdkHttpDefault;
}
```

### 7.1 Implementing `edk.http`

The reference `edk.http` implementation should be ordinary Etas source layered
over public `std` substrate. The package owns user-facing HTTP types and the
package action:

| Layer | Owned by | Example |
|---|---|---|
| User-facing API | EDK | `edk.http.client.request(req: HttpRequest)` |
| User-facing authority | EDK | `EdkHttp.request<M, H>` |
| Wire codec | `std` | `HttpWireRequest`, `HttpWireResponseHead`, `std.http.codec.*` |
| Transport substrate | `std` | `Net.tcp_connect`, `Tls.handshake`, `Stream.read`, `Stream.write` |

The implementation pipeline should be explicit:

1. normalize and validate the EDK `HttpRequest`;
2. translate it to `HttpWireRequest`;
3. open TCP with `std.net.tcp.connect`;
4. wrap the stream with `std.tls.connect` for HTTPS;
5. write encoded wire bytes with `std.stream.write_all`;
6. read response bytes with `std.stream.read_until_limit`;
7. decode wire response metadata with `std.http.codec`;
8. translate the wire response back to EDK `HttpResponse`;
9. map substrate errors into the public `HttpError` type.

`std.tls.connect` returns the precise opaque type `TlsStream`. The later calls
to `std.stream.write_all` and `std.stream.read_until_limit` accept it because
those APIs are generic over `S ~ ByteStream`, and `TlsStream` satisfies the
standard `ByteStream` spec. This is spec-bound polymorphism, not general
subtyping.

EDK package errors should use one aggregate enum instead of many duplicate
record types:

```etas
public enum HttpError {
    Network(NetworkError),
    Tls(TlsError),
    Stream(StreamError),
    Codec(HttpCodecError),
    InvalidUrl { input: string, message: string },
    InvalidHeader { name: string, message: string },
}
```

For example:

```etas
flow EdkHttp.open_tls(req: HttpRequest) -> TlsStream
    ![
        Net.tcp_connect<_, _>,
        Tls.handshake<_>,
        Error<HttpError>
    ]
{
    handle {
        let tcp = std.net.tcp.connect(req.host, req.port, req.options);
        return std.tls.connect(tcp, req.host, req.tls);
    } with {
        Error<NetworkError>.raise(err) => {
            perform Error<HttpError>.raise(HttpError.Network(err));
        }

        Error<TlsError>.raise(err) => {
            perform Error<HttpError>.raise(HttpError.Tls(err));
        }
    }
}

flow EdkHttp.write_wire_request(stream: TlsStream, req: HttpWireRequest) -> unit
    ![Stream.write<TlsStream>, Error<HttpError>]
{
    handle {
        std.stream.write_all(stream, std.http.codec.encode_request(req));
    } with {
        Error<StreamError>.raise(err) => {
            perform Error<HttpError>.raise(HttpError.Stream(err));
        }
    }
}

flow EdkHttp.read_wire_response(
    stream: TlsStream,
    limit: ByteLimit,
    timeout: Timeout?
) -> bytes
    ![Stream.read<TlsStream>, Error<HttpError>]
{
    handle {
        return std.stream.read_until_limit(stream, limit, timeout);
    } with {
        Error<StreamError>.raise(err) => {
            perform Error<HttpError>.raise(HttpError.Stream(err));
        }
    }
}
```

This keeps the public application contract small:

```etas
![EdkHttp.request<"GET", "api.github.com">, Error<HttpError>]
```

while still letting EDK itself be checked against the lower-level substrate
actions:

```etas
![Net.tcp_connect<"api.github.com", 443>, Tls.handshake<"api.github.com">, Stream.write<_>, Stream.read<_>]
```

The first reference implementation should not include every HTTP client feature.
Redirects, connection pooling, cookies, proxy support, compression, streaming
bodies, auth schemes, cache policy, and retry/backoff should be layered as EDK
modules over this minimal substrate-backed client. Each such layer should expose
its own policy-visible behavior when it changes authority, replay, idempotency,
or trust semantics.

The exact low-level APIs are part of the standard-library substrate design, not
EDK-private host calls. Effect/action owners use uppercase names without a `Std`
prefix, for example `Net.tcp_connect`, `Stream.read`, `Tls.handshake`, and
`Fs.write`.

Real EDK source must also map substrate errors such as `NetworkError`,
`TlsError`, `StreamError`, and `HttpCodecError` into the public EDK error type
such as `HttpError`. The example focuses on the dependency direction and
wire-level type boundary: `std.http.codec` operates on `HttpWire*` types, while
EDK owns user-facing `HttpRequest` and `HttpResponse` records.

Acceptable exceptions are narrow:

| Exception | Rule |
|---|---|
| Test mocks | May replace EDK flows or substrate actions in test profiles |
| Generated source | May generate Etas source from OpenAPI/MCP/protobuf schemas, then compile it normally |
| Platform acceleration | May provide an optimized implementation only if the Etas implementation remains the reference semantics |
| Temporarily missing substrate | Must be recorded as a substrate gap and should not become the final public API design |

This rule is deliberately strict. EDK is not just a convenience library; it is a
stress test for whether Etas can express real integration code.

## 9. Trace Specs And Safety Defaults

EDK should be safe by default:

- web, HTTP, PDF, and document readers return `Untrusted<...>` unless a wrapper
  explicitly validates or sanitizes the content;
- writes, email sends, browser actions, database mutations, payment operations,
  deployment operations, and command execution should be high-impact actions;
- high-impact EDK actions should ship with suggested trace-spec templates;
- EDK wrappers should expose idempotency keys for non-idempotent operations
  such as email, payment, publishing, and issue creation;
- path, account, host, repository, tenant, datasource, and collection scopes
  should appear as effect/action parameters when they matter for trace specs.

Example trace spec:

```etas
spec PublishReportPolicy: trace =
    +EdkWeb.search
    & +EdkWorkspace.write<"reports/**">
    & +Approval.request
    & +EdkEmail.send<WorkAccount>
    & -EdkWorkspace.write<"src/**">
    & (Approval.request >> EdkEmail.send<WorkAccount>);
```

## 10. Example Use

```etas
import std.ui;
import edk.web.search;
import edk.workspace.files;
import edk.email.smtp;

spec ReportPolicy: trace =
    +EdkWeb.search
    & +EdkWorkspace.write<"reports/**">
    & +Approval.request
    & +EdkEmail.send<WorkAccount>
    & (Approval.request >> EdkEmail.send<WorkAccount>);

@model("gpt-5.5")
agent Writer(input: { topic: string, pages: Array<Untrusted<SearchResult>> }) -> Draft {
    let prompt = Prompt.new()
        .system(Trusted("Write a concise internal report."))
        .data(input);
    return perform infer<Draft>(prompt);
}

flow PublishReport(topic: string) -> EmailReceipt
    ~ ReportPolicy
{
    let pages = search.query(topic);
    let draft = Writer.run({ topic, pages });

    files.write("reports/latest.md", draft.body.to_bytes());

    if ui.approve("Send report?", draft, risk = High) {
        return smtp.send(WorkAccount, "team@example.com", draft);
    }

    abort("report rejected");
}
```

The inferred behavior remains explicit:

```text
effects =
    [EdkWeb.search,
     EdkWorkspace.write<"reports/**">,
     EdkEmail.send<WorkAccount>,
     Approval.request]

requested_actions =
    [Agentic.infer<Writer.run, _>,
     EdkWeb.search,
     EdkWorkspace.write<"reports/**">,
     Approval.request,
     EdkEmail.send<WorkAccount>]
```

## 11. Compatibility And Versioning

EDK packages should use the same package compatibility rules as other Etas
packages, with stricter stability expectations:

- public type signatures, effect rows, action names, action parameters, tool
  schemas, and default trust labels are API surface;
- narrowing an effect row may be compatible when it only removes impossible
  behavior;
- widening an effect row is a breaking change unless the added action is behind
  a new opt-in API;
- changing an action parameter that policy can match is a breaking change;
- mock binding behavior should be stable enough for golden tests.

EDK can evolve faster than `std`, but slower than arbitrary third-party
packages. This lets Etas keep the core stable while still providing useful
official integrations.

## 12. Checks And Golden Tests

The EDK repository should be continuously checked by the Etas implementation.
At minimum, it should support commands equivalent to:

```text
etas check --workspace
etas effects --workspace
etas test --workspace
etas trace examples/report-publisher
etas edk golden --check
```

The exact CLI names can change, but the checked artifacts should be stable:

| Test family | What it checks | Example failure |
|---|---|---|
| Package check | Imports, package graph, public metadata, visibility | `edk.email` imports a private item from `edk.http` |
| Type check | Public and private type correctness | HTTP decoder returns `bytes` where `HttpResponse` is required |
| Effect summary | Inferred effects and requested actions | `edk.workspace.write_text` hides `EdkWorkspace.write<P>` |
| Trace-spec examples | Suggested trace specs accept and reject intended cases | Report trace spec permits `EdkWorkspace.write<"src/**">` by accident |
| Handler examples | Dry-run, retry, error, and approval handlers have expected escaping effects | Dry-run workspace handler still lets `EdkWorkspace.write` escape |
| Golden traces | Stable action order, handler dispatch, default execution, limits, and errors | Email example sends before approval in the trace |
| Interpreter smoke | Executable examples run on the reference interpreter | `edk.http` parser uses unsupported language behavior |
| Diagnostics | Negative tests produce useful errors | Missing `limit` in an agent loop is accepted |
| Substrate coverage | EDK uses only approved `std` substrate primitives | `edk.db` requires an unapproved private socket binding |

Golden effect summaries should be checked as source-level semantic contracts:

```text
flow edk.workspace.files.write_text
  effects: [EdkWorkspace.write<_>, Error<IOError>]
  requested_actions: [EdkWorkspace.write<_>]
  determinism: NonDeterministic
```

Golden traces should check runtime behavior:

```text
Trace:
  FlowEnter(PublishReport)
  ActionAttempt(EdkWeb.search)
  AgentCall(Writer)
  ActionAttempt(EdkWorkspace.write(path = "reports/latest.md"))
  ActionAttempt(Approval.request)
  ActionAttempt(EdkEmail.send<WorkAccount>)
  FlowExit(EmailReceipt)
```

EDK should also include negative examples:

```etas
spec ReportOnly: trace =
    +EdkWorkspace.write<"reports/**">;

flow Bad(path: Path, body: bytes) ~ ReportOnly {
    EdkWorkspace.write(path, body); // requires residual check or rejection
}
```

These tests keep the language implementation and the SPEC aligned.

## 13. EDK As A Language Design Pressure Test

EDK is also a practical check on the language design and interpreter:

- every EDK package should include small executable examples;
- core integrations should include golden effect summaries and golden traces;
- tests should exercise ordinary imports, package metadata, Etas tool bodies,
  substrate use, mock bindings, handlers, policies, limits, and errors;
- if a common EDK wrapper is awkward to express, that is evidence that the
  language surface or standard runtime APIs need review.

This is secondary to EDK's main purpose as a user-facing tool-kit, but it is
important: the official tool packages should keep the SPEC honest.
