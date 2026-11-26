# Examples

Sections 1 and 2 share declarations; section 3 is independent.

## 1. Stateful Team, Roles And Upward Effects

This standalone variant keeps state across calls. `Team` owns freshly
constructed children; a passive record could also store their references but
would not own them merely by doing so.

```etas
effect AskLead {
    action clarify(question: string) -> string;
}

spec Reviewer ~ Active {
    flow review(var self, draft: string) -> bool ![];
}

@model("configured-model")
agent Coder = {
    revisions: Array<string>,
};

impl Coder {
    public flow implement(var self, goal: string) -> string {
        let guidance = perform AskLead.clarify(goal);
        let prompt = Prompt.new().data({ goal, guidance, revisions = self.revisions });
        let draft = perform infer<string>(prompt);
        self.revisions = self.revisions.push(draft);
        return draft;
    }

    public flow count(self) -> usize ![] {
        return self.revisions.len();
    }
}

@model("configured-model")
agent ModelReviewer = { reviews: usize };

impl ModelReviewer ~ Reviewer {
    public flow review(var self, draft: string) -> bool {
        let accepted = perform infer<bool>(Prompt.new().data(draft));
        self.reviews = self.reviews + 1;
        return accepted;
    }
}

agent MockReviewer = {};
impl MockReviewer ~ Reviewer {
    public flow review(var self, draft: string) -> bool ![] {
        return !std.text.is_empty(draft);
    }
}

@model("configured-model")
agent Team = {
    coder: Coder,
    reviewers: Array<? ~ Reviewer>,
};

impl Team {
    public flow deliver(self, goal: string) -> string {
        let coder = self.coder;
        let reviewers = self.reviewers;
        var draft = coder.implement(goal);

        for _ in [0, 3) {
            let candidate = draft;
            let reviews = std.task.map_concurrent(
                reviewers, reviewer => reviewer.review(candidate)
            );
            var accepted = true;
            for verdict in reviews {
                if !verdict { accepted = false; }
            }
            if accepted { return draft; }
            draft = coder.implement(goal + "\nRevise the previous draft.");
        }
        return draft;
    }

    tool ask_coder(self, goal: string) -> string ![AskLead.clarify] {
        return self.coder.implement(goal);
    }

    @tools([Team.ask_coder])
    public flow model_delegation(self, goal: string) -> string {
        return perform infer<string>(Prompt.new().data(goal));
    }
}

@model("configured-model")
agent Clarifier = {};
impl Clarifier {
    public flow run(self, input: { goal: string, question: string }) -> string {
        return perform infer<string>(Prompt.new().data(input));
    }
}

@limits([Tokens(120_000), Concurrency(4)])
flow BuildWithTeam(goal: string) -> string {
    let team = Team {
        coder = Coder { revisions = [] },
        reviewers = [
            ModelReviewer { reviews = 0 },
            MockReviewer {},
        ],
    };
    return handle team.deliver(goal) with {
        AskLead.clarify(question) => {
            (Clarifier {}).run({ goal, question })
        }
    };
}
```

| Behavior | What the example establishes |
|---|---|
| State | Successful Coder calls commit a revision; error/cancellation before return discards pending revisions. |
| Roles | Both concrete reviewers implement an active behavioral spec; the heterogeneous array retains checked dispatch. |
| Concurrent calls | Mapping starts tasks over distinct reviewer instances; `candidate` avoids capturing mutable `draft`. Repeated aliases do not gain independent write access. |
| Upward handler | Coder's action reaches this caller's handler through Team. Another client may supply a different handler without changing Coder. |
| Model delegation | `team.model_delegation(goal)` under the same handler lets the model request the bound `ask_coder` tool. Its snapshot retains the child reference but cannot modify Team state. |
| Ownership/budgets | Construction owns fresh field children; scope exit cleans them up. Calls/tasks share enclosing token/concurrency accounts. |

## 2. Exclusive Use And Explicit Snapshots

Within a managed scope, using the declarations above:

```etas
let coder = Coder { revisions = [] };
let same = coder;
handle {
    let before = same.count();
    let work = std.task.start(() => coder.implement(goal));
    // same.count(); // rejected: may overlap the writer task
    let patch = std.task.await(work);
    (before, same.count(), patch)
} with {
    AskLead.clarify(question) => question;
}
```

`before` is ordinary snapshot data. `same` aliases coder; it cannot query old
state during the update. After awaiting completion, its read is valid again.

| Case in a managed scope | Result |
|---|---|
| `join((() => coder.implement(goal), () => same.implement(goal)))` | Reject overlapping writes to a known shared instance; no serialization queue. |
| Concurrent `coder.count()` and `same.count()` without a writer | Shared reads allowed. |
| Concurrent updates on independently constructed instances | Instance access is disjoint; still check external effects, handlers, budgets and captures. |
| Dynamically supplied reviewers alias | Unknown identities require proof or exact guards; conflicting admission fails before body execution. |
| Peer reentry through an alias to a live writer | Reject, even in one task; not a direct authorized self-helper call. |

## 3. Scoped Constructor Injection

This standalone example injects two string fields and an active reviewer
without requiring a model provider.

```etas
spec ReviewRole ~ std.agent.Active {
    flow review(self, text: string) -> string ![];
}

agent EchoReviewer = {};
impl EchoReviewer ~ ReviewRole {
    public flow review(self, text: string) -> string ![] {
        return text;
    }
}

type ReviewConfig = {
    @inject
    label: string,
    @inject
    footer: string,
}

agent ReviewTeam = {
    @inject
    reviewer: ? ~ ReviewRole,
}

impl ReviewTeam {
    public flow run(self, text: string) -> string ![] {
        return self.reviewer.review(text);
    }
}

flow ScopedReview(text: string) -> string ![] {
    let reviewer: ? ~ ReviewRole = EchoReviewer {};
    let footer = "";
    let dependencies = handler {
        label => "review: ",
        footer,
        reviewer,
    };
    return handle {
        let config = ReviewConfig {};
        let decorated = config { footer = " [reviewed]" };
        let team = ReviewTeam {};
        team.run(decorated.label + text + decorated.footer)
    } with dependencies;
}
```

`label` and `footer` select different string providers. The typed `reviewer`
binding makes shorthand match the existential field. Reads use stored values;
the reviewer keeps its owner. `Agentic.create/call` remain requested metadata.

| Check, using the declarations above | Expected result |
|---|---|
| `ReviewConfig { label = "explicit", footer = "" }` | Neither provider runs; module-scope construction still obeys global-initializer rules. |
| `ReviewTeam { reviewer }` | Explicit shorthand bypasses the provider; ordinary active construction/dispatch checks remain. |
| Two `ReviewConfig {}` constructions | Four requests, one per omitted field per construction; no implicit cache. |
| `config { footer = " [reviewed]" }`, for `config: ReviewConfig` | New record retains label without provider calls; config is unchanged. Agent-instance heads are rejected. |
| `handler { string => "fallback"; label => "review: "; }` | Named label wins within this handler; footer uses the type fallback. |
| Inner `handler { string => "inner"; }` | Overrides outer string providers; saved fields do not change. |
| `handler { label => "a"; label => "b"; }` | Rejected: duplicate name/type pair. |
| `flow Missing() -> ReviewConfig ![] { ReviewConfig {} }` | Rejected: uncovered dependency requests cannot escape the closed row. |
| `handler { label: string => 42; }` | Rejected: provider value does not match its declared type. |
| Decoder receives no label | Missing required data, not implicit invocation of the label provider. |
| Store an invocation-local injected reviewer into longer-lived state | Rejected when its owner scope would be escaped. |
