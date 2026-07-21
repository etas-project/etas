# IR Stack, FIR, and AIR

This document defines Etas's IR stack:

```text
source
  -> AST
  -> HIR
  -> typed HIR
  -> FIR
  -> optimized FIR
  -> AIR
  -> runtime
```

The stack is introduced across project phases, not all at once:

| Phase | IR / execution focus | Purpose |
|---|---|---|
| Phase 1 | typed HIR interpreter | Validate frontend, name resolution, type checking, effect restrictions, and deterministic local execution |
| Phase 2 | AIR v0 + runtime execution model | Make runtime execution, traces, checkpoints, handlers, and external-effect mediation explicit |
| Phase 3 | FIR middle-end + optimized AIR lowering | Add analysis and transformation infrastructure for agent workflow optimization |

The important separation is:

| Layer | Main role |
|---|---|
| HIR | Resolved, source-shaped representation for names, scopes, source mapping, type/effect facts, and diagnostics |
| FIR | Flow IR: typed, effect-aware, SSA-like analysis and transformation IR |
| AIR | Agent IR: executable runtime IR for interpretation, checkpointing, replay, handlers, traces, and backend/runtime plans |

Typed HIR is the Phase 1 execution target. AIR should become stable enough in
Phase 2 to support runtime interpretation, checkpoint and resume, trace replay,
and future backend generation. FIR should become stable enough in Phase 3 to
support static analysis, graph visualization, optimization, and transformation.

## 1. Design Goals

FIR and AIR should both make determinism, effects, and runtime orchestration explicit, but at different levels of abstraction.

| Goal | Meaning |
|---|---|
| Typed | Every node input and output has a known type |
| Effect-aware | Every node declares possible effects |
| Determinism-aware | Each flow records whether it is deterministic or non-deterministic |
| Authority-aware | Tool, memory, and host access carries effect/action requirements |
| Traceable | Runtime events can be mapped back to AIR node ids |
| Checkpointable | Long-running flows can resume from stable runtime boundaries |
| Structured | Branches, loops, library joins, and retries remain analyzable |
| Backend-neutral | AIR is not tied to one runtime implementation |

FIR is the LLVM-like middle IR for Etas flow analysis: explicit values, data dependencies, control regions, effect summaries, and rewrite-friendly nodes. FIR is not executable runtime state.

AIR is not intended to be a low-level CPU IR. It is closer to an executable typed flow/control-flow graph for deterministic helpers, effectful host interactions, and non-deterministic agent computation.

AIR is also not the source AST. The AST preserves source syntax, while AIR preserves checked execution semantics. A prototype may interpret typed AST directly, but the long-term runtime should interpret AIR so policy checks, effect/action checks, budget checks, runtime checkpoint boundaries, trace node ids, and data dependencies are explicit.

### 1.1 Representation Choice

FIR should be a typed structured graph IR with three-address-code-like nodes and SSA-like value references. In Phase 2, AIR v0 may be generated directly from typed HIR. In Phase 3, AIR should be generated from FIR after required static analyses and selected transformations. Neither FIR nor AIR should use bytecode as the primary representation.

The recommended FIR shape is:

```text
v_out = NodeKind(v_in1, v_in2, ...)
control/data/error/checkpoint edges are explicit
structured regions preserve branch, loop, retry, join, and handler boundaries
```

The recommended AIR shape is:

```text
node[index] = AirOp(inputs = slots, outputs = slots, next = precomputed successor)
frames define value slot layout
regions define loop, join, handler, and checkpoint scopes
```

This split is closer to graph TAC for FIR and executable plan for AIR than to stack bytecode:

| Candidate | Fit for Etas | Reason |
|---|---|---|
| Bytecode | Poor as primary FIR/AIR | Compact and easy to interpret, but hides graph structure, data dependencies, dominance, approval-before-effect checks, and checkpoint boundaries. |
| Linear three-address code | Partial fit | Makes values and operations explicit, but a flat instruction stream loses too much structure around `join`, `retry`, `handle`, loops, and resume points unless many labels and side tables are added. |
| Structured graph TAC | Best primary FIR | Keeps operations explicit like TAC, while preserving control/data edges, regions, node ids, effects and actions, limits, and rewrite boundaries. |
| Executable structured graph | Best primary AIR | Keeps runtime checks, checkpoint boundaries, trace ids, handler dispatch, and replay metadata explicit after FIR analysis and transformation. |

Bytecode may still be useful later as an execution backend:

```text
Etas source -> typed HIR -> FIR -> AIR -> AIR bytecode / native backend / distributed runtime plan
```

But bytecode should be generated from AIR, not replace AIR. Phase 1 should
interpret checked typed HIR. Phase 2 should interpret checked AIR directly so
runtime mediation uses a stable executable representation. Phase 3 should move
static analysis and optimization primarily into FIR.

### 1.2 FIR Feature Coverage Table

This table records where current Etas features belong. The rule of thumb is:

- HIR preserves what the user wrote and what names resolve to.
- FIR represents analyzable semantics and supports rewrites.
- AIR represents executable runtime steps, checkpoints, traces, and replay.

