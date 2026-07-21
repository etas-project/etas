# Application Boundaries and Framework Coverage

This document compares Etas's current language design with common multi-agent system frameworks and clarifies the intended application boundary. It is not a language specification. It is a product and research checklist for deciding which features belong in Etas source syntax, which belong in the standard library/runtime, and which should remain platform integrations.

## 1. Scope

Etas should cover multi-agent systems that need typed orchestration, explicit tool authority, bounded non-determinism, memory access discipline, human approval, traceability, replay, and static analysis.

Etas should not try to be every layer of an agent platform. Scheduling, hosted deployment, visual workflow editing, dashboards, connector marketplaces, billing, RBAC administration, and no-code authoring are important product features, but they should be built on top of Etas manifests, AIR, traces, and runtime APIs rather than added as core source-language constructs.

The practical boundary is:

```text
Etas source language:
  typed flows, agents, tools, typed memory APIs, policies, effects, handlers, limits, composition

Etas runtime and standard library:
  checkpointing, group chat combinators, scheduling adapters, session support,
  model providers, tool registries, manifests, trace export

Etas platform integrations:
  dashboards, visual builders, connector catalogs, team/RBAC admin,
  hosted automations, billing, monitoring UI
```

## 2. Systems Reviewed

This comparison is based on public documentation for representative agent frameworks and SDKs:

