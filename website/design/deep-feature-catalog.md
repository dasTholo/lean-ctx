# lean-ctx — Complete Product Capability Map

lean-ctx is a local-first context runtime for AI agents. It sits between an
agent and the systems it uses—code, terminal, tickets, knowledge, policies,
and other agents—to decide what information should be delivered, in what form,
under what constraints, and with what proof. Token reduction is an outcome;
the broader product is control, continuity, intelligence, and governance for
AI work.

## Category 1: Context Control and Information-Density Runtime

### What it does

Transforms code, terminal output, search results, and other context into the
smallest useful representation for the current task. It offers ten read modes,
AST/signature extraction across many languages, entropy and task-aware
selection, delta reads, cached compressed representations, shell-output
compression, cross-file de-duplication, and safe anchored editing. It can also
choose whether a source should be compressed in-place or retrieved from a
larger corpus.

### Why it matters (business value)

Developers get more effective reasoning from the same model and context window,
while teams lower latency and avoid agents wandering through raw logs or large
files. Unlike a generic summarizer, this makes each interaction appropriate to
the task: exact detail for edits, structure for orientation, and condensed
evidence for diagnosis. It preserves productivity as repositories and agent
workloads scale.

### Key components/tools

`ctx_read`, `ctx_smart_read`, `ctx_shell`, `ctx_compare`, `ctx_dedup`,
`ctx_delta`, `ctx_context`, `ctx_compose`, the Context Gate, Tree-sitter
parsing, shell-pattern modules, compressed-output cache, and bounce tracking.

## Category 2: Adaptive Context Intelligence

### What it does

Detects intent, selects a read mode and model tier, applies pressure-aware
downgrades, and learns from outcomes. The Context Kernel combines knowledge,
episodic/procedural memory, session state, graph proximity, delivery history,
policy, and token budgets into a bounded context plan. Feedback and bounce
signals adjust compression depth and field weights over time; shadow and
enforce modes make changes safe to introduce.

### Why it matters (business value)

Teams do not have to hand-tune every agent prompt, context budget, or tool
choice. The runtime can improve the quality/cost/latency trade-off from real
usage, reduce context failures under pressure, and prevent “savings” that users
immediately undo by asking for the full source.

### Key components/tools

Context Kernel (`bridge`, `adaptive_bridge`, `orchestrator`, `context_broker`,
`feedback`, `learning`, `knowledge_health`, `degradation`, `receipt_chain`),
`ctx_intent`, `ctx_plan`, `ctx_feedback`, model-routing and response-
optimization OCLA services.

## Category 3: A Governed Context Kernel

### What it does

Acts as a central compiler for agent context. It produces plans with selected,
excluded, and deferred candidates; allocates a hard token budget; injects only
the relevant long-term, episodic, procedural, and session information; records
delivery receipts; tracks provider contributions; and enforces source and
policy restrictions before information reaches a model.

### Why it matters (business value)

This turns brittle prompt assembly into an operational layer with consistent
behavior across tools and clients. Enterprises can set context budgets and
allowed data sources once, then gain predictable agent performance and a
reconstructable record of what influenced a decision.

### Key components/tools

`ContextKernel`, `ContextPlanV1`, `ContextReceiptV1`, retrieval compiler,
context ledger, enforcement policy, MCP bridge, proxy bridge, identity and
attribution wiring, provider traces, evidence hooks, and dashboard reports.

## Category 4: Persistent Project Memory and Knowledge Health

### What it does

Stores long-term facts, patterns, decisions, feedback, relations, and temporal
validity. It supports lexical, semantic, and hybrid recall; local embeddings;
contradiction/supersession judgement; lifecycle decay; archival and automatic
rehydration; consolidation from sessions; import/export; project “rooms”; and
knowledge-quality reports. The memory model distinguishes working session
state from long-term knowledge, episodic history, and learned procedures.

### Why it matters (business value)

