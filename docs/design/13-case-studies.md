# Case Studies

This document records concrete design pressure from real agent-system engineering patterns. A case study is not a tutorial and not a marketing example. It should answer:

- what the pattern needs;
- which parts Etas already models well;
- where the current language, standard library, runtime, manifest, or tooling is still weak;
- which gaps should become source syntax, which should remain library/runtime support, and which belong to platforms.

## 1. Case Study Overview

The first case study is Harness Engineering, but it should not be the only one. Etas should collect case studies at the same engineering level: not "build a customer support agent", but "which system-engineering problem appears repeatedly when building multi-agent software, and what should the language make analyzable?"

| Case study | Related industry / academic concept | Why it is at the same level as Harness Engineering | Etas pressure point |
|---|---|---|---|
| Harness Engineering | harness engineering, context engineering, agent harnesses | Governs the execution environment around an agent: prompt assembly, tool exposure, policy, validation, feedback, tracing, and deployment configuration. | Make harness logic typed, effect-aware, trace-aware, and visible to static analysis. |
| Memory Engineering | long-term memory, memory management, stateful agents, context state | Governs what state survives across turns/runs, how it is retrieved, compressed, forgotten, versioned, and replayed. | Clarify `MemoryRegion<S>`, `Store<K, V>`, `Message<T>`, `Conversation`, session retention, `Memory.read<R>` / `Memory.write<R>`, and context harness integration. |
| AgentOps / Observability Engineering | AgentOps, agent observability, lifecycle tracing, run monitoring | Governs how production agent runs are traced, inspected, replayed, debugged, and audited. | Turn AIR nodes, effect summaries, agent/tool/message events, determinism, and replay metadata into a stable observability surface. |
| Evaluation Engineering | agent evaluation, trajectory evaluation, eval harnesses | Governs how agent systems are tested beyond final-answer matching: tool use, planning, handoff, safety, cost, robustness, and trace quality. | Support mock tools/models, golden traces, policy simulation, effect coverage, and typed evaluation flows. |
| Tool Governance / Authority Engineering | agentic AI security, tool governance, least privilege, guardrails | Governs which external actions an agent may perform and how authority, sandboxing, approval, and audit are enforced. | Keep `tool`, effect actions, sandbox policy, approval dominance, and manifests separate but connected. |
| Orchestration / Protocol Engineering | multi-agent orchestration, graph/swarm/workflow/handoff, agent protocols | Governs how agents interact: sequence, graph, handoff, delegation, shared state, retries, cancellation, and failure propagation. | Clarify `flow`, `~>`, `|`, `join([...])`, typed messages, session semantics, and whether `protocol` needs first-class syntax. |
| Recovery / Replay Engineering | replay, checkpointing, idempotency, recovery semantics | Governs how failed or resumed runs avoid repeating approvals, model calls, and high-impact tool side effects. | Define determinism, trace, checkpoint, idempotency, replay, recompute, deduplicate, and resample behavior. |

Recommended first batch:

1. Harness Engineering.
2. AgentOps and Replay Engineering.
3. Tool Governance and Authority Safety.
4. Evaluation Engineering for Multi-Agent Systems.

Memory Engineering should also be covered, but it overlaps with Harness Engineering. The boundary should be: Harness Engineering decides what goes into the current model call; Memory Engineering decides how durable state is modeled, updated, retrieved, summarized, versioned, and forgotten over time.

### 1.1 Related Terms And References

The terminology below is useful for positioning Etas without over-claiming that every concept is already a settled academic term.