| Feature / construct | HIR responsibility | FIR responsibility | AIR responsibility |
|---|---|---|---|
| Module, package, import, visibility | Preserve package id, module tree, imports, aliases, grouped imports, wildcard imports, re-exports, item names, scopes, and resolved symbols | Mostly not represented except as symbol/module/package metadata on referenced declarations and exported API summaries | Mostly erased; runtime receives selected program/module/package metadata |
| Type declarations, records, enums | Preserve source declarations, generic parameters, field names, variants, and spans | Use normalized checked types on values and operations | Store runtime schemas needed for validation, serialization, and trace decoding |
| Flow declarations | Preserve source signature, optional return type, body shape, parameters, and source spans | Lower to typed flow graph with params, blocks/regions, SSA values, effects and actions, determinism, and data/control dependencies | Lower to executable `AirFlow` with runtime class, trace ids, checkpoint boundaries, handler tables, and dispatch metadata |
| Return type inference | Keep optional return annotation and source return/final-expression structure | Store inferred output type and reject inconsistent exits before AIR | Use final checked output type only |
| Determinism inference | Attach inferred facts to HIR ids/symbols after checking | Propagate and refine `Deterministic` / `NonDeterministic` across calls, agents, tools, handlers, and composition | Store final determinism for replay/recompute/resample behavior |
| Local bindings and variables | Preserve `let`, `var`, patterns, names, scopes, and mutation syntax | Convert to SSA values, use-def chains, and explicit updates/phi-like region outputs | Store runtime locals/value slots only where needed for interpretation, checkpoint, and resume |
| Pure expressions | Preserve source expression tree and operator syntax | Normalize to pure operations such as const, record, field, tuple, list, unary, binary, and pure calls | Execute or precompute remaining pure operations |
| `if` / `match` | Preserve source branches and patterns | Build explicit branch/match regions with dominance and effect summaries | Execute branch dispatch and trace selected path |
| `for` / `while` | Preserve source loop syntax, bindings, condition, and `limit` clauses | Build loop regions with carried values, loop effects, limit requirements, and hoistable dependencies | Execute loop state machine with budget accounting, checkpoint/resume at loop boundaries, and limit enforcement |
| Stage `limit` clauses | Preserve source limit clauses attached to a pipeline stage | Normalize to stage-local budget/resource checks | Execute concrete limit checks before or around the stage |
| `~>` pipeline | Preserve source operator and stage list | Desugar to explicit stage graph with data edges and per-stage metadata | Execute lowered stage calls; emit normal call/message/trace events |
| Stage composition operator | Preserve source composition expression | Type as a composed flow and lower to sequence/graph of stage calls with unioned effects and determinism | Execute the generated composed flow; no special runtime operator required |
| `join([...])` stdlib combinator | Resolve as library flow call | Recognize join shape when signature and arguments allow parallel scheduling analysis | Execute as structured concurrent branches or sequential fallback, preserving trace and failure policy |
| Agent declarations | Preserve `agent` item, nominal identity, optional default `run` body, static annotations, and `impl agent` methods | Model as a nominal component with callable method summaries, input/output schemas, method body effects, latent tool/effect boundary, effect/action requirements, and session/message dependencies | Execute generated or explicit methods, prompt/message encoding, policy checks, provider calls, structured output validation, trace, replay, and usage accounting |
| Agent inference / `perform infer` | Preserve source `perform infer<T>(prompt)` and current agent method identity | Lower to an `AgentCall` / `Agentic.infer<C, O>` runtime action boundary and schema requirements; keep `Agentic.infer<C, O>` in requested-action metadata, not the public escaping effect row | Execute provider call through the runtime agent provider or an explicit test/runtime override; record model response, usage, trace, retry/replay metadata |
| Typed `Message<T>`, conversation, session | Preserve ordinary type/value usage and resolved runtime support APIs | Track message payload type, sender/receiver/session/provenance dependencies, handoff points, and context-window constraints | Append/read conversation state, emit message/handoff trace events, enforce retention and context policy |
| Prompt runtime support values | Preserve normal value construction and resolved APIs | Track prompt parts, trust/provenance labels, channel placement, and prompt taint facts | Encode provider request payloads and record prompt hashes/metadata for replay and audit |
| Etas tool declarations | Preserve `tool` item, signature, optional effect contract, requirements, body, and source spans | Model as a model-callable boundary whose body lowers to a checked subgraph with argument validation, schema output validation, effects and actions, policy checks, and trace metadata | Execute the lowered body under tool-call mediation; record model-callable invocation, validation, effects, and result |
| Runtime-provided tool symbols | Preserve imported tool metadata, signature, explicit effects and actions, and host binding metadata | Model as effectful host operation with idempotency/cache/sandbox metadata and effect/action requirements | Execute via runtime tool registry, sandbox, effect/action checks, policy checks, idempotency keys, and trace |
| Tool calls | Preserve call syntax and resolved callee | Lower Etas tool calls to boundary-enter/body graph/boundary-exit; lower runtime-provided tool calls to checked host operation with explicit data dependencies, effect/action boundary, cacheability, and side-effect classification | Execute checked tool boundary and record result, error, idempotency, and replay metadata |
| Effect tags | Preserve effect declarations and references in requirements/handlers | Propagate effect sets through nodes/regions; distinguish tag-only effects from action signatures | Store final effects on runtime nodes for policy, audit, trace, and handler dispatch |
| Effect actions | Preserve `action` signatures and `perform` expressions | Lower `perform` to typed effect operation with result type, resume behavior, and handler requirements | Dispatch to nearest runtime-scoped handler or default runtime implementation; manage resume/abort semantics |
| `handle` / `resume` | Preserve handler arms, patterns, and lexical scope; `resume` only inside handler arm | Build handler regions, operation-to-arm edges, result types, and legal resume points | Execute runtime-scoped handler dispatch; store resume state in checkpoints and traces |
| Errors / `never` | Preserve `Error<E>` uses, `return`, `abort`, and non-returning operations | Represent exceptional control edges and non-resumable operation exits | Execute abort/error propagation, trace failure, and recovery behavior |
| Trace spec operators `+A`, `-A`, `A >> B`, `A << B` | Preserve source trace spec expressions and resolved action patterns | Turn into allow/deny, dominance, and postcondition constraints over effect/action/tool nodes | Insert/check runtime trace-spec guards when dynamic state is involved; emit satisfaction or violation evidence |
| `spec ...: trace` declarations | Preserve source trace spec items and references | Compile to analyzable constraints over effects and actions, agents, tools, messages, and paths | Evaluate runtime-dependent trace predicates and emit allow/deny events |
| Persistent memory support types and resource handles | Preserve `MemoryRegion<S>`, `Store<K, V>`, top-level `std.memory.region<...>` handles, store field names, resolved import aliases, resolved std memory API calls, and source names | Track memory read/write regions, aliasing, versions, region-sensitive effects, and manifest schema requirements | Execute persistent reads/writes, version checks, retention, checkpoint, and replay integration through runtime-bound memory backends |
| Limits / budgets | Preserve `limit` or normalized `with` options as source clauses | Attach limit dimensions to loop/stage/flow regions; prove required limits are present | Enforce token/context/cost/time/iteration counters and checkpoint active limit state |
| `main` entry | Preserve source entry flow and return expression | Check `main(args: Array<string>) -> i32`, summarize reachable effects/actions, and validate entry requirements | Select `main` as the entry `AirFlow`, verify host grants, execute it, and return its status code |
| Source comments and trivia | Ignored after syntax except source spans/formatting | Not represented | Not represented |
| Trace events | Not represented as source constructs | Planned from node ids/effects/checkpoint points but not executed | Materialized as runtime events linked to AIR node ids |
| Optimizations | Not performed; HIR remains source-shaped for diagnostics | Primary home for DCE, constant folding, prompt partial evaluation, retrieval hoisting, CSE under policy, context slicing, and parallel scheduling | Consume optimized plan; do not perform broad rewrites except runtime scheduling within verified metadata |