It converts repeated agent work into a durable asset instead of letting each
new chat rediscover repository conventions and prior decisions. Quality controls
keep memory from becoming a stale prompt dump, so teams can trust it as their
codebase and organization change.

### Key components/tools

`ctx_knowledge` (remember, recall, search, consolidate, judge, lifecycle,
health, relations, embedding maintenance), `ProjectKnowledge`, knowledge
ranking/chunking/persistence, knowledge snapshots, OKF import/export, and
embedding indexes.

## Category 5: Graph-Native Code Understanding

### What it does

Builds a property graph of files, symbols, imports, calls, exports, type
references, and tests. It powers graph-aware read hints, related-file lookup,
hybrid search ranking, incremental indexing, function/file blast-radius
analysis, call tracing, cycles, centrality, “god node” detection, architectural
clusters/layers/entry points, and code-health hotspots.

### Why it matters (business value)

Agents and developers can answer “what else changes if we touch this?” before
they edit. This reduces regression risk, review time, and the cost of becoming
productive in unfamiliar systems. Structural understanding is substantially
more actionable for code than treating a repository as unconnected text chunks.

### Key components/tools

`ctx_graph`, `ctx_impact`, `ctx_callgraph`, `ctx_architecture`, `ctx_quality`,
`ctx_smells`, `ctx_review`, `graph_index`, `graph_analysis`, the SQLite-backed
property graph, and Tree-sitter extractors.

## Category 6: Hybrid Retrieval Across Code and External Work

### What it does

Combines lexical BM25, dense/local embeddings, and graph proximity with fusion
and reranking. External providers for GitHub, GitLab, Jira, PostgreSQL,
configuration sources, and MCP bridges return structured records that become
the same chunks, knowledge facts, graph edges, cache entries, and searchable
context as local code. Provider health, auth readiness, TTL caching, and
scaffolding are built into the framework.

### Why it matters (business value)

An agent can reason over the real operational context behind a code change—open
issues, pull requests, tickets, schemas, documentation, and CI-related data—
without teams building a separate RAG pipeline for each system. This makes AI
work more accurate and cuts tool-switching and integration maintenance.

### Key components/tools

`ctx_provider`, `ContextProvider`, `ProviderRegistry`, provider cache/health,
GitHub/GitLab/Jira/Postgres/MCP providers, `consolidation`,
`consolidation_engine`, `ctx_semantic_search`, and `ctx_preload`/`ctx_prefetch`.

## Category 7: Session Continuity and the Context Time Machine

### What it does

Persists tasks, findings, decisions, files touched, progress, test evidence,
next steps, intents, configuration, and playbooks. It creates compact recovery
snapshots with executable recall/search commands and graph/knowledge context;
records and recalls session summaries; and supports signed context snapshots
that can be listed, verified, restored, published, imported, and replayed
against repository history. Portable context packages bundle knowledge, graph,
session, patterns, and gotchas with integrity checks and auto-load support.

### Why it matters (business value)

Long-running work survives compaction, model resets, handoffs, and time away
from a project. Teams can resume work quickly, debug what an agent knew at a
given point, reproduce a decision, and distribute hard-won project context
without relying on a vendor-hosted memory black box.

### Key components/tools

`ctx_session`, session persistence/compaction/playbooks, `session_summary`,
`context_snapshot` (builder, timeline, digest, signing, restore, publish),
`ctx_pack`, and `.ctxpkg`/OKF formats.

## Category 8: Multi-Agent Coordination and Handoffs

### What it does

Provides an agent registry with roles, liveness/reaping, persistence, shared
state, and agent diaries. Its A2A layer supports task delegation, updates,
results, questions, context sharing, privacy levels, priorities, TTLs, relay
chains with cycle/hop protection, rate limits, health telemetry, cost
attribution, remote transport, dead-letter handling, budget cascades, agent
cards, and A2A-compatible transfer envelopes. Handoff bundles package
transferable context deterministically for another agent or client.