- Harness Engineering / Externalization: recent agent literature discusses memory, skills, protocols, and harnesses as externalized infrastructure around LLM agents. Etas should treat the harness as a compile-time and runtime object rather than scattered framework glue. Reference: [Externalization in LLM Agents](https://arxiv.org/abs/2604.08224).
- Context Engineering: industry systems use this term for managing the agent context window, context state, and what information is supplied to the model. Reference: [Letta context engineering](https://docs.letta.com/guides/agents/context-engineering).
- AgentOps / Observability: production agent systems need lifecycle tracing, monitoring, debugging, and safety observability. Reference: [AgentOps: Enabling Observability of LLM Agents](https://arxiv.org/abs/2411.05285).
- Evaluation Engineering: agent evaluation increasingly covers planning, tool use, memory, robustness, safety, cost, and trajectory-level behavior rather than only final text quality. Reference: [A Survey on Evaluation of Large Language Model-based Agents](https://arxiv.org/abs/2503.16416).
- Tool Governance / Agentic Security: agentic systems add risks around tool misuse, privilege compromise, memory poisoning, communication poisoning, and rogue agents. Reference: [OWASP Agentic AI Threats coverage](https://pipelab.org/learn/owasp-agentic-threats/).
- Multi-agent Orchestration: frameworks commonly expose graph, swarm, workflow, and handoff patterns for coordinating specialized agents. Reference: [Strands multi-agent patterns](https://strandsagents.com/docs/user-guide/concepts/multi-agent/multi-agent-patterns/).

These references suggest that Etas case studies should focus on engineering surfaces that recur across agent frameworks: context, memory, observability, evaluation, tool authority, orchestration, and recovery. The language contribution is not that Etas invents these concerns, but that it can make them explicit, typed, effect-aware, and traceable.

## 2. Harness Engineering

Harness engineering is the engineering discipline around the agent rather than only inside the agent prompt. It includes tool exposure, runtime policy, guardrails, validation, feedback loops, context construction, memory access, observability, replay, recovery, evaluation, and deployment configuration.

In ordinary framework code, the harness is often spread across Python or TypeScript callbacks, middleware, tool registries, prompt builders, evaluator scripts, tracing hooks, and platform configuration. Etas should make the important parts of that harness visible as typed, effect-aware, trace-aware program semantics.

### 2.1 Harness Requirements

| Requirement | Meaning |
|---|---|
| Tool boundary | The agent can only call declared tools under explicit effect/action authority and sandbox policy. |
| Context boundary | The agent receives selected context, not arbitrary ambient state. |
| Prompt boundary | Trusted instructions and untrusted data remain separated. |
| Verification boundary | Outputs are schema-checked and optionally validated before later side effects. |
| Human gate | High-impact actions can require approval before execution. |
| Resource control | Loops, retries, model calls, and retrieval steps need token, cost, context, time, and iteration budgets. |
| Feedback loop | A failed validation or review can feed back into revision under a bounded loop. |
| Recovery policy | Crashes, retries, resampling, and replay must not duplicate high-impact side effects. |
| Observability | Runs produce semantic traces, not just logs. |
| Evaluation harness | Tests can mock tools/models, compare traces, simulate policy, and replay failures. |
| Deployment manifest | A platform can review required effects and actions, tools, models, limits, approvals, and trace policy before a run. |

### 2.2 What Etas Models Well Today

| Harness concern | Etas mechanism | Status |
|---|---|---|
| Agent boundary | `agent` declaration with nominal identity, typed `run` method, model config, tool exposure, analyzable context code before `perform infer`, and policy requirements | Strong |
| Tool authority | `tool` declarations with `![...]` effect/action contracts and typed policy requirements | Strong |
| Human approval | `approve(...)`, `Approval.request`, `policy`, `handle`, and approval dominance analysis | Strong |
| Sandboxed commands | `Command` effect plus default `DefaultCommandSandbox` runtime support | Strong |
| Structured output | ordinary `type`, `Schema<T>`, `ResponseDecode` evidence, validation flows, `ValidationError` | Strong enough for MVP |
| Prompt trust | `Prompt`, `PromptEncode` evidence, `Trusted<T>`, `Untrusted<T>`, `Sanitized<T>`, explicit prompt channels | Strong |
| Bounded loops | `for`, `while`, `retry`, and `limit` with typed `Limit` values | Strong |
| Flow composition | `|`, `~>`, first-class flow values, `join([...])` stdlib combinator | Strong for code-first harnesses |
| Non-determinism | inferred `Deterministic` / `NonDeterministic` flow facts | Strong conceptually |
| Trace planning | AIR nodes, effect summaries, memory versions, approval ids, checkpoint metadata | Strong design direction |

Example:

```etas
type ResearchRequest = {
    topic: string,
    audience: string,
}

type Report = {
    title: string,
    body: Markdown,
    citations: Array<Citation>,
}

tool web.search(topic: string) -> Array<Untrusted<WebPage>> ![Network];

tool cms.publish(report: Report) -> unit ![Cms.publish<Site>];

flow BuildResearchPrompt(input: { req: ResearchRequest, pages: Array<Sanitized<WebPage>> }) -> Prompt {
    return Prompt.new()
        .system(Trusted("Write a factual, cited report."))
        .data(input);
}

@model("gpt-5.5")
agent ResearchWriter(input: { req: ResearchRequest, pages: Array<Sanitized<WebPage>> }) -> Report {
    return perform infer<Report>(BuildResearchPrompt(input));
}

flow SafeResearch(req: ResearchRequest) -> Result<Report, HarnessError> {
    return {
        let raw_pages = web.search(req.topic);
        let pages = sanitize_pages(raw_pages);

        var report = ResearchWriter.run({ req, pages });

        while !validate_report(report).ok
            limit Iterations(3), Tokens(60_000)
        {
            let feedback = validate_report(report).error;
            report = ResearchWriter.run({ req, pages = apply_feedback(pages, feedback) });
        }

        if approve("Publish report?", report, risk = High) {
            cms.publish(report);
        }

        report
    }?;
}
```

This is a harness, not just a prompt:

- `web.search` is the only network data source and declares `Network`;
- raw pages are `Untrusted<...>` until sanitized;
- the agent receives selected context through typed input;
- the validator controls the revision loop;
- `limit` bounds non-deterministic retry cost;
- publication is a high-impact tool behind approval and effect/action checks;
- AIR can trace the search, sanitization, model calls, validation attempts, approval, and publish action.

### 2.3 Current Gaps

Etas can express a harness-shaped program, but the current design is not yet a complete harness engineering platform. The gaps are mostly outside core syntax.

| Gap | Why it matters | Preferred layer |
|---|---|---|
| Harness manifest format | Deployment systems need a concrete artifact listing effects and actions, model profiles, tools, limits, trace policy, checkpoint policy, and public API summaries. | Compiler/runtime |
| Tool binding registry | `tool` declarations need concrete bindings to MCP, OpenAPI, HTTP, Python services, CLI commands, WASM modules, or hosted tools. | Runtime/tooling |
| Mock model/tool support | Evaluation harnesses need deterministic fixtures and failure injection. | Tooling/runtime |
| Golden trace tests | Reliability tests should compare semantic traces, not only final strings. | Tooling |
| Policy simulation | CI should answer "would this flow be deployable under policy X?" without executing side effects. | Compiler/tooling |
| Context slicing policy | Etas needs standard rules for selecting messages, typed memory API results, retrieval results, summaries, and prompt parts. | Stdlib/runtime |
| Conversation retention | `Message<T>`, `Conversation`, and `SessionConfig` need concrete truncation, summarization, and replay contracts. | Stdlib/runtime |
| Retry and feedback combinators | Users should not hand-write every validation/revision loop. | Stdlib |
| Recovery profiles | Replay, recompute, deduplicate, and resample behavior must be specified per AIR node/tool/model call class. | AIR/runtime |
| Concurrent merge rules | `join([...])` needs clear failure, cancellation, partial result, and memory-write merge semantics. | Stdlib/runtime |
| Interop code generation | OpenAPI, MCP, JSON Schema, protobuf, and package registries should generate `tool` and schema declarations. | Tooling |
| Dashboard/visualization metadata | Platforms need trace and AIR metadata for UI, but this should not add language keywords. | Platform |

### 2.4 Layering Decision

Harness engineering should not become a pile of new keywords. The core source language should remain small.

| Need | Source syntax? | Recommended representation |
|---|---:|---|
| Tool boundary | Yes | `tool`, `effect`, `require` |
| Agent boundary | Yes | `agent` |
| Flow orchestration | Yes | `flow`, `|`, `~>`, `if`, `match`, loops, `limit` |
| Human gate | No special expression | `approve(...)` support flow, `Approval.request` action, policy |
| Checkpoint | No keyword | `runtime.checkpoint(...)` stdlib/runtime support |
| Parallelism | No keyword | `join([...])` stdlib combinator |
| Guardrails | Mostly no keyword | validators, policies, handlers, tool schemas |
| Harness profile | No source keyword | manifest generated from source plus deployment config |
| Evaluation harness | No source keyword | test runner, mock tools/models, golden traces |
| Hosted deployment | No source keyword | runtime/platform |

### 2.5 Design Implications

Harness engineering suggests these priorities for Etas:

1. Specify the deployment/harness manifest.
2. Specify trace schema and golden trace comparison.
3. Specify runtime tool binding metadata and code generation from external schemas.
4. Add stdlib harness combinators for validation, retry-with-feedback, fallback, evaluation, and context selection.
5. Define `join([...])` merge/failure semantics.
6. Define recovery behavior for model calls, tool calls, approval, memory reads/writes, checkpoints, and handlers.
7. Keep source syntax small unless a harness concept must participate in static analysis and cannot be represented as a type, flow, effect, policy, or stdlib combinator.

The key claim is:

> Etas should make harness engineering analyzable. The harness should compile into effects and actions, manifests, AIR nodes, traces, and runtime policies instead of remaining scattered across framework callbacks and deployment glue.