## 2. AIR Program Shape

AIR v0 is an executable runtime plan. It is not the general analysis graph and should not require the interpreter to walk arbitrary maps or rediscover control flow from edge lists.

The runtime-facing shape is:

```text
AirProgram {
  id: ProgramId,
  modules: List<Module>,
  types: TypeTable,
  agents: AgentTable,
  tools: ToolTable,
  memories: MemoryRegionTable,
  policies: PolicyTable,
  flows: List<AirFlow>,
  entry: FlowId,
}
```

Each flow lowers to an indexed executable plan:

```text
AirFlow {
  id: FlowId,
  name: string,
  input_type: Type,
  output_type: Type,
  effects: EffectSet,
  determinism: Deterministic | NonDeterministic,
  runtime: Direct | Effectful | Orchestrated,
  effect_boundary: EffectSet,
  frame: FrameLayout,
  nodes: List<AirNode>,
  regions: List<AirRegion>,
  handlers: List<HandlerTable>,
  entry: NodeIndex,
  exits: List<NodeIndex>,
  trace_plan: TracePlan,
  checkpoint_plan: CheckpointPlan,
}
```

The entry flow is `main(args: Array<string>) -> i32`. AIR stores `main` as the entry `AirFlow`, executes it directly, and treats its `i32` result as the process status code. Source-level code may still create, pass, compose, or return flow values elsewhere, and AIR stores those inferred internal flow types as `Flow<I, O, E>`.

### 2.1 Why Indexed AIR

For efficient interpretation, AIR uses:

- dense node indexes instead of map lookup;
- value slots instead of source variable names;
- precomputed successors instead of scanning edge lists;
- frame layouts for checkpoint and resume;
- handler tables for `perform` dispatch;
- join state layouts for concurrent branches;
- trace and checkpoint plans attached to nodes.

This keeps the interpreter loop simple:

```text
while running:
  node = flow.nodes[cursor]
  result = execute(node.op, frame)
  cursor = resolve_next(node.next, result)
  update_trace_and_checkpoint(node, result)
```

AIR may still expose a graph view for debugging and visualization, but that graph view is derived from executable AIR. It is not the primary runtime storage format.

## 3. Execution Model

The interpreter runs a stack of flow frames.