### Why it matters (business value)

Multi-agent delivery stops being a collection of opaque chats. Managers can
allocate work, control information sharing, limit runaway delegation, assign
costs, and reliably transfer a task without paying to rebuild the recipient’s
understanding. This is the substrate for agent fleets, not just parallel tabs.

### Key components/tools

`ctx_agent`, `ctx_task`, `ctx_handoff`, `ctx_share`, `ctx_workflow`, A2A
messages/relay/transfer/compression/telemetry/rate limiter, agent registry,
roles, bridge, diary, and the OCLA Agent Gateway.

## Category 9: Context OS for Multi-Client Operations

### What it does

Provides a process-local runtime for shared sessions, an event bus, filtered
and directed subscriptions, consistency levels, redacted event payloads, and
metrics. Tool actions are translated into context events such as session
mutation, knowledge creation, graph builds, artifacts, and proofs, enabling
HTTP/daemon/team-server clients to observe the same evolving context state.

### Why it matters (business value)

This creates a shared operational plane for an organization’s human and agent
clients. Work can be coordinated and observed across IDEs, terminals, and
services instead of being trapped in one agent process.

### Key components/tools

`context_os`, `SharedSessionStore`, `ContextBus`, filtered subscriptions,
redaction, Context OS metrics, daemon/HTTP MCP serving, and team-server
surfaces.

## Category 10: Governance, Security, and Proof-Carrying Context

### What it does

Applies built-in and organizational policies to roles, data sources, memory,
budgets, workflow state, degradation behavior, trust, and compliance coverage.
It includes policy packs for baseline, strict redaction, SOC 2, healthcare,
EU finance, EU AI Act deployers, and ISO 42001 alignment. It generates
aggregated compliance reports in rendered/PDF forms and supports redaction,
path jail, shell allowlists, deterministic outputs, audit/evidence bundles,
signed ledgers, and context proof export.

### Why it matters (business value)

Regulated and security-conscious organizations can adopt agent workflows
without treating every model call as an ungoverned data export. It supplies
auditable evidence of controls and context exposure, reducing compliance effort
and making AI operations easier to defend to customers, auditors, and security
teams.

### Key components/tools

`policy` (runtime, floor, coverage, org trust/store/model), policy packs,
`compliance_report`, `ctx_proof`, `ctx_verify`, Context IR, evidence bundles,
redaction, and OCLA policy/ledger contracts.

## Category 11: Open Context Lifecycle Architecture (OCLA)

### What it does

Defines an open, versioned contract boundary for the entire context lifecycle.
The common `OclaService` contract plus fifteen pluggable capability traits cover
observation, usage, metrics, savings evidence, intent, outcomes, compression,
response optimization, model routing, efficiency analysis, configuration
tuning, experiments, connector scheduling, agent messaging, and delivery
deduplication. Canonical token envelopes carry provider-neutral accounting,
identity, tenant, trace, policy, and idempotency information without exposing
payload bytes.

### Why it matters (business value)

Customers can integrate, replace, or independently verify parts of the runtime
without being locked to one agent, model, provider, or dashboard. A stable
contract lets platform teams build a context stack that evolves safely and gives
partners an extensibility surface beyond a proprietary prompt wrapper.

### Key components/tools

`core/ocla`, standalone OCLA crate and client SDKs, registry, sidecar/gRPC/
OpenAPI/wire bridges, cache tiers, delivery registry, response cache, routing
experiments/quality gates, unified ledger, tracing, and health surfaces.

## Category 12: Engineering Workflow, Assurance, and FinOps

### What it does

Orchestrates workflow stages, transitions, completion, and evidence; manages
agent tasks; generates budget-aware plans; supports bounded repository
exploration with citations; finds refactoring targets and LSP references;
reviews changed files with impact/call-graph/smell/test context; tracks
per-agent and per-tool costs; exposes gains, metrics, heatmaps, cache status,
and benchmarks; and discovers uncompressed command families from real history.