| System | Relevant features |
|---|---|
| [LangGraph overview](https://docs.langchain.com/oss/python/langgraph), [persistence](https://docs.langchain.com/oss/python/langgraph/persistence), [human-in-the-loop](https://docs.langchain.com/oss/python/langchain/human-in-the-loop), and [multi-agent patterns](https://docs.langchain.com/oss/python/langchain/multi-agent/index) | Graph-shaped workflows, persistence, checkpointing, human-in-the-loop, custom multi-agent control flow |
| [AutoGen teams and termination](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/teams.html), [termination conditions](https://microsoft.github.io/autogen/dev/user-guide/agentchat-user-guide/tutorial/termination.html) | Conversable agents, group chat teams, round-robin/selector/swarm patterns, handoff, termination |
| [CrewAI documentation](https://docs.crewai.com/) and [CrewAI Flows](https://docs.crewai.com/en/concepts/flows) | Crews, tasks, flows, tools, memory, knowledge, guardrails, stateful and event-driven automation |
| [CloudWeGo Eino](https://github.com/cloudwego/eino), [Eino user manual](https://www.cloudwego.io/docs/eino/), [Eino chain/graph/workflow orchestration](https://www.cloudwego.io/docs/eino/core_modules/chain_and_graph_orchestration/), and [Eino components](https://www.cloudwego.io/docs/eino/core_modules/components/) | Go LLM application framework, components, ADK agents, graph/workflow orchestration, tools, retrievers, streaming, callbacks, interrupt/resume, and visual/debug tooling |
| [OpenAI Agents SDK guide](https://developers.openai.com/api/docs/guides/agents), [guardrails](https://openai.github.io/openai-agents-js/guides/guardrails), [running agents](https://openai.github.io/openai-agents-js/guides/running-agents), and [tracing](https://github.com/openai/openai-agents-python/blob/main/docs/tracing.md) | Agents, tools, handoffs, guardrails, context propagation, tracing, structured runs |
| [Microsoft Agent Framework group chat orchestration](https://learn.microsoft.com/en-us/agent-framework/workflows/orchestrations/group-chat) | Group chat orchestration, manager strategies, shared context and termination policies |

The goal is not to clone these APIs. The goal is to ensure Etas can express the important system shapes while preserving a small, analyzable language core.

### 2.1 Agent Definition Shape Across Frameworks

Current agent frameworks converge on a similar host-language shape:

```text
Agent = model + instruction/prompt + tools + message state + runtime harness
```

This shape is useful and should not be dismissed. The design question for Etas
is whether `agent` adds language semantics that a Python, TypeScript, or Go class
cannot provide.

Representative LangChain code uses the documented `create_agent` pattern:

```python
from langchain.agents import create_agent
from langchain.tools import tool
from pydantic import BaseModel

@tool
def search(query: str) -> str:
    """Search for information."""
    return f"Results for: {query}"

class Answer(BaseModel):
    summary: str
    confidence: float

agent = create_agent(
    model="openai:gpt-5.5",
    tools=[search],
    system_prompt="You are a helpful research assistant.",
    response_format=Answer,
)

result = agent.invoke({
    "messages": [
        {"role": "user", "content": "Summarize AI agent frameworks"}
    ]
})
```

LangChain describes an agent as a model calling tools in a loop and its harness
as the model, prompt, tools, and middleware around that loop. Its docs also show
structured output with `response_format`, conversation persistence with
`thread_id`, and middleware for context, guardrails, fault tolerance, and
human-in-the-loop steering. See [LangChain Agents](https://docs.langchain.com/oss/python/langchain/agents).

Representative Google ADK code uses the documented `root_agent` / `Agent`
shape:

```python
from google.adk.agents import Agent

def get_current_time(city: str) -> dict:
    """Returns the current time in a specified city."""
    return {
        "status": "success",
        "city": city,
        "time": "10:30 AM",
    }

root_agent = Agent(
    model="gemini-flash-latest",
    name="root_agent",
    description="Tells the current time in a specified city.",
    instruction=(
        "You are a helpful assistant that tells the current time in cities. "
        "Use the 'get_current_time' tool for this purpose."
    ),
    tools=[get_current_time],
)
```

ADK's quickstart states that `root_agent` is the required element of an agent
project, and its `LlmAgent` documentation identifies name, model, description,
instructions, tools, context, planners, code execution, and structured data as
core configuration concerns. See [ADK Python Quickstart](https://adk.dev/get-started/python/)
and [ADK Simple Agents](https://adk.dev/agents/llm-agents/).

Representative AutoGen code uses the documented `AssistantAgent` pattern:

```python
from autogen_agentchat.agents import AssistantAgent
from autogen_ext.models.openai import OpenAIChatCompletionClient

async def web_search(query: str) -> str:
    """Find information on the web."""
    return "Search results..."

model_client = OpenAIChatCompletionClient(model="gpt-4.1-nano")

agent = AssistantAgent(
    name="assistant",
    model_client=model_client,
    tools=[web_search],
    system_message="Use tools to solve tasks.",
    max_tool_iterations=10,
)

result = await agent.run(task="Research AutoGen and summarize it.")
```

AutoGen describes `AssistantAgent` as a built-in agent that uses a language model
and can use tools. Its docs show Python functions becoming tools, tool schemas,
MCP workbenches, agents-as-tools, parallel tool calls, and `max_tool_iterations`.
They also warn that agent/team tools with internal state should not be run in
parallel. See [AutoGen Agents](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/agents.html).

AutoGen custom agents are the main counterexample to "agent as configuration".
They define an ordinary Python method body such as `on_messages(...)`:

```python
from autogen_agentchat.agents import BaseChatAgent
from autogen_agentchat.messages import TextMessage
from autogen_agentchat.base import Response

class ArithmeticAgent(BaseChatAgent):
    async def on_messages(self, messages, cancellation_token):
        number = int(messages[-1].content)
        result = self._operator_func(number)
        return Response(chat_message=TextMessage(
            content=str(result),
            source=self.name,
        ))
```

That body is fully expressive Python. This is powerful, but it also means the
framework normally cannot statically summarize arbitrary imports, globals,
effectful SDK clients, prompt trust, policy dominance, or resource behavior.

The equivalent Etas shape should make the same product-level ideas explicit as
source semantics:

```etas
type ResearchQuestion = {
    topic: string,
}

type Answer = {
    summary: string,
    confidence: f64,
}

effect WebSearch extends Network {
    action search(query: string) -> string;
}

tool web.search(query: string) -> string ![WebSearch.search] {
    return perform WebSearch.search(query);
}

@model("gpt-5.5")
@tools([web.search])
@limits([Tokens(8_000), ContextTokens(32_000)])
agent Researcher(input: ResearchQuestion) -> Answer {
    let prompt = Prompt.new()
        .system(Trusted("You are a careful research assistant."))
        .data(input);

    let answer = perform infer<Answer>(prompt);
    return normalize_answer(answer);
}
```

Etas should compile this into an agent summary:

```text
Researcher.run:
  input = ResearchQuestion
  output = Answer
  tool_surface = [web.search]
  escaping_effects = [WebSearch.search]
  requested_actions = [Agentic.infer<Researcher.run, Answer>, WebSearch.search]
  limits = [Tokens(8_000), ContextTokens(32_000)]
  prompt_trust = system: Trusted, data: ResearchQuestion
```

The difference is not shorter syntax. It is that the agent boundary becomes a
typed, policy-aware, optimizable inference boundary.

The agent-definition comparison is:

| Framework | Agent definition surface | Extension surface | Implication |
|---|---|---|---|
| LangChain | No explicit body; `create_agent(...)` configures model, tools, prompt, response format, and middleware | Middleware/context engineering callbacks | Good runtime harness, but behavior is spread across host callbacks and ordinary Python |
| Google ADK | No explicit body; `Agent(...)` / `LlmAgent` configures model, instruction, tools, schema, callbacks, planner, and context | Callbacks, tools, workflow agents, custom components | Strong SDK/runtime structure, but not a language-level typed body |
| AutoGen | No explicit body for `AssistantAgent(...)`; it configures model client, tools, system message, and tool loop settings | Custom `BaseChatAgent.on_messages(...)` body in Python | Expressive, but arbitrary host-language behavior is opaque to static effect/policy analysis |
| Etas | `agent { ... perform infer<T>(prompt) ... }` is a source-level agent method body | `impl agent` methods and specs are checked by the compiler; `run` is generated for the ergonomic form | Restricted enough to summarize context, inference, tool surface, effects, policy, limits, and trace |

Therefore Etas should not define `agent` as "a block that only builds a
`Prompt`". That would make it look like a static prompt-template wrapper. The
better boundary is:

```text
agent method = ordinary checked code + agent-scoped infer boundaries
```

Code before `perform infer` may select context, sanitize or summarize it, and
build a `Prompt`. The `perform infer<T>(prompt)` operation records
`Agentic.infer<Agent.method, T>` and invokes the runtime provider through the
default handler. Code after `perform infer` may normalize or repair the typed
model result. The agent block should
not become an arbitrary multi-agent workflow body: cross-agent orchestration,
approval gates, publishing, workspace writes, email sends, and durable business
side effects should remain in surrounding `flow` code unless a future SPEC
explicitly widens the agent boundary.

| Dimension | LangChain / ADK / AutoGen | Etas design target |
|---|---|---|
| Agent representation | Host-language object, class, or SDK configuration | Source-language `agent` construct |
| Model call | Runtime loop step | `Agentic.infer<Agent.method, O>` requested action with typed payload |
| Tool exposure | Python/TypeScript/Go callable schema or workbench | `tool` boundary with type, schema, effect/action row, and policy surface |
| Tool authority | Runtime list, guardrail, callback, middleware, or deployment convention | Static effect inference plus runtime policy enforcement |
| Context harness | Middleware, callback, memory adapter, or ordinary host code | Code before `perform infer` is analyzable context selection, sanitization, and prompt assembly |
| Structured output | Pydantic/schema/runtime validation | Declared output type plus schema/decoder support |
| Conversation state | Thread id, checkpointer, message history, agent state | `Message<T>`, `SessionConfig`, typed memory APIs, and trace semantics |
| Human approval | Middleware/callback/interruption | `policy`, `Approval.request`, handlers, and traceable action evidence |
| Limits | Runtime options such as iteration count, timeout, token budget | Source-level `limit` contracts and runtime accounting |
| Parallelism | Runtime option; user must avoid conflicting state | Effect summaries, policy, and trace can constrain safe scheduling |
| Optimization | Mostly manual middleware/graph rewrite | Agent fusion, context harness optimization, cache planning, and prompt partial evaluation under effect/policy checks |
| Deployment manifest | Platform config or observed runtime metadata | Derived from typed source, effect summaries, tool surface, policy, and limits |

The common framework gap is opacity. A callable tool may reach out to network,
write files, read secrets, or call another SDK client outside the framework's
tool table. Middleware can intercept framework-visible events, but it cannot
generally prove whole-program authority, prompt-trust, approval dominance,
resource, or replay properties across arbitrary host-language callbacks.

Therefore Etas should make three agent properties language-visible:

1. context harness semantics: what context is selected, transformed, trusted,
   redacted, summarized, and placed into which prompt channel;
2. tool surface and effect contract: what model-callable actions may be requested
   and what ordinary effects may escape from generated `Agent.run`;
3. `Agentic.infer<C, O>` as a semantic boundary: every agent call is traceable,
   mockable, replayable, resample-aware, policy-checkable, and optimizable.

## 3. Framework Feature Comparison

Legend:

| Mark | Meaning |
|---|---|
| ✓ | Native, documented, or direct design support |
| △ | Partial support, library/runtime support, or expressible with boilerplate |
| ✗ | Not a core feature or not a primary focus |

| Area | Feature | Etas | LangGraph | AutoGen | CrewAI | Eino | OpenAI Agents SDK | Microsoft Agent Framework | Notes |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| Core model | Single agent abstraction | ✓ | △ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas treats `agent` as a first-class source construct; Eino exposes ADK agents in Go rather than language syntax. |
| Core model | Typed agent input/output | ✓ | △ | △ | △ | △ | ✓ | △ | Etas makes typed output part of the declaration. Eino uses Go type alignment for components/graphs, but this is host API typing rather than PL typing. |
| Core model | Multi-agent teams | △ | ✓ | ✓ | ✓ | ✓ | △ | ✓ | Etas can express teams through flows today; standard group combinators are recommended. |
| Core model | Role-based agents | ✓ | △ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas roles are encoded through prompts/configuration on `agent`. |
| Core model | Agent as first-class language construct | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | Existing tools are host-language frameworks or SDKs; Etas's novelty is the PL layer. |
| Core model | Agent-as-tool / manager pattern | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas can model this as a typed stage/tool adapter; manager-style orchestration is common. |
| Core model | Flow/workflow abstraction | ✓ | ✓ | △ | ✓ | ✓ | △ | ✓ | Etas `flow` is source syntax; LangGraph, CrewAI, Eino, and Microsoft expose workflow concepts directly. |
| Core model | Flow as first-class value | ✓ | △ | △ | △ | △ | △ | △ | Etas allows flow values to be passed, returned, composed, and inferred. |
| Core model | Typed message flow | ✓ | △ | △ | △ | △ | △ | △ | Etas treats `Message<T>` as the typed runtime communication value, while most frameworks expose messages through host-language objects or chat history. |
| Core model | Task/process abstraction | △ | △ | ✓ | ✓ | △ | △ | ✓ | CrewAI has Tasks and Processes; Etas should express these as flows plus stdlib conventions rather than add `task` as a keyword. |
| Core model | Dynamic instructions / prompt templates | ✓ | △ | △ | ✓ | ✓ | ✓ | △ | Etas expresses prompt construction as flows returning the support type `Prompt`; prompt is not a source keyword. |
| Core model | Prompt separated from message | ✓ | △ | △ | △ | △ | △ | △ | Etas distinguishes `Prompt` as model-call input from `Message<T>` as MAS communication state. |
| Core model | Model/provider configuration | △ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas agent fields can reference model profiles, but provider metadata and routing are runtime concerns. |
| Core model | Model settings / tool choice | △ | △ | △ | ✓ | △ | ✓ | △ | Etas can represent these as stage options; explicit tool-choice semantics need design. |
| Composition | Sequential pipeline | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas uses <code>&#124;</code> for stage composition and `~>` for application. |
| Composition | Graph/DAG workflow | △ | ✓ | ✓ | ✓ | ✓ | △ | ✓ | Etas can express DAGs with `let`, `join`, and composition, but has no dedicated graph syntax in MVP. |
| Composition | Conditional routing | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas uses ordinary `if`/`match` and first-class flows. |
| Composition | Supervisor/router pattern | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas expresses supervisor logic as a normal flow. |
| Composition | Handoff between agents | △ | △ | ✓ | △ | △ | ✓ | ✓ | Etas expresses handoff through typed `Message<T>` routing and session support; runtime/AIR must preserve `Handoff` trace events. |
| Composition | Handoff trace event | ✓ | △ | △ | △ | △ | ✓ | △ | Etas models `Handoff` as AIR/trace semantics, not as a source keyword. Existing frameworks may trace handoff operationally. |
| Composition | Round-robin group chat | △ | △ | ✓ | △ | △ | △ | ✓ | Etas can express it with loops and limits; stdlib `group.round_robin` is recommended. |
| Composition | Selector group chat | △ | △ | ✓ | △ | △ | △ | ✓ | Etas can model selector logic as an agent or flow choosing the next speaker. |
| Composition | Swarm/open-ended team | △ | △ | ✓ | △ | △ | △ | △ | Etas should keep this bounded by policy, protocol, and limits rather than make it unconstrained. |
| Composition | Subgraphs/subflows | ✓ | ✓ | ✓ | ✓ | ✓ | △ | ✓ | Etas composition lowers into AIR; subflows remain analyzable. |
| Composition | Parallel fan-out/fan-in | △ | ✓ | △ | ✓ | ✓ | △ | ✓ | Etas currently uses `join([...])` as a stdlib combinator, not a keyword. |
| Composition | Declarative workflow file format | ✗ | △ | △ | ✓ | ✗ | △ | ✓ | Microsoft and CrewAI support declarative/YAML-style workflow definitions. Eino is primarily Go API based. |
| Composition | State reducers / merge semantics | △ | ✓ | △ | △ | △ | △ | △ | LangGraph makes graph state updates central. Etas needs clearer merge rules for concurrent writes and `join`. |
| Control flow | Ordinary `if`/`match` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas keeps ordinary programming constructs inside flows. |
| Control flow | `for`/`while` loops | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas adds static/runtime analysis for non-deterministic loop bodies. |
| Control flow | Explicit loop/resource limit | ✓ | △ | ✓ | △ | △ | △ | △ | Etas has typed `limit` contracts in source. Other frameworks expose useful runtime knobs such as max iterations, timeout, or token limits, but they are usually not a unified analyzable flow contract. |
| Control flow | Termination conditions | ✓ | △ | ✓ | △ | △ | △ | ✓ | AutoGen-style max messages, text mention, and function-call termination maps to ordinary boolean conditions and typed limits. |
| Control flow | Interrupt/pause/resume | △ | ✓ | △ | ✓ | ✓ | △ | ✓ | Etas can express approval and checkpointed resume, but general interrupt semantics need runtime specification. |
| Control flow | Human feedback after agent response | △ | ✓ | ✓ | ✓ | ✓ | △ | ✓ | Tool approval is covered; request-info style review between agents should be modeled as stdlib/runtime support. |
| Tools | Tool declaration | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas tools are external authority boundaries with explicit effects. |
| Tools | Tool schema/typed parameters | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas makes this part of source-level type checking. |
| Tools | Tool effect declaration | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | Existing frameworks may use permissions/guardrails, but not an explicit PL effect row. |
| Tools | Effect/action authority | ✓ | △ | △ | △ | △ | △ | △ | Etas authority is expressed through typed effect/action rows checked with policies. |
| Tools | Tool sandboxing | ✓ | △ | △ | △ | △ | ✓ | △ | Etas defaults `Command` to sandbox policy; OpenAI documents sandbox agents/tools. |
| Tools | Tool guardrails | ✓ | △ | △ | ✓ | △ | ✓ | ✓ | Etas expresses guardrails through policies, requirements, handlers, and typed validators. |
| Tools | Hosted tools | △ | △ | △ | △ | △ | ✓ | △ | OpenAI has hosted tools such as web/file search and code execution. Etas should model these as hosted tool bindings with declared effect/action rows. |
| Tools | MCP tool/server integration | △ | ✓ | ✓ | ✓ | △ | ✓ | ✓ | Etas should support MCP through tool registry/import metadata, not a source keyword. |
| Tools | Deferred tool loading / tool discovery | △ | △ | △ | △ | △ | ✓ | △ | Etas manifests can support discovery, but lazy tool exposure semantics need runtime design. |
| Tools | Tool result behavior control | △ | △ | ✓ | ✓ | △ | ✓ | △ | OpenAI exposes stop-on-tool and tool-use behavior; Etas can model this as stage options or policy. |
| Tools | Built-in browser/computer/code tools | △ | △ | △ | △ | △ | ✓ | △ | Etas can bind these as sandboxed tools with `Command`, `Network`, `FileIO`, or UI effects, but built-ins are platform-specific. |
| Effects and safety | Static effect inference | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | This is a primary PL differentiator. |
| Effects and safety | Effect boundary enforcement | ✓ | ✗ | ✗ | ✗ | ✗ | △ | △ | Etas can reject or deny operations before side effects based on inferred/declared effects. |
| Effects and safety | Runtime-scoped handlers | ✓ | ✗ | △ | △ | △ | △ | △ | Other tools provide callbacks or middleware; Etas models selected operations with `handle`. |
| Effects and safety | Approval before selected effect | ✓ | △ | ✓ | ✓ | △ | △ | ✓ | Etas can make approval dominance a static/runtime policy property. |
| Effects and safety | Prompt injection / trust typing | ✓ | △ | △ | △ | △ | △ | △ | Etas has trust/provenance wrappers; frameworks usually rely on guardrails, prompt discipline, or runtime checks. |
| Effects and safety | Policy as analyzable construct | ✓ | △ | △ | △ | △ | △ | △ | Existing frameworks support policies/guardrails operationally, not as a small source-level policy language. |
| Effects and safety | Input/output guardrails | ✓ | △ | △ | ✓ | △ | ✓ | ✓ | Etas policies and validators can express this; runtime ergonomics should match existing frameworks. |
| Effects and safety | Tripwire / halt-on-violation | ✓ | △ | △ | ✓ | △ | ✓ | ✓ | Etas can model this with `Error<E>`, `abort`, and policy denial. |
| Effects and safety | Secrets management | △ | △ | △ | ✓ | △ | △ | ✓ | Etas has `Secret<T>` and `Secret` effect, but host secret-store integration is runtime/platform work. |
| Effects and safety | Rate limits / quotas | △ | ✓ | △ | ✓ | △ | △ | ✓ | Etas has `limit` and policy hooks; cross-run quota enforcement needs runtime support. |
| Memory | Short-term conversation state | △ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas design now identifies `Message<T>`, `SessionConfig`, and `Conversation`, but runtime behavior still needs implementation detail. |
| Memory | Long-term memory | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas uses compiler-known `MemoryRegion<S>` / `Store<K, V>` support types and typed APIs rather than ambient memory. |
| Memory | Typed memory schema | ✓ | △ | △ | △ | △ | △ | △ | Etas makes memory schema part of the program through standard support types and manifest bindings. |
| Memory | Memory provenance | ✓ | △ | △ | △ | △ | △ | △ | Etas can encode provenance and trust at the type level. |
| Memory | Conversation IDs / thread IDs | △ | ✓ | ✓ | ✓ | △ | ✓ | ✓ | Etas exposes this through `SessionConfig` and `Conversation` support types rather than implicit globals. |
| Memory | Context/dependency injection | △ | △ | ✓ | △ | ✓ | ✓ | ✓ | Etas can pass ordinary context structs, but stage-level context injection should be specified. |
| Memory | Knowledge sources / vector stores | △ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas can model these as tools/memory stores; standard RAG libraries are still needed. |
| Memory | Context window management | △ | △ | △ | ✓ | △ | △ | △ | Etas needs standard context policies such as last-N turns, summaries, retrieval, and compression. |
| Durability | Checkpoint/resume | △ | ✓ | △ | ✓ | ✓ | △ | ✓ | Etas has `runtime.checkpoint(...)`; formal resume semantics still need work. |
| Durability | Failure recovery | △ | ✓ | △ | △ | △ | △ | ✓ | Etas can model recovery, but runtime contracts need to be specified. |
| Durability | Replay/time travel | △ | ✓ | △ | △ | △ | △ | △ | Etas trace semantics make this possible; runtime/tooling remains to be designed. |
| Durability | Idempotent side-effect dedup | △ | △ | △ | △ | △ | △ | △ | Etas should specify AIR node replay and tool-call dedup rules. |
| Durability | Fork from checkpoint | △ | ✓ | △ | △ | △ | △ | △ | LangGraph time travel includes exploring alternative trajectories from checkpoints; Etas trace semantics can support this later. |
| Durability | Pending writes / partial step recovery | △ | ✓ | △ | △ | △ | △ | △ | Etas should define how successful parallel branches are retained when another branch fails. |
| Observability | Tracing | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas trace is semantic, not just logging. |
| Observability | Streaming events | △ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas runtime should expose streaming trace/event APIs. |
| Observability | Metrics/cost accounting | △ | ✓ | △ | ✓ | △ | ✓ | ✓ | Etas has typed limits; metrics export should be runtime support. |
| Observability | Visual graph view | △ | ✓ | △ | ✓ | ✓ | ✓ | ✓ | Etas can generate views from AIR, but this is a platform feature. |
| Observability | Lifecycle hooks/callbacks | △ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Existing frameworks expose hooks heavily. Etas should keep hooks as runtime extension points governed by effects. |
| Observability | Sensitive-data controls in traces | △ | ✓ | △ | ✓ | △ | ✓ | ✓ | Etas has `Secret<T>`, but trace redaction/export policy should be specified. |
| Observability | External tracing integrations | △ | ✓ | △ | ✓ | △ | ✓ | ✓ | Etas should export OpenTelemetry-like spans or a stable trace schema. |
| Evaluation | Evaluation harness | △ | ✓ | △ | ✓ | △ | ✓ | △ | Etas should add mock model/tool, golden trace, replay, and policy simulation modes. |
| Evaluation | Deterministic replay tests | △ | ✓ | △ | △ | △ | △ | △ | Etas's determinism analysis gives a strong basis, but tooling is not specified yet. |
| Evaluation | Prompt/version regression tests | △ | ✓ | △ | ✓ | △ | △ | △ | Etas can test prompts as flows returning `Prompt`, but versioned prompt/eval workflows need tooling. |
| Evaluation | Training/fine-tuning workflow support | ✗ | △ | △ | ✓ | ✗ | ✓ | △ | This is adjacent platform/tooling, not a PL core feature. |
| Deployment | Deployment manifest | △ | △ | △ | △ | △ | △ | △ | Etas should generate this from typed source/AIR; frameworks often rely on platform config. |
| Deployment | Background/async runs | △ | ✓ | ✓ | ✓ | △ | ✓ | ✓ | Etas should treat this as runtime scheduling, not syntax. |
| Deployment | Webhooks/triggers | △ | ✓ | △ | ✓ | △ | ✓ | ✓ | Etas should use runtime adapters and manifests rather than source keywords. |
| Deployment | Hosted platform | ✗ | ✓ | △ | ✓ | ✗ | ✓ | ✓ | Etas is a PL design; hosting is a separate platform layer. |
| Deployment | Connector marketplace | ✗ | ✓ | △ | ✓ | ✗ | ✓ | △ | Etas should integrate with registries, not become one. |
| Deployment | CLI/project scaffolding | △ | ✓ | ✓ | ✓ | △ | ✓ | ✓ | Etas will need tooling, but this should not affect language semantics. |
| Deployment | Environment management | △ | ✓ | △ | ✓ | △ | ✓ | ✓ | Etas manifests can describe requirements; environments are managed by runtime/platform. |
| Deployment | Team/RBAC management | ✗ | ✓ | △ | ✓ | ✗ | ✓ | ✓ | Etas can consume identity/identity/authority context, but user administration belongs to platforms. |
| Deployment | Artifact/workspace management | △ | △ | ✓ | △ | △ | ✓ | △ | Etas can model workspaces through memory/tools and sandbox profiles; hosted workspace UX is platform work. |
| Interoperability | A2A / agent protocol support | △ | △ | △ | △ | △ | △ | △ | Etas can compile to or consume protocols through runtime adapters; protocol interop is not yet specified. |
| Interoperability | OpenAPI/function import | △ | ✓ | ✓ | ✓ | △ | ✓ | ✓ | Etas should generate/import tool signatures from schemas, but this is tooling. |
| Interoperability | Package/module ecosystem | △ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | Etas specifies modules, imports, re-exports, wildcard imports, and package manifests; the package manager implementation is tooling. |
| Multimodal | Image/audio/file inputs | △ | ✓ | ✓ | ✓ | △ | ✓ | ✓ | Etas type system can model these values, but built-in media types and tool support need stdlib/runtime work. |
| Multimodal | Image generation / artifact outputs | △ | △ | △ | ✓ | △ | ✓ | △ | Etas can bind generation as tools; artifact lifecycle is platform/runtime. |
| PL semantics | Static type system for workflows | ✓ | ✗ | ✗ | ✗ | △ | ✗ | ✗ | Eino benefits from Go type checking, but not from a dedicated agent-flow type system. |
| PL semantics | Static type system for messages | ✓ | ✗ | ✗ | ✗ | △ | ✗ | ✗ | `Message<T>` lets Etas type-check payloads while preserving sender, receiver, session, role, trust, and provenance metadata. |
| PL semantics | First-class non-determinism analysis | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | Etas infers `Deterministic` or `NonDeterministic`. |
| PL semantics | Compiler IR for agent systems | ✓ | △ | ✗ | △ | △ | ✗ | △ | Etas AIR is intended to preserve flow/effect/trace semantics. |
| PL semantics | Optimizable deterministic subflows | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | Etas can lower deterministic flows differently from non-deterministic orchestration. |

This table intentionally gives Etas `△` for several runtime/platform areas. A PL should make those features possible and analyzable, but should not absorb every platform feature into the core grammar.

## 4. Etas Coverage Matrix

| Existing multi-agent feature | Etas construct today | Coverage | Notes |
|---|---|---:|---|
| Single agent with tools | `agent`, `tool`, typed output, effects and actions | Strong | Core case. Tool authority is explicit and statically visible. |
| Sequential multi-agent workflow | <code>A &#124; B &#124; C</code>, `x ~> A` | Strong | Composition returns a `flow`, so effects and determinism remain analyzable. |
| Supervisor/router pattern | `flow`, `match`, first-class flow values | Strong | A supervisor is an ordinary flow that chooses another flow or agent. |
| Handoff to specialist | typed `Message<T>`, flow routing, stage composition, session support, optional `protocol` | Mostly strong | Source syntax can express handoff; AIR/trace should preserve explicit `Handoff` events. |
| Typed agent messages | `Message<T>`, session metadata, provenance, AIR message events | Strong | This is now a core runtime semantic direction, not a `msg` or `message` keyword. |
| Human-in-the-loop | `Approval.request`, `approve(...)`, `policy`, `handle` | Strong | Etas can place gates before specific effects/actions, not only at the end of a run. |
| Tool guardrails | `tool` effects, `require`, policy checks, handlers | Strong | Guardrails become typed pre-execution requirements and post-execution validation flows. |
| Structured output | typed `agent` and `tool` return types | Strong | Validation should be part of the runtime and trace. |
| RAG and knowledge tools | typed tools, typed memory APIs, provenance/trust types | Strong | Retrieval is an ordinary tool or flow with `Network`, `FileIO`, or `Memory.read<R>` effects. |
| Memory and state | `MemoryRegion<S>`, `Store<K, V>`, explicit typed memory APIs | Strong | Etas avoids magic ambient memory by using typed support APIs instead of dedicated persistent-state syntax. |
| Checkpoint/resume | `runtime.checkpoint(...)`, trace, durable runtime | Partial | Expressible, but formal checkpoint/replay/idempotency rules need more detail. |
| Graph/DAG workflow | `let`, <code>&#124;</code>, `~>`, `join([...])` | Mostly strong | Efficient enough for code, but no dedicated visual graph syntax in MVP. |
| Group chat/debate | `while`/`for` + `limit` + memory/trace + agents | Partial | Expressible, but common patterns need standard-library combinators. |
| Round-robin / selector team | loops, flow selection, stdlib group combinators | Partial | Should not require users to hand-write boilerplate conversation loops. |
| Swarm/open-ended coordination | optional `protocol`, runtime policies, loops with limits | Partial | Etas should keep this advanced because open-ended coordination weakens static guarantees. |
| Termination conditions | `limit`, `break`, `return`, boolean conditions | Strong | AutoGen-style max messages, text mention, and function-call termination maps to ordinary conditions plus typed limits. |
| Sandboxed code execution | `Command` effect + `Sandbox(...)` support values | Strong | Default command sandbox should remain runtime policy, not syntax. |
| Observability/tracing | trace semantics, AIR nodes, runtime trace export | Strong | Needs standard trace schema for interoperability. |
| Scheduling and triggers | runtime manifest, stdlib adapters | Partial | Should not be a source keyword in MVP. |
| Deployment packaging | compiler-generated manifest | Partial | Needs concrete manifest format. |
| Visual workflow editor | AIR/manifest metadata | Out of core | Platform feature, not language syntax. |
| Connector marketplace | tool registry and effect/action metadata | Out of core | Platform/runtime concern. |
| Team/RBAC administration | effect actions, policies, host identity | Out of core | Etas can consume identity context; platform manages users and roles. |

## 5. Application Boundary

Etas is a good fit for systems where the agent workflow itself is the critical asset:

1. document-centric collaboration, such as research, writing, review, code generation, legal drafting, financial reporting, and compliance workflows;
2. enterprise automations that call high-impact tools such as email, CRM, calendar, databases, ticketing systems, filesystems, browsers, payment systems, or shells;
3. agentic backends where an API request enters a typed flow and produces a structured response or controlled side effect;
4. workflows that need replay, audit, approval, cost limits, trace review, and runtime recovery;
5. systems where prompt injection, tool authority, tenant isolation, and data provenance are not optional concerns.

Etas is a weaker fit as the only abstraction for:

1. fully open-ended agent societies where participants, topology, and goals mutate without a bounded plan;
2. low-code visual products where most users never see source code;
3. connector catalogs, hosted task queues, billing, RBAC screens, and observability dashboards;
4. pure chatbots with little typed workflow, no high-impact tools, and no durable state;
5. research simulations where the goal is unconstrained emergent behavior rather than verifiable production behavior.

## 6. Production Pain Points

Production agent systems repeatedly expose the same gaps:

| Need / Pain Point | Etas coverage | Layer |
|---|---|---|
| ReAct-style loops | `flow`, `while`, `for`, `limit`, `agent.run`, `tool` | Core language |
| Human approval | `approve(...)`, `Approval.request`, policy dominance checks | Runtime support + effect system |
| Token/cost/time guardrails | `limit Tokens(...)`, `Cost(...)`, `WallTime(...)` | Core language + runtime support |
| Structured audit chain | AIR node ids and trace events | AIR/runtime |
| Checkpoint and resume | `runtime.checkpoint(...)`, trace, memory versions | Runtime |
| Retry and fallback | `Result<T, E>`, `retry`, branching, handlers | Core language/runtime |
| Multi-provider routing | agent `model` declarations and backend metadata | Runtime/tooling |
| Tenant isolation | ordinary `TenantContext`, memory scoping, effect/action policy | Library/policy |
| Session state machine | `enum`, private fields, `impl` methods | General language |
| Domain events and outbox | records, memory writes, event bus tools | Library pattern |
| Metrics and observability | trace events, cost accounting, metrics export | Runtime/platform |

These requirements do not justify new keywords by themselves. They mostly require strong general-purpose constructs plus runtime conventions.

### 6.1 Session Object Pattern

A session should not be modeled as a plain public mutable struct. It is a lifecycle-bearing domain object with guarded transitions, identity checks, and recovery semantics:

```etas
enum SessionState {
    Running,
    Waiting,
    Completed,
    Failed(reason: string),
    Cancelled(by: string),
}

type Session = {
    private id: SessionId,
    private tenant_id: TenantId,
    private user_id: UserId,
    private state: SessionState,
    private events: Array<DomainEvent>,
}

impl Session {
    flow pause(self, interrupt: Interrupt) -> Result<unit, SessionError> {
        match self.transit_to(Waiting) {
            Ok(_) => {},
            Err(err) => return Err(err),
        }

        self.emit(InterruptRaised { session_id = self.id, interrupt });
        return Ok(unit);
    }

    private flow transit_to(self, to: SessionState) -> Result<unit, SessionError> {
        if !valid_transition(self.state, to) {
            return Err(InvalidStateTransition { from = self.state, to });
        }

        self.state = to;
        return Ok(unit);
    }
}
```

This is a language pattern built from records, enums, private fields, methods, and result types. It does not require a `session` keyword.

### 6.2 Tenant, Domain Event, And Outbox Patterns

Tenant context should remain an ordinary value carried through flows, tools, typed memory APIs, policies, and traces:

```etas
type TenantContext = {
    tenant_id: TenantId,
    user_id: UserId,
    request_id: TraceId,
}
```

Domain events and outboxes should be library/runtime patterns:

```etas
type ProjectMemorySchema = MemoryRegion<{
    Sessions: Store<SessionId, Session>,
}>;

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project"
    );

flow PersistSession(session: Session) -> unit {
    ProjectMemory.Sessions.write(session);
    EventBus.publish(session.events());
    session.clear_events();
}
```

The runtime trace schema should standardize metadata such as tenant id, user id, request id, flow id, session id, model token usage, tool latency, approval actor, checkpoint id, and memory version.

## 7. Design Implications

The comparison suggests that Etas's current core is directionally sufficient, but several features should be clarified as runtime or library features.

### 7.1 Keep The Core Small

Do not add source keywords for every framework feature. The current core should remain centered on:

- `flow` for typed orchestration;
- `agent` for non-deterministic model-backed reasoning;
- `tool` for external authority boundaries;
- compiler-known memory support types such as `MemoryRegion<S>` and `Store<K, V>` for explicit durable state;
- `policy`, `require`, `effect`, `handle`, and `limit` for governance and recovery;
- <code>&#124;</code> and `~>` for ergonomic composition.

Features such as checkpoints, group chat teams, triggers, domain events, outboxes, tenant context, provider routing, and connector registries should be standard-library/runtime patterns first.

### 7.2 Add Runtime-Level Session Semantics

Existing frameworks expose strong multi-turn conversation patterns. Etas currently models this through typed memory APIs, traces, and flows, but the user-facing story is still implicit.

Recommended addition:

```etas
type SessionConfig = {
    id: SessionId,
    retention: RetentionPolicy,
    context_policy: ContextPolicy,
}

flow SupportConversation(input: UserMessage) {
    let msg = Message.with_session(
        Message.new(input),
        SessionConfig {
            id = current_session(),
            retention = RetainSummaries,
            context_policy = LastTurns(12),
        }
    );

    return msg ~> TriageAgent;
}
```

This should be a runtime support type and stage option, not a `session` keyword. The compiler/runtime can still make session use visible in AIR and traces.

### 7.3 Provide Group Chat Combinators

AutoGen-style teams and Microsoft Agent Framework group chats are common enough that users should not hand-roll them every time.

Recommended standard-library shape:

```etas
flow Debate(question: Question) -> Answer {
    let transcript =
        group.round_robin(
            [Researcher, Critic, Synthesizer],
            question,
            limits = [Iterations(6), Tokens(30_000)],
            stop_when = mentions("FINAL"),
        );

    return transcript ~> FinalAnswerWriter;
}
```

These combinators should lower to ordinary flows with loops, limits, typed memory access, and agent calls. They do not need special syntax until usage proves that syntax would remove real complexity.

### 7.4 Make Handoff A Trace Concept

OpenAI Agents SDK and several multi-agent frameworks treat handoff as a distinct runtime event. Etas can express handoff as flow routing, but traces should preserve it.

Recommended AIR/trace event:

```text
Handoff {
  from: AgentId,
  to: AgentId,
  reason: string,
  input_type: TypeId,
  output_type: TypeId,
  trace_spec_decision: TraceSpecDecisionId,
}
```

This does not require a `handoff` keyword. The compiler can infer handoff-like edges when an agent result routes into another agent or when a supervisor flow selects a specialist.

### 7.5 Formalize Durable Execution

LangGraph-style checkpointing is one of the clearest production requirements. Etas should specify:

- which AIR nodes can be recomputed;
- which nodes must be replayed from trace;
- how non-idempotent tools are deduplicated;
- how approval decisions are persisted;
- how memory versions are checked during resume;
- how handler scopes interact with recovery.

The MVP can keep `runtime.checkpoint(...)` as a library call, but the trace and runtime semantics should be formal enough that users can trust restart behavior.

### 7.6 Make Observability Semantic

Production agent observability should answer more than "which callback logged a line?" It should explain the semantic execution of a run:

- which flow, agent, tool, handler, approval, checkpoint, and policy nodes executed;
- which prompt and context slices were assembled for each model call;
- which model/tool inputs and outputs were validated, redacted, replayed, or rejected;
- which typed memory regions and message/session values were read or written;
- which effects and actions, policies, and limits governed each step;
- which resources were consumed: tokens, context tokens, cost, wall time, retries, and tool calls;
- why recovery chose recompute, replay, deduplicate, or resample.

Most existing frameworks provide observability through callbacks, middleware, logs, hosted dashboards, or tracing SDK integrations. Those are valuable, but they are usually runtime instrumentation over host-language objects. If model calls, tool calls, memory reads, or approvals are hidden inside arbitrary Python, TypeScript, or Go callbacks, the framework can only observe what the callback reports.

Etas should make observability compiler-assisted. Since `agent`, `tool`, `effect`, `limit`, typed messages, typed memory APIs, approval, handlers, and AIR nodes are explicit semantic boundaries, the runtime can emit trace events that are linked back to source spans and verified runtime metadata.

| Observability surface | Typical frameworks | Etas advantage |
|---|---|---|
| Trace structure | Callback or SDK event stream | AIR/source-derived semantic trace with stable node ids |
| Model call visibility | Provider wrapper instrumentation | `PromptBuild`, `AgentCall`, schema validation, context slice, model profile, usage, and replay metadata |
| Tool call visibility | Tool wrapper instrumentation | Typed `ToolCall` with effects and actions, policy evidence, idempotency, and sandbox metadata |
| Message/handoff visibility | Chat history or framework event | `Message<T>`, session metadata, provenance, and inferred `Handoff` trace events |
| Memory visibility | Framework memory adapter logs | Region-sensitive `Memory.read<R>` / `Memory.write<R>` trace facts and memory versions |
| Resource visibility | Post-hoc token/cost metrics | Runtime metering checked against source-level `limit` contracts and remaining budget |
| Policy visibility | Guardrail pass/fail logs | Traceable `PolicyCheck`, `Approval`, `require before/after`, handler, and denial evidence |
| Recovery visibility | Checkpoint status | Per-node recompute/replay/deduplicate/resample decision linked to determinism, effects, and trace values |
| Redaction | Dashboard/export configuration | `Secret<T>`, trust/provenance labels, and trace export policy visible to compiler/runtime |

The runtime should still export to ordinary observability systems such as logs, metrics, traces, dashboards, and OpenTelemetry-style spans. The distinction is that Etas can generate those artifacts from semantic execution metadata rather than relying only on user-written instrumentation. This makes trace review, audit, replay, golden trace tests, cost debugging, and policy incident analysis part of the language/runtime contract.

### 7.7 Generate A Deployment Manifest

Frameworks and hosted agent platforms need to know what a run may do before it starts. Etas should generate a manifest from typed source and AIR:

```text
flow PublishWeeklyReport:
  determinism: NonDeterministic
  effects: [AcademicSearch.search, ProjectWorkspace.write<"reports/**">, CompanyEmail.send<WorkAccount>]
  requested_actions: [Agentic.infer<_, _>, AcademicSearch.search, ProjectWorkspace.write<"reports/**">, CompanyEmail.send<WorkAccount>]
  allowed_effects:
    - AcademicSearch.search
    - ProjectWorkspace.write<"reports/**">
    - CompanyEmail.send<WorkAccount>
  tools:
    - web.search
    - report.write
    - email.send
  memory:
    read:
      - ReportMemory.Sources
    write:
      - ReportMemory.Runs
  policies:
    - Approval.request before CompanyEmail.send<WorkAccount>
  limits:
    - Tokens(80_000)
    - WallTime(minutes(15))
```

This is a major advantage over host-language DSLs because the manifest is derived from source semantics rather than from one observed run.

### 7.8 Add Evaluation And Test Harness Conventions

Multi-agent systems need tests that go beyond ordinary unit tests. Etas should define standard test modes:

- mock model responses for deterministic tests;
- mock tool implementations and failure injection;
- golden trace comparison;
- policy simulation without executing side effects;
- budget regression tests;
- structured output conformance tests;
- replay tests for checkpointed flows.

These are runtime/tooling features, not source-language keywords.

## 8. Current Gaps To Track

| Gap | Layer | Suggested direction |
|---|---|---|
| Conversation/session semantics are specified but not implemented | Runtime support | Implement `Message<T>`, `SessionConfig`, `Conversation`, context policies, retry-safe append, and trace events |
| Group chat boilerplate | Standard library | Add `group.round_robin`, `group.selector`, `group.swarm` with limits |
| Handoff visibility | AIR/trace | Implement explicit `Handoff` trace events inferred from typed message routing |
| Durable resume rules | Runtime semantics | Formalize recompute/replay/dedup/idempotency |
| Parallel failure recovery | Runtime semantics | Define pending writes and partial branch recovery for `join` |
| Deployment manifest | Compiler/runtime | Emit manifest from typed AST/AIR |
| Tool discovery and MCP | Runtime/tooling | Import MCP/OpenAPI tools into typed `tool` declarations and effect/action metadata |
| Tool-choice behavior | Runtime support | Specify stage options for forced tool use, allowed tool subsets, and stop-on-tool behavior |
| Context window policy | Runtime support | Add policies for last-N turns, summarization, retrieval, compression, and session retention |
| Lifecycle hooks | Runtime support | Expose hooks as effect-governed runtime extension points |
| Trace redaction | Runtime/security | Define `Secret<T>` redaction and export policies in trace schema |
| Trigger/schedule integration | Runtime/platform | Use manifest plus adapters, not keywords |
| Visual graph editing | Platform | Generate from AIR and source locations |
| Connector discovery | Platform/runtime | Tool registry plus effect/action metadata |
| Evaluation workflow | Tooling | Add mock, replay, policy simulation, and golden trace modes |
| Prompt/version regression | Tooling | Add prompt versioning, eval fixtures, and structured output regression tests |
| Multimodal artifacts | Standard library/runtime | Define media/file/artifact support types and platform storage conventions |
| A2A/protocol interoperability | Runtime/tooling | Add adapters rather than a broad source-level protocol mandate |
| Open-ended dynamic teams | Advanced runtime | Keep bounded by policies, limits, and optional protocols |

## 9. Coverage Judgment

Etas can cover most current multi-agent work if "cover" means "the workflow can be expressed as typed source, compiled into analyzable AIR, executed under runtime policy, and traced." The language core already covers the dominant patterns: single agents, tool use, typed outputs, sequential collaboration, supervisor routing, graph composition, human approval, budgets, compiler-known memory support types, sandboxed commands, errors, handlers, and trace-aware execution.

Etas should not claim to cover every product feature offered by agent platforms. Hosted deployment, connector marketplaces, no-code builders, RBAC administration, and dashboards are necessary ecosystem pieces, but they are not evidence that the source language needs more keywords.

The strongest positioning is:

> Etas is the typed, effect-aware, trace-aware programming layer for production multi-agent systems. It should compile to manifests, AIR, runtime plans, and traces that agent platforms can host and visualize.

This keeps the PL boundary crisp while leaving room for a rich runtime and platform ecosystem.