```text
RuntimeState {
  current: FrameId,
  frames: Stack<AirFrame>,
  handlers: Stack<ActiveHandler>,
  joins: Map<JoinId, JoinState>,
  trace: TraceCursor,
  replay: ReplayCursor?,
  host: RuntimeHost,
}
```

Each frame stores values by slot:

```text
AirFrame {
  flow_id: FlowId,
  cursor: NodeIndex,
  slots: SlotArray,
  region_stack: List<RegionFrame>,
  active_limits: List<LimitState>,
  memory_versions: Map<MemoryRegion, VersionId>,
}
```

The runtime does not execute data edges. Data dependencies are compiled into input and output slot lists on each node. Control moves through `Next`.

```text
Next =
  Continue(NodeIndex)
| Branch { true: NodeIndex, false: NodeIndex }
| Match { arms: List<(PatternId, NodeIndex)>, fallback: NodeIndex? }
| Loop { body: NodeIndex, exit: NodeIndex, state: LoopStateId }
| Join { join_id: JoinId, branches: List<NodeIndex>, after: NodeIndex }
| Return
| Abort
| DispatchHandler
```

The compiler may keep source-level control regions in AIR metadata, but the executable transition for each node must be explicit enough that the interpreter does not need to re-run graph analysis.

## 4. Values And Slots

AIR values are stored in typed slots instead of source variable names.

```text
ValueSlot {
  index: u32,
  type: Type,
  trust: TrustLabel?,
  provenance: Provenance?,
}
```

```text
FrameLayout {
  params: List<ValueSlot>,
  locals: List<ValueSlot>,
  temporaries: List<ValueSlot>,
  captures: List<ValueSlot>,
  result: ValueSlot?,
}
```

Source bindings lower to slots:

```etas
let papers = Researcher.run(topic);
```

becomes:

```text
slot[0] topic  : string
slot[1] papers : Array<Paper>
```

This keeps type, trust, and provenance metadata available to runtime checks and trace export while avoiding source-name lookup during interpretation.

## 5. Node Metadata

Every AIR node has common metadata.

```text
AirNode {
  id: NodeId,
  index: NodeIndex,
  op: AirOp,
  inputs: List<ValueSlot>,
  outputs: List<ValueSlot>,
  next: Next,
  effects: EffectSet,
  effect_boundary: EffectSet,
  policies: List<PolicyRef>,
  limits: LimitSet,
  checkpoint: CheckpointKind,
  trace: TraceKind,
  handler_scope: HandlerTableId?,
  source_span: SourceSpan,
  trace_label: string,
}
```

`source_span` supports diagnostics and source-level debugging. `trace_label` gives runtime events a stable human-readable anchor.

`checkpoint` tells the runtime whether this node is a legal resume boundary:

```text
CheckpointKind =
  None
| Before
| After
| BeforeAndAfter
```

`trace` controls event emission:

```text
TraceKind =
  None
| Span
| ExternalCall
| Decision
| Checkpoint
```

## 6. Node Kinds

AIR node kinds should be executable operations. Source syntax sugar and broad analysis-only nodes should already be eliminated by FIR lowering.

Core operations:

```text
AirOp =
  Const(value)
| PureCall(function_id)
| FlowCall(flow_id)
| PromptBuild(agent_method_id)
| AgentCall(agent_method_id, provider_profile, output_schema) // semantic action: Agentic.infer<C, O>
| ValidateSchema(schema)
| ToolCall(tool_id, idempotency_policy)
| MessageCreate(message_schema)
| MessageAppend(session_slot, message_slot)
| Handoff(from_agent, to_agent, reason_slot)
| Memory.read(region, consistency)
| Memory.write(region, write_policy)
| Approval(approval_policy)
| PolicyCheck(policy_id)
| EffectBoundaryCheck(effect_ref)
| BudgetCheck(limit_scope)
| Branch
| Match(pattern_table_id)
| LoopEnter(region_id)
| LoopNext(region_id)
| ForEachNext(region_id)
| Perform(action_id)
| HandlerEnter(handler_table_id)
| HandlerExit(handler_table_id)
| JoinStart(join_id)
| JoinPoll(join_id)
| RuntimeCheckpoint(label)
| Return
| Abort(reason_kind)
```

`PureCall` is an internal lowering target for deterministic source flows and compiler/runtime helper calls. It is not produced from a separate source-level function declaration, because Etas uses `flow` as the callable source construct. `FlowCall` represents a source flow call that still requires flow-level effect, determinism, or runtime semantics.

`PromptBuild` constructs the model-call input package for an `AgentCall`. It is not the semantic representation of agent-to-agent communication. Typed communication is represented by `MessageCreate`, `MessageAppend`, and `Handoff` nodes or equivalent trace events over Agent/runtime support values such as `Message<T>`, `SessionConfig`, and `Conversation`.

`Perform`, `HandlerEnter`, and `HandlerExit` represent runtime-scoped effect handlers. The AIR does not need unrestricted language-level continuations. A `Perform` node uses `Next.DispatchHandler`; the runtime records an attempted action, looks up the nearest handler arm for the action id, and either dispatches to that arm or to the registered default implementation.