### Why it matters (business value)

It moves AI-assisted engineering from “ask an agent and hope” to a measurable,
repeatable delivery process. Leaders can see usage and cost allocation, while
developers receive the risk, test, and architectural context needed to ship
changes with more confidence.

### Key components/tools

`ctx_workflow`, `ctx_task`, `ctx_plan`, `ctx_explore`, `ctx_refactor`,
`ctx_review`, `ctx_cost`, `ctx_metrics`, `ctx_gain`, `ctx_heatmap`,
`ctx_benchmark`, `ctx_discover`, and `ctx_artifacts`.

## Category 13: Cross-Repository and Ecosystem Reach

### What it does

Supports multiple project roots with searchable cross-repository context,
portable knowledge/context packages, and a broad integration surface: MCP,
CLI, shell hooks, daemon and HTTP serving, SDKs, and editor/agent-specific
setup and capability detection. Dynamic tool categories let clients load
specialized tools only when needed.

### Why it matters (business value)

Organizations can deploy one context layer across monorepos, service estates,
and heterogeneous developer environments rather than adopting a different AI
workflow per IDE or language. Selective tool loading also keeps the agent’s own
tool context manageable as the platform grows.

### Key components/tools

`ctx_multi_repo`, multi-root search, context packages, MCP resources/prompts,
dynamic tool loading, shell hooks, daemon, HTTP/team server, TypeScript/Python/
Rust/Go SDKs, and integrations for major coding agents and IDEs.

## The “Aha!” Moments — Things Nobody Else Has

1. **A compiler for context, not a summarizer.** The kernel selects, budgets,
   suppresses, enforces, and receipts context from multiple stores.
2. **One pipeline for “compress what fits” and “retrieve what does not.”** It
   avoids the common RAG mistake of retrieving/summarizing information that
   could have been delivered structurally and faithfully.
3. **Honest optimization through bounce accounting.** Savings are adjusted when
   users re-read compressed content, so the system can optimize for successful
   work rather than flattering token metrics.
4. **Graph-aware context is continuous, not an optional analysis screen.**
   Structure influences reads and retrieval as well as blast-radius and
   architecture analysis.
5. **Memory with a lifecycle.** Facts can be scored, judged, superseded,
   archived, restored, related, and consolidated rather than accumulating as
   untrusted chat history.
6. **A Context Time Machine.** Signed, git-aware snapshots make it possible to
   inspect, reproduce, restore, and share the effective context behind past
   agent work.
7. **Agent handoffs as governed, interoperable artifacts.** A2A-compatible
   bundles carry task context with privacy, cost, relay, and lifecycle controls.
8. **Proof-carrying context.** Policy determines what an agent may see; signed
   evidence and compliance reports can demonstrate what it did see and why.
9. **External work systems become first-class context.** Tickets, PRs, issues,
   databases, and MCP data enter the same retrieval and knowledge fabric as
   source code rather than living in side-channel tabs.
10. **An open lifecycle contract.** OCLA turns the context layer into an
    extensible platform with stable provider/model/agent boundaries.

## Product Positioning Options

1. **The Context Control Plane for AI Engineering** — govern what every agent
   sees, remembers, costs, and proves across code and work systems.
2. **The Context OS for Agent Fleets** — persistent memory, coordination,
   policy, and observability for multiple agents and clients working together.
3. **The Codebase Intelligence Runtime** — graph-native retrieval, impact
   analysis, architecture health, and workflow assurance for reliable code
   changes.
4. **The AI Continuity Layer** — turn fragile chats into durable, portable,
   replayable project understanding that survives sessions and handoffs.
5. **Governed Context Infrastructure** — local-first, auditable, policy-driven
   context delivery for enterprises that need AI capability without surrendering
   control of their engineering knowledge.

