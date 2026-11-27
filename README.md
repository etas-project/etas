# Etas

An effect-typed programming language for agent systems.

`agent` holds isolated state, `flow` implements behavior, and `tool` exposes
operations to models. Agents `perform` requests; callers interpret them with
`handle ... with`.

Experimental research prototype. Syntax and APIs may change; not production-ready.

## Quickstart

Requires Rust 1.85+ and a native C/C++ build toolchain. Ensure Cargo's `bin`
directory is on your `PATH`.

```sh
git clone https://github.com/etas-project/etas.git
cd etas
cargo run --locked --release -p etas_cli -- std install --prefix "${CARGO_HOME:-$HOME/.cargo}"
etas run examples/hello.es
etas check examples/hello.es
etas effects examples/hello.es
```

This installs the CLI and matching standard library. `hello.es` needs no API key.
`check` validates programs; `effects` shows their inferred effects and requests.

## Agent Patterns

The excerpts below show the core logic. Each linked file includes `main` and offline model fixtures: no API key, network access or actual file writes. Session handoff uses the CLI's in-memory session backend.

```sh
etas run examples/supervision.es
etas run examples/review_panel.es
ETAS_HOST_MEMORY=memory ETAS_HOST_SESSION=memory \
    etas run examples/session_handoff.es --allow-effects
etas run examples/typed_guardrail.es
```

Real models and external tools require configured host services and permissions.

### 1. [Supervision And Human Approval](examples/supervision.es)

The caller chooses an adviser and intercepts writes without changing the coder.

```etas
effect AskArchitect {
    action clarify(question: string) -> string;
}

@model("configured-model")
agent Coder = { revisions: Array<string> };
impl Coder {
    public flow implement<R ~ Region>(var self, task: string, path: WorkspacePath<R>) -> string
        ![AskArchitect.clarify, Fs.write<R>, Error<std.fs.IOError>] {
        let design = perform AskArchitect.clarify(task);
        let code = perform infer<string>(Prompt.new().data(task).data(design));
        self.revisions = self.revisions.push(code);
        perform Fs.write<R>(path, std.codec.text.utf8_encode(code));
        return code;
    }
}

flow Build<R ~ Region>(coder: Coder, architect: Architect, task: string, path: WorkspacePath<R>)
    -> Result<string, string> {
    return handle Ok(coder.implement(task, path)) with {
        AskArchitect.clarify(question) => architect.answer(question);
        Fs.write<R>(target, body) => {
            if !approve("Write generated code?", body, Risk.High) {
                finish Err("Write rejected");
            }
            perform Fs.write<R>(target, body)
        }
    };
}
```

The demo tests rejection and acceptance. Rejection discards pending coder state, **not completed external operations**. The forwarded write reaches an outer handler, which prints a dry-run message here. Unhandled effects can propagate.

### 2. [Reflection And A Reviewer Panel](examples/review_panel.es)

One review request, interchangeable implementations: a peer, a panel or a mock.

```etas
effect Review {
    action check(draft: string) -> bool;
}

@model("configured-model")
agent Writer = {};

impl Writer {
    public flow write(self, topic: string) -> Option<string> {
        var draft = "";
        for _ in [0, 1, 2] {
            draft = perform infer<string>(Prompt.new()
                .system(Trusted("Write or revise the draft for review."))
                .data(topic).data(draft));
            if perform Review.check(draft) { return Some(draft); }
        }
        return None;
    }
}

spec Reviewer ~ std.agent.Active {
    flow review(self, draft: string) -> bool ![];
}

flow Panel(writer: Writer, reviewers: Array<RuleReviewer>, topic: string) -> Option<string> {
    return writer.write(topic) with {
        Review.check(draft) => {
            let votes = std.task.map_concurrent(reviewers, r => r.review(draft));
            !votes.is_empty() && votes.iter().all(vote => vote)
        }
    };
}
```

Swap the review arm for `reviewer.review(draft)` or a test predicate without changing Writer. The demo runs two independent reviewers and checks both success and exhaustion: three failed reviews return `None`, not an unapproved draft.

### 3. [Triage And Session Handoff](examples/session_handoff.es)

`never` requests a transfer with no normal continuation; `finish` supplies the answer.

```etas
enum Department { Billing, Support }
effect Routing {
    action handoff(department: Department, text: string) -> never;
}

flow Route(config: SessionConfig, triage: Triage, billing: BillingDesk,
    support: TechnicalDesk, text: string) -> string {
    return with_lease(config, lease =>
        handle triage.classify(text)
        with {
            Routing.handoff(department, message) => {
                finish handoff(lease, next => match department {
                    Department.Billing => billing.take(next, message),
                    Department.Support => support.take(next, message),
                });
            }
        }
    );
}
```

The demo mocks classification but uses real lease acquisition, transfer and cleanup. `handoff` invalidates the old lease; `finish` returns the destination's answer without resuming triage. This is not an exactly-once delivery guarantee.

### 4. [A Typed Guardrail](examples/typed_guardrail.es)

Require validated input at the downstream agent's type boundary.

```etas
type CheckedNote = { private text: string };
type NoteRejected;

flow check_note(note: Untrusted<string>) -> CheckedNote {
    return match note.value {
        "build: passed" => CheckedNote { text = "The build passed." },
        "build: failed" => CheckedNote { text = "The build failed." },
        _ => perform Error<NoteRejected>.raise(NoteRejected {}),
    };
}

effect Findings {
    action submit(note: Untrusted<string>) -> string;
}

@model("configured-model")
agent ReportWriter = {};
impl ReportWriter {
    public flow compose(self, note: CheckedNote) -> string {
        return perform infer<string>(Prompt.new()
            .system(Trusted("Write a one-sentence build report."))
            .data(note.text));
    }
}

flow Report(researcher: Researcher, writer: ReportWriter, page: Untrusted<string>) -> string {
    return researcher.summarize(page) with {
        Findings.submit(note) => writer.compose(check_note(note));
    };
}
```

Passing `Untrusted<string>` directly to `compose` is a type error. `CheckedNote` is an application type, not std `Sanitized`: this validator accepts two known messages, not arbitrary webpages. The demo checks rejection before downstream
inference. Content remains in `.data`, not `.system`.

## Learn More

- [Language specification](docs/design/README.md) and [more examples](docs/design/07-examples.md)
- [Effects and handlers](docs/design/06-effect-system-and-inference.md)
- [Runnable examples](examples/) and [local development](docs/development/local-checkouts.md)
- [VS Code extension](https://github.com/etas-project/etas-ide) and [Etas Development Kit](https://github.com/etas-project/etas-edk)

## Citation

If you use Etas in research, please cite
[ETAS: An Effect-Typed Language for Agent Systems](https://arxiv.org/abs/2607.17780).
Machine-readable citation metadata, including the preferred paper citation, is
available in [`CITATION.cff`](CITATION.cff).

## License

[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)