The source stage composition operator `|` is eliminated before AIR. A composition such as `Researcher | Writer | Publisher` is typed as a flow and lowered as the corresponding sequence of `AgentCall`, `FlowCall`/`PureCall`, and `ToolCall` nodes. The composed flow's effects are the union of stage effects, and its determinism is the maximum of the stage determinism classes.

The source pipeline operator `~>` is also eliminated before AIR. Each pipeline stage lowers to the ordinary node for that stage: `AgentCall` for an agent, `FlowCall` or `PureCall` for a flow depending on its inferred runtime class, or `ToolCall` for a tool. If the stage is a composed flow created by `|`, the pipeline lowers to that composed sequence. A stage-local `limit ...` clause lowers to explicit budget checks around that stage. Extra prompt context should appear as explicit input values or be selected inside the agent context harness; model/tool configuration belongs in agent declarations, wrapper flows/tools, package metadata, or runtime configuration. These operators do not hide effects and actions, policy checks, approval requirements, or trace nodes.

When a pipeline value is `Message<T>`, lowering preserves message metadata. Passing a message into an agent can produce a `MessageAppend` for the active session and may produce a `Handoff` event if ownership/control moves from one agent stage to another. This keeps MAS communication visible in AIR without adding a source-level `msg`, `message`, or `handoff` keyword.

### 6.1 Handler Dispatch

AIR handlers are runtime-scoped. The interpreter keeps a stack of active handler tables:

```text
ActiveHandler {
  table: HandlerTableId,
  frame_id: FrameId,
  region_id: RegionId,
}

HandlerTable {
  id: HandlerTableId,
  arms: Map<ActionId, HandlerArm>,
}

HandlerArm {
  action: ActionId,
  input_slots: List<ValueSlot>,
  resume_slot: ValueSlot?,
  entry: NodeIndex,
  non_resumable: bool,
}
```

When `Perform(action_id)` executes:

1. the runtime emits a trace event for the attempted action;
2. it searches the handler stack from innermost to outermost;
3. if a handler arm exists, it emits a handled-action event and saves the current continuation in the frame;
4. if no handler exists, it emits a default-action execution event and calls the registered default action implementation if one exists;
5. resumable actions bind the handler or default result to `resume_slot` and continue at the saved `Next`;
6. non-resumable actions must `finish` the handled expression, abort, or be completed by a non-returning default implementation.

A handled action does not automatically execute the default implementation.
Forwarding to the default implementation must be represented as an explicit
runtime support call or lowered operation, and that forwarded default execution
still carries the original effect/action metadata.

`resume` is not a callable runtime function in AIR. It lowers to setting the saved continuation result and jumping to the saved continuation.

### 6.2 Join Execution

`join` lowers to `JoinStart` and `JoinPoll` plus a `JoinRegion`. The interpreter creates child branch frames and records their state:

```text
JoinState {
  id: JoinId,
  parent_frame: FrameId,
  branches: List<BranchState>,
  policy: All | Any | Quorum(n),
  failure_policy: FailFast | CollectErrors | IgnoreFailed,
  cancellation: CancelLosers | KeepRunning,
  result_slots: List<ValueSlot>,
}
```

Branch frames have their own cursors, local slots, trace spans, and checkpoint state. `JoinPoll` resolves when the join policy is satisfied, writes ordered branch results into `result_slots`, applies cancellation if needed, and advances to the `after` node.

## 7. Effects, Actions, And Limits

AIR stores effects, action boundaries, and limits on nodes and regions.

```text
EffectSet      = Set<EffectRef>
LimitSet       = {
  Iterations?: u64,
  Tokens?: u64,
  ContextTokens?: u64,
  Cost?: Money,
  WallTime?: Duration,
}
```

Multiple limits are conjunctive. The runtime must enforce every active limit.

Example:

```etas
while needs_revision(draft)
    limit Iterations(3), Cost(usd(1.00))
{
    draft = Writer.run(draft);
}
```

lowers to a `While` region with:

```text
limits = { Iterations: 3, Cost: usd(1.00) }
effects = effects(body)
```

## 8. Structured Control Regions

Branches and loops should remain structured in AIR only where structure helps execution. AIR regions are runtime scopes, not analysis-only graph regions.

```text
AirRegion =
  LoopRegion {
  id: RegionId,
  condition: NodeIndex,
  body_entry: NodeIndex,
  exit: NodeIndex,
  limits: LimitSet,
  carried_slots: List<ValueSlot>,
  checkpoint: CheckpointKind,
}

| ForEachRegion {
  id: RegionId,
  binding: ValueSlot,
  collection: ValueSlot,
  body_entry: NodeIndex,
  exit: NodeIndex,
  limits: LimitSet,
  carried_slots: List<ValueSlot>,
}

| JoinRegion {
  id: JoinId,
  branches: List<BranchPlan>,
  join_policy: All | Any | Quorum(n),
  result_slots: List<ValueSlot>,
  failure_policy: FailFast | CollectErrors | IgnoreFailed,
  cancellation: CancelLosers | KeepRunning,
  after: NodeIndex,
}
```

Keeping structured regions in AIR is useful for:

- loop resume points;
- budget accounting scopes;
- handler scopes;
- join task state;
- branch cancellation;
- runtime checkpoint placement;
- trace nesting.

FIR remains the better place for dominance, effect summarization, data-flow, and graph visualization. AIR keeps only the structure needed for execution and recovery.

## 9. Lowering Rules

### 9.1 Deterministic Flow Calls

Calls to source flows inferred as `Deterministic` and `runtime = Direct` may lower to `PureCall`.

```etas
let q = normalize(topic);
```

```text
n1 = PureCall(normalize, [topic]) -> q
```

If the callee is non-deterministic or requires runtime mediation, lowering keeps a `FlowCall` or a structured orchestration node instead.

### 9.2 Flow Calls

Flow calls lower to `FlowCall`.

```etas
let summary = SummarizePaper(paper);
```

```text
n1 = FlowCall(SummarizePaper, [paper]) -> summary
```

The caller's effect set must include the callee's effects.

### 9.3 Agent Calls

Agent methods lower like flow bodies, but each `perform infer<T>(prompt)` becomes
an `AgentCall` node whose semantic action is `Agentic.infer<C, O>`, with schema
validation, policy checks, and trace-producing runtime steps. The generated
`run` entrypoint remains typed as `I -> O`, but AIR preserves the prompt,
provider-response, handler, and replay boundary for each inference operation.

```etas
let review = Reviewer.run(draft);
```

```text
n1 = PromptBuild(Reviewer.run, [draft]) -> prompt
n2 = PolicyCheck(Reviewer.policy)
n3 = AgentCall(Reviewer.run, prompt, Schema<Review>) -> review  // Agentic.infer<Reviewer.run, Review>
n4 = PostProcess(Reviewer.run, draft, review) -> review
```

Schema validation is part of the provider boundary and should be visible in
traces and replay metadata, even when the AIR uses an `AgentCall` node whose
result is already the decoded typed value.

`PromptBuild` represents the portion of the agent method body that constructs the
`Prompt` passed to `perform infer`. This code may select context, assemble prompt
parts, and call typed memory APIs. Channel placement must be explicit through
`Prompt`/`PromptPart` values and `PromptEncode` specs; the runtime must not use
`toString` as the default prompt protocol, and it must preserve trust/provenance
metadata through prompt construction, provider dispatch, replay, and audit.

If the agent input or output is a `Message<T>`, AIR also records message/session semantics:

```etas
let reply =
    Message.with_session(msg, session)
    ~> RefundAgent;
```

may lower to:

```text
n1 = MessageAppend(session, msg)
n2 = Handoff(from = TriageAgent, to = RefundAgent, message = msg, reason = route.reason)
n3 = PromptBuild(RefundAgent.run, [msg, session.context]) -> prompt
n4 = PolicyCheck(RefundAgent.policy)
n5 = AgentCall(RefundAgent.run, prompt, Schema<Message<RefundReply>>) -> reply  // Agentic.infer<RefundAgent.run, Message<RefundReply>>
n6 = MessageValidate(reply, Message<RefundReply>) -> reply
n7 = MessageAppend(session, reply)
```

The exact node split can vary by backend, but the trace must preserve message id, sender, receiver, session id, role, payload type, provenance, and handoff metadata.

### 9.4 Tool Calls

Runtime-provided tool calls lower to effect-boundary and policy checks followed by `ToolCall`.
Etas-implemented tool calls lower to a tool boundary node, argument/schema
validation, the lowered body graph, output validation, and a boundary exit node.

```text
n1 = EffectBoundaryCheck(ProjectWorkspace.write<"reports/**">)
n2 = PolicyCheck(ToolPolicy)
n3 = ToolCall(fs.write, [path, content]) -> unit
```

### 9.5 Approval

The Agent/runtime support flow `approve(...)` lowers to an `Approval` node. It is not a special source expression; the source parser sees an ordinary call, and typed lowering recognizes the support signature.

```text
n1 = Approval(request) -> bool
```

High-impact effects can require an approval node to dominate the effectful node in the control graph.

Source trace requirements such as:

```etas
spec SafeEmail: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

lower to typed trace constraints such as:

```text
Before(guard = Approval.request, target = CompanyEmail.send<WorkAccount>)
```

The same AIR constraint shape can represent non-approval guards such as `HumanReview before PublicPublish` or `Sanitized before PromptSystemWrite`.

### 9.6 Memory

Typed memory API calls lower to explicit nodes. AIR memory regions come from compiler-known `MemoryRegion<S>` / `Store<K, V>` types and manifest/runtime bindings.
Imported resource handles and `import ... as ...` aliases resolve to the same
resource symbol before AIR lowering, so aliases do not create distinct memory
identities.

```text
n1 = Memory.read(ProjectMemory.Papers, key) -> paper
n2 = Memory.write(ProjectMemory.Decisions, key, value) -> unit
```

Versioning metadata is attached at runtime and recorded in trace events.

## 10. AIR Example

Source:

```etas
flow WritePaper(topic: string) -> Draft {
    let papers = Researcher.run(topic);
    var draft = Writer.run({ topic, papers });

    while needs_revision(draft)
        limit Iterations(3), Cost(usd(2.00))
    {
        let review = Reviewer.run(draft);
        draft = Writer.run({ topic, papers, review });
    }

    if approve("Accept draft?", draft, risk = Medium) {
        return draft;
    }

    abort("Draft rejected");
}
```

AIR sketch:

```text
flow WritePaper(topic: string) -> Draft
![AcademicSearch.search, ProjectWorkspace.write<"drafts/**">, Memory.read<ResearchMemory>, Memory.write<ResearchMemory>, Approval.request]

requested_actions:
  [Agentic.infer<_, _>, AcademicSearch.search, ProjectWorkspace.write<"drafts/**">, Memory.read<ResearchMemory>, Memory.write<ResearchMemory>, Approval.request]

slots:
  s0  topic        : string
  s1  prompt0      : Prompt
  s2  raw_papers   : ModelResponse
  s3  papers       : Array<Citation>
  s4  prompt1      : Prompt
  s5  raw_draft    : ModelResponse
  s6  draft        : Draft
  s7  needs_more   : bool
  s8  prompt2      : Prompt
  s9  raw_review   : ModelResponse
  s10 review       : Review
  s11 prompt3      : Prompt
  s12 approved     : bool

regions:
  r0 = LoopRegion(
    condition = n9,
    body_entry = n11,
    exit = n18,
    carried_slots = [s6],
    limits = { Iterations: 3, Cost: usd(2.00) }
  )

nodes:
  n1  PromptBuild(Researcher.run) in [s0]       out [s1]   next n2
  n2  PolicyCheck(Researcher)     in []         out []     next n3
  n3  AgentCall(Researcher.run)   in [s1]       out [s2]   next n4   checkpoint agent_call
  n4  ValidateSchema(Array<Citation>) in [s2]    out [s3]   next n5

  n5  PromptBuild(Writer.run)     in [s0, s3]   out [s4]   next n6
  n6  PolicyCheck(Writer)         in []         out []     next n7
  n7  AgentCall(Writer.run)       in [s4]       out [s5]   next n8   checkpoint agent_call
  n8  ValidateSchema(Draft)       in [s5]       out [s6]   next n9

  n9  PureCall(needs_revision)    in [s6]       out [s7]   next branch true=n10 false=n18
  n10 BudgetCheck(r0)             in []         out []     next n11
  n11 PromptBuild(Reviewer.run)   in [s6]       out [s8]   next n12
  n12 AgentCall(Reviewer.run)     in [s8]       out [s9]   next n13  checkpoint agent_call
  n13 ValidateSchema(Review)      in [s9]       out [s10]  next n14
  n14 PromptBuild(Writer.run)     in [s0,s3,s10] out [s11] next n15
  n15 AgentCall(Writer.run)       in [s11]      out [s5]   next n16  checkpoint agent_call
  n16 ValidateSchema(Draft)       in [s5]       out [s6]   next n17
  n17 LoopNext(r0)                in []         out []     next n9

  n18 Approval(Medium)            in [s6]       out [s12]  next branch true=n19 false=n20 checkpoint before_after
  n19 Return                      in [s6]       out []     next return
  n20 Abort(UserRejected)         in []         out []     next abort

next:
  encoded directly in each node above
```

## 11. Trace Mapping

Every runtime event should include the originating AIR node id.

```text
TraceEvent {
  trace_id: TraceId,
  node_id: NodeId,
  flow_id: FlowId,
  event_kind: EventKind,
  timestamp: Time,
  input_refs: List<ValueId>,
  output_refs: List<ValueId>,
  effects: EffectSet,
  metadata: Map<string, Value>,
}
```

Examples:

```text
ModelCall(node_id = n1, agent = Researcher, tokens = ...)
ToolCall(node_id = n3, tool = fs.write, status = ...)
Approval(node_id = n6, decision = accepted)
Checkpoint(node_id = n3, memory_version = ...)
```

This makes traces replayable, auditable, and linkable back to source spans.

For a non-deterministic flow, the semantic result is a set of possible outputs and traces. Deterministic flows are the singleton case:

```text
FlowSemantics : Input × Env -> P(Output × Env × Trace)
AgentSemantics: Input × Env -> P(Output × Env × Trace × Cost)
```

Trace semantics gives the runtime three distinct recovery operations:

| Operation | Re-executes the step? | Expected result | Typical use |
|---|---:|---:|---|
| Recompute | Yes | Same | Deterministic local computation |
| Replay | No, read recorded trace value | Same run | Agent calls, approvals, external reads, crash recovery |
| Resample | Yes | New branch | Explicit regeneration, branch exploration, quality retry |

Recovery uses node metadata such as input hashes, memory versions, model profile, prompt hash, tool idempotency keys, approval ids, and session/message ids to decide whether a replayed value is still valid.

## 12. Checkpoints And Resume

Checkpointable nodes include:

- agent calls;
- tool calls;
- approval gates;
- memory writes;
- loop boundaries;
- explicit `runtime.checkpoint(...)` calls.

A checkpoint stores enough state to rebuild the interpreter, not just the current node:

```text
CheckpointState {
  flow_id: FlowId,
  frame_id: FrameId,
  cursor: NodeIndex,
  slots: SlotArray,
  frame_stack: List<FrameSnapshot>,
  region_stack: List<RegionFrame>,
  handler_stack: List<ActiveHandler>,
  join_states: List<JoinState>,
  memory_versions: Map<MemoryRegion, VersionId>,
  active_limits: List<LimitState>,
  trace_id: TraceId,
  trace_cursor: TraceCursor,
  replay_cursor: ReplayCursor?,
  pending_external_op: ExternalOpId?,
}
```

Resume loads the checkpoint state and continues from `cursor`. If `pending_external_op` is present, the runtime must first resolve it through replay, provider/tool idempotency, or handler policy before advancing the cursor.

AIR nodes that may cross a process boundary or wait on external input should normally checkpoint before and after execution:

| Node kind | Before | After | Reason |
|---|---:|---:|---|
| `AgentCall` | Yes | Yes | Avoid losing in-flight model requests and support replay/resample |
| `ToolCall` | Yes | Yes | Prevent duplicated external side effects |
| `Approval` | Yes | Yes | Do not re-request approval after crash unless policy requires it |
| `Memory.write` | Yes | Yes | Preserve version and write idempotency |
| `JoinStart` / `JoinPoll` | Yes | Yes | Preserve pending branch state |
| Loop boundary | Optional | Yes | Preserve budget and iteration state |
| Pure local operation | No | No | Recompute cheaply |

## 13. AIR Verifier

Before runtime execution, the AIR verifier checks that the executable plan is well formed:

| Check | Requirement |
|---|---|
| Node indexes | Every `next` target points to an existing node in the same flow or to a legal flow exit |
| Slot layout | Every input slot is initialized before use; every output slot has the declared type |
| Frame layout | Flow params, locals, temporaries, captures, and result slots are disjoint unless explicitly aliased |
| Effects | Node effects are included in the enclosing flow effect summary |
| Effect/action metadata | Every `ToolCall`, `AgentCall`, memory operation, and command-like operation has required effect/action metadata |
| Policies | Required `PolicyCheck` nodes are present before guarded runtime operations |
| Limits | Loops and runtime-scoped agentic regions have active limit metadata when required by language policy |
| Handlers | Every `Perform` either has a matching handler arm, a registered runtime default, or is statically rejected; forwarding to a default implementation must remain explicit |
| Resume | Resumable actions have a resume slot and saved continuation; non-resumable actions cannot resume |
| Join | Join branch result slots match the join policy and result arity |
| Checkpoint | Checkpointable external operations have enough metadata for replay, deduplication, or resample |
| Trace | Trace-producing nodes have stable node ids and trace labels; `Perform` records attempted, handled, and default-action execution events |

The interpreter may still validate host policy at runtime, but it should not need to infer missing structural information. If AIR verification fails, the program does not run.

## 14. FIR Optimization Hooks

FIR is the primary home for compiler-style optimizations. AIR receives the optimized executable plan and records enough metadata to preserve trace, replay, and policy semantics.

| Optimization | FIR Support | AIR Requirement |
|---|---|---|
| Context slicing | Data dependencies, prompt/message provenance, and memory read regions | Execute the sliced context and record the slice policy |
| Prompt partial evaluation | Split static and dynamic prompt/message bindings | Preserve prompt hash, source span, and trust/provenance metadata |
| Retrieval hoisting | Lift cacheable/idempotent `ToolCall` nodes under the same policy/identity/authority context | Execute once, reuse by node references, and record dedup evidence |
| Agent call caching | Stable node inputs, model config, memory snapshot, and policy | Replay cached result only when cache key and policy remain valid |
| Verifier insertion | Insert validation nodes after agent/tool outputs | Execute inserted runtime checks and include them in trace |
| Concurrent scheduling | Independent joined subgraphs with no data dependency or effect conflict | Schedule branches without changing externally visible ordering constraints |
| Cost planning | Node effect summaries, determinism, limits, and estimated token/context sizes | Enforce chosen budget plan and record runtime usage |

Optimizations must preserve policy and replay semantics or record that a transformation occurred. AIR should not perform broad semantic rewrites after checkpoint and trace boundaries have been assigned.

## 15. Serialization

AIR should have a deterministic serialized form for debugging, tests, and future distributed backends.

Early implementation can use JSON:

```json
{
  "flow": "WritePaper",
  "effects": ["Network", "FileIO", "Memory.read<ResearchMemory>", "Memory.write<ResearchMemory>", "Approval"],
  "requested_actions": ["Agentic.infer<_, _>", "Network", "FileIO", "Memory.read<ResearchMemory>", "Memory.write<ResearchMemory>", "Approval"],
  "nodes": [
    {
      "id": "n1",
      "kind": "AgentCall",
      "agent": "Researcher",
      "inputs": ["topic"],
      "outputs": ["papers"],
      "effects": ["Network", "FileIO"],
      "requested_actions": ["Agentic.infer<_, _>", "Network", "FileIO"]
    }
  ],
  "edges": [
    { "from": "n1", "to": "n2", "kind": "Control" }
  ]
}
```

The serialized format is not the source language. It is an implementation artifact for tools, runtimes, and tests.
