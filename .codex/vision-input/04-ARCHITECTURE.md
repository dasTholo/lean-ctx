# Thinkery AI Agent Platform — Complete Platform Architecture

Status: platform architecture v5
Role: Agent 4 — PLATFORM-ARCHITECTURE
Date: 2026-08-18
Scope: LeanCTX Engine, Agent Runtime, Control Plane, Distribution, Enterprise

This document is the complete architecture for Thinkery as an AI Agent Platform.
It is intentionally implementation-aware.
It maps the existing lean-ctx repository to a five-layer product model, names the
process and data boundaries, and labels what is shipped, partial, external, or to build.

The word “LeanCTX” means the open-source local engine.
The word “Thinkery” means the product composed of the engine and commercial planes.
The word “kit” means a signed Context Kit containing instructions, knowledge, tools,
policies, examples, tests, and metadata for a specialized agent.
The word “agent” means a managed execution identity with a runtime, a kit, a task,
permissions, budgets, and evidence.

## 1. Executive architecture decision

Thinkery is a local-first agent platform with a hosted control plane.

The engine remains the trustable edge.

The edge sees the customer’s data and executes the task.

The control plane decides fleet policy and stores organizational metadata.

The control plane does not need raw customer prompts to operate.

The runtime receives signed, bounded instructions from the control plane.

The runtime enforces those instructions locally and fails closed when required.

The distribution plane ships signed Context Kits and agent templates.

The enterprise plane adds identity, tenants, support, deployment, and SLOs.

The commercial planes add coordination, hosting, governance, and scale.

They never remove a local LeanCTX capability behind an account or license.

The Local-Free Invariant is a product rule and an architecture constraint.

The open runtime owns execution and observation.

The commercial platform owns authoritative decisions.

This is the key authority split:

```text
OSS runtime:
  execute
  compress
  retrieve
  remember locally
  enforce received policy
  emit neutral observations
  create local evidence

Thinkery platform:
  normalize fleet observations
  author policy
  approve policy
  attribute outcomes
  optimize across agents
  settle commercial results
  administer tenants
  operate hosted services
```

The platform must not turn a local estimate into a billing claim merely by copying it.

The platform must produce a separate verified result from preserved evidence.

## 2. The five-layer model

Layer 1 is the LeanCTX Engine.

Layer 1 is open source and local by default.

Layer 2 is the Agent Runtime.

Layer 2 is the execution and lifecycle substrate that hosts agents.

Layer 2 can run on a laptop, a customer worker, or a managed worker.

Layer 3 is the Control Plane.

Layer 3 is commercial, but its local enforcement protocol is open.

Layer 4 is Distribution.

Layer 4 is the signed package and kit ecosystem.

Layer 5 is Enterprise.

Layer 5 is the organizational operating model around the platform.

The layers are not five nested binaries.

They are five responsibility boundaries.

```text
                         ┌───────────────────────────────────────────┐
                         │ LAYER 5 — ENTERPRISE                      │
                         │ identity, tenants, RBAC, SSO, SLO, support │
                         └───────────────────┬───────────────────────┘
                                             │ governed APIs
                         ┌───────────────────▼───────────────────────┐
                         │ LAYER 4 — DISTRIBUTION                     │
                         │ registry, kit builder, marketplace,       │
                         │ templates, signatures, versions           │
                         └───────────────────┬───────────────────────┘
                                             │ signed kits and policy refs
                         ┌───────────────────▼───────────────────────┐
                         │ LAYER 3 — CONTROL PLANE                    │
                         │ fleet, governance, optimization, evidence │
                         └───────────────────┬───────────────────────┘
                                             │ signed assignments,
                                             │ budgets, policy bundles
                         ┌───────────────────▼───────────────────────┐
                         │ LAYER 2 — AGENT RUNTIME                    │
                         │ start, supervise, kit-load, delegate,     │
                         │ retry, report, drain, recover              │
                         └───────────────────┬───────────────────────┘
                                             │ MCP, A2A, OCLA, proxy,
                                             │ local evidence
                         ┌───────────────────▼───────────────────────┐
                         │ LAYER 1 — LEANCTX ENGINE                  │
                         │ context, memory, tools, bus, contracts,   │
                         │ proxy, evidence, local security            │
                         └───────────────────────────────────────────┘
```

The downward direction is authority and desired state.

The upward direction is observation, receipt, and evidence.

The runtime is the only layer allowed to touch customer files and model credentials.

The control plane receives the minimum data required for coordination and audit.

## 3. What runs where

The platform has three physical zones.

Zone A is the local or customer execution zone.

Zone B is the Thinkery control zone.

Zone C is the distribution and identity zone.

Zone A may be fully offline.

Zone B is optional for personal use.

Zone C may be mirrored by a customer.

```text
CUSTOMER EXECUTION ZONE
  developer laptop
  CI runner
  customer Kubernetes worker
  customer VM
  customer on-prem host
  ├─ lean-ctx binary
  ├─ agent supervisor
  ├─ Context Kit cache
  ├─ local memory and graph
  ├─ local model proxy
  ├─ local evidence ledger
  └─ optional outbound connector

THINKERY CONTROL ZONE
  API gateway
  authentication and tenant service
  fleet registry
  policy service
  task scheduler
  telemetry ingest
  optimization service
  trace and evidence index
  dashboard API
  billing and entitlement service

DISTRIBUTION / IDENTITY ZONE
  Context Kit registry
  package blob store
  signature and revocation service
  marketplace catalog
  SSO/SCIM connectors
  customer private registry mirror
```

The local engine works without Zone B and Zone C.

The hosted product uses Zone B and Zone C as additive services.

The customer-hosted product can place all three zones inside customer infrastructure.

Sensitive model prompts stay in Zone A by default.

Control-plane telemetry is event-shaped and policy-filtered.

Raw content export requires an explicit customer policy and approval.

## 4. Deployment modes

Thinkery has four supported deployment modes.

### 4.1 Personal local mode

Runs on a developer laptop.

LeanCTX is started by an agent hook, CLI, or local MCP client.

Memory is local.

The agent may use the local proxy.

No account is required.

No raw telemetry leaves the laptop.

The local dashboard is available on loopback.

Context Kits are installed from a file or registry.

This mode is already the strongest shipped mode.

### 4.2 Team hybrid mode

Runs a local engine per developer or worker.

The Thinkery control plane provides shared policy and fleet inventory.

The local engine buffers events when disconnected.

The customer chooses whether event payloads include content references or excerpts.

Shared kit metadata comes from the hosted registry.

Customer data remains in the customer network unless explicitly exported.

This is the recommended first commercial deployment.

### 4.3 Managed cloud mode

Thinkery runs the supervisor and workers in a managed cloud account.

The customer supplies model-provider credentials through a managed secret path.

The worker still hosts LeanCTX at the execution edge.

The control plane manages desired state, not individual prompt contents.

This mode enables fleet-level optimization and central trace exploration.

The worker can be region-pinned.

The worker can be configured for no-retention or customer-managed storage.

### 4.4 Customer-controlled mode

The full platform runs in the customer cloud, VPC, or on-prem cluster.

Identity, registry, stores, and workers are customer-operated.

Thinkery provides software updates, support, signed releases, and runbooks.

Customer-controlled mode uses the same wire contracts as hosted mode.

The local-free invariant is strongest here because the system is inspectable.

## 5. Physical placement matrix

| Capability | Local | Customer infra | Thinkery cloud | Required outbound data |
|---|---|---|---|---|
| File reads | yes | yes | worker-side | none by default |
| Shell execution | yes | yes | worker-side | none by default |
| Context compression | yes | yes | worker-side | compressed result only if opted in |
| Agent memory | local store | customer store | optional managed store | policy-selected facts |
| Code graph | local index | customer index | optional metadata index | references or summaries |
| MCP server | yes | yes | worker-side | protocol events only |
| Agent bus | local bus | customer bus | managed bus | routed events |
| A2A delegation | local or private | customer network | managed relay | task envelopes |
| OCLA capability calls | yes | yes | worker-side sidecar | observations |
| Model proxy | local loopback | customer gateway | managed gateway | provider request metadata |
| Usage metering | estimate | customer ledger | authoritative ingest | signed observations |
| Evidence receipt | yes | yes | optional mirror | hashes and receipts |
| Dashboard | local | customer | hosted | redacted events |
| Policy authoring | local config | customer control plane | hosted control plane | signed policy bundle |
| Kit build | yes | yes | CI service | package metadata and blobs |
| Kit install | yes | yes | worker-side | verified artifact |
| Marketplace browse | optional | optional | hosted or mirrored | catalog metadata |
| SSO/SCIM | no | customer IdP | Thinkery IdP integration | identity metadata |
| Billing | no | customer procurement | Thinkery billing | usage and plan events |

The execution zone is authoritative for whether an action actually happened.

The control zone is authoritative for organizational policy and accounting decisions.

The distribution zone is authoritative for package identity and release metadata.

## 6. API boundary principles

Every boundary is versioned.

Every request has a tenant, workspace, agent, task, and trace identity where applicable.

Every response declares its data classification.

Every side effect produces a receipt.

Every retry carries an idempotency key.

Every signed object has canonical serialization rules.

Every policy decision has a reason code.

Every locally generated estimate is labeled unverified.

Every commercial verification points back to preserved evidence.

The control plane never calls a local tool directly.

It submits a task or signed assignment to the runtime.

The runtime never invents a commercial policy.

It enforces the latest valid policy it has received.

The registry never executes an agent.

It stores, verifies, indexes, and serves artifacts.

The dashboard never becomes the source of truth for execution.

It reads APIs and evidence indexes.

## 7. Canonical platform objects

### 7.1 Tenant

`tenant_id` is the hard security boundary.

All organization data is tenant-scoped.

Tenant identifiers are never inferred from a user-controlled path.

Tenant scope is checked before storage and before response.

### 7.2 Workspace

`workspace_id` groups repositories, data sources, policies, and agents.

A workspace may map to a customer project, team, or business process.

Workspace membership is independent of agent identity.

### 7.3 Agent identity

`agent_id` is a stable logical identity.

An agent has a public key or attested runtime identity.

The agent record includes role, owner, status, capabilities, and heartbeat.

The existing registry already models status, attestation, and identity checks.

### 7.4 Agent instance

`instance_id` identifies one running process for an agent.

An agent may have many instances over time.

An instance has a lease, process identity, version, kit lock, and health state.

### 7.5 Task

`task_id` identifies one delegated unit of work.

Task state transitions are explicit.

Terminal states are completed, failed, and canceled.

The task carries origin, destination, description, deadline, budget, and lineage.

### 7.6 Context Kit

`kit_id` plus a semantic version identifies a distributable knowledge and behavior set.

The kit has a digest and publisher signature.

The kit declares tools, policies, model constraints, and compatibility.

The runtime locks the exact digest, not just the version range.

### 7.7 Policy bundle

`policy_bundle_id` identifies an authored governance object.

The bundle is signed by an authorized policy issuer.

The runtime validates scope, expiry, nonce, and constraints.

### 7.8 Execution receipt

`receipt_id` identifies a tool or model execution fact.

The receipt includes input digest, output digest, latency, token counts, and status.

It does not require raw content.

### 7.9 Outcome

`outcome_id` identifies the accepted result evaluation.

An outcome is separate from an execution receipt.

An outcome may be locally observed or commercially verified.

Only the platform can issue an authoritative verified outcome.

### 7.10 Evidence bundle

An evidence bundle is a portable, signed, offline-verifiable set of receipts,
policy snapshots, audit-chain entries, and coverage reports.

The bundle is reproducible from bounded inputs.

The bundle declares honest proof limits.

## 8. End-to-end architecture diagram

```text
User / system event
        │
        ▼
Control Plane task API or local agent trigger
        │
        ▼
Task admission: identity, tenant, kit, policy, budget
        │
        ▼
Runtime supervisor starts or reuses agent instance
        │
        ▼
Context Kit resolver verifies digest, signature, compatibility
        │
        ▼
Context compiler loads memory, graph, task references, and policy
        │
        ▼
LeanCTX MCP / shell / proxy gates every tool call
        │
        ├──────────────► local context stores and indexes
        │
        ├──────────────► A2A bus for delegated subtasks
        │
        ├──────────────► OCLA capabilities and adapters
        │
        ├──────────────► model provider through proxy
        │
        └──────────────► evidence ledger and receipts
        │
        ▼
Agent produces result plus structured report
        │
        ▼
Runtime validates report, policy, artifacts, and outcome references
        │
        ├──────────────► local completion and user response
        ├──────────────► signed event buffer
        └──────────────► control-plane ingest when allowed
        │
        ▼
Control Plane correlates traces, evaluates quality, and updates fleet state
```

## 9. Data flow diagram: context request

```text
Agent asks: ctx_read(path)
        │
        ▼
PathJail and workspace trust check
        │
        ▼
Role and policy check
        │
        ▼
Session and task context lookup
        │
        ▼
Intent and read-mode selection
        │
        ├─ full
        ├─ map
        ├─ signatures
        ├─ diff
        ├─ task
        ├─ reference
        ├─ aggressive
        ├─ entropy
        ├─ lines
        └─ auto
        │
        ▼
Cache / AST / graph / compressor pipeline
        │
        ▼
Secret redaction and sensitivity floor
        │
        ▼
Context IR record with source and lineage
        │
        ▼
Evidence receipt with input and output digests
        │
        ▼
Compressed result to agent
```

No control-plane round trip is required for a local read.

## 10. Data flow diagram: model request through the proxy

```text
Agent harness or runtime
        │ provider request
        ▼
Local LeanCTX proxy
        │ authenticate and identify agent/task/tenant
        ▼
Request normalization
        │ provider-specific codec
        ▼
Prompt and tool-result optimization
        │ compression, dedup, cache, routing policy
        ▼
Budget and model-route check
        │ cost estimate and policy decision
        ▼
Provider connector
        │ OpenAI / Anthropic / Gemini / ChatGPT / Azure / Bedrock
        ▼
Upstream response
        │ usage and latency metadata
        ▼
Response shaping and redaction
        │ output optimization and verification
        ▼
Usage receipt and local ledger append
        │ signed observation may be buffered
        ▼
Agent harness receives response
```

The proxy can continue locally when the control plane is unavailable.

Commercial model routing requires a signed assignment or local fallback policy.

## 11. Data flow diagram: agent-to-agent delegation

```text
Parent agent
  │ creates task with parent_task_id and child budget
  ▼
Runtime task admission
  │ target role, kit compatibility, policy, lease availability
  ▼
Agent registry / scheduler
  │ choose healthy instance
  ▼
A2A task envelope
  │ signed identity, privacy, TTL, trace, retry, evidence refs
  ▼
Agent bus or remote relay
  │ local ContextBus, customer transport, or managed relay
  ▼
Child agent runtime
  │ loads child Context Kit and bounded briefing pack
  ▼
Child execution
  │ tools, proxy, memory, OCLA, evidence
  ▼
Child result message and artifact refs
  │ structured facts, findings, next steps, receipts
  ▼
Parent runtime
  │ verifies signature, scope, artifact hashes, and task state
  ▼
Parent synthesis
  │ result includes child lineage and accepted outcome
```

Private messages require an explicit recipient.

Project-scoped messages cannot leak across project roots.

Directed events are visible only to their target agents.

## 12. Data flow diagram: governance loop

```text
Admin authors policy in Control Plane
        │
        ▼
Policy validator and conflict checker
        │
        ▼
Approval workflow and change record
        │
        ▼
Policy signer issues versioned bundle
        │
        ▼
Fleet dispatcher targets tenant/workspace/role/kit
        │
        ▼
Runtime verifies signature and effective time
        │
        ▼
Runtime enforces allow / deny / redact / route / budget rules
        │
        ├─ policy accepted receipt
        ├─ policy rejected receipt
        └─ local fail-closed decision
        │
        ▼
Signed observation is ingested
        │
        ▼
Control Plane reports policy coverage and exceptions
```

Policy authoring and policy enforcement are different responsibilities.

## 13. Data flow diagram: evidence and audit

```text
Tool call / model call / policy event / task transition
        │
        ▼
Canonical event envelope
        │ event_id, trace_id, task_id, agent_id, tenant scope
        ▼
Hash input and output references
        │ no raw content required
        ▼
Local append-only evidence ledger
        │ hash chain, optional Ed25519 signatures
        ▼
Receipt joins context IR, usage, outcome, and policy
        │
        ├─ local proof export
        ├─ handoff bundle
        ├─ A2A snapshot
        └─ signed evidence bundle
        │
        ▼
Redaction and export policy
        │
        ▼
Outbound event buffer
        │ retry-safe, tenant scoped, content-minimized
        ▼
Control Plane ingest
        │
        ▼
Trace explorer, compliance report, verified attribution
```

## 14. Data flow diagram: kit lifecycle

```text
Author source files, instructions, tools, tests, examples
        │
        ▼
Kit Builder
        │ validate manifests, collect layers, compute stats
        ▼
Local test runner
        │ conformance, safety, quality, prompt budget
        ▼
Publisher signing key
        │ Ed25519 package signature
        ▼
Registry publish gate
        │ namespace, version, integrity, audit, malware checks
        ▼
Marketplace catalog
        │ discovery, compatibility, quality, price, visibility
        ▼
Runtime resolver
        │ select exact version and dependencies
        ▼
Download and verify
        │ SHA-256, signature, revocation, policy consent
        ▼
Local kit cache
        │ immutable digest-addressed content
        ▼
Agent instance
        │ kit lock and Context Kit loaded
```

## 15. Data flow diagram: failure and retry

```text
Attempt starts
    │
    ├─ admission failure ───────► terminal rejected, no side effect
    │
    ├─ transient tool failure ──► bounded retry with same idempotency key
    │                              exponential backoff and jitter
    │
    ├─ provider 429 ────────────► retry-after or alternate permitted route
    │
    ├─ provider 5xx ────────────► circuit breaker, alternate route, or pause
    │
    ├─ policy unavailable ──────► cached policy if within grace, else fail closed
    │
    ├─ agent heartbeat lost ────► lease expiry, checkpoint, replacement instance
    │
    ├─ result invalid ──────────► repair attempt or human approval
    │
    └─ budget exhausted ────────► stop, emit receipt, preserve partial artifacts
    │
    ▼
Every branch records reason, attempt number, parent event, and next action.
```

Retries never silently duplicate a side effect.

Side-effecting tools require idempotency or explicit human approval.

## 16. Layer boundary summary

| Boundary | Request | Response | Owner of truth |
|---|---|---|---|
| L1 ↔ L2 | tool, task, agent, kit, policy context | result, receipt, health | runtime execution |
| L2 ↔ L3 | desired state, assignment, observation | acknowledgement, decision | control plane policy |
| L2 ↔ L4 | kit resolve, download, verify | immutable package | registry identity |
| L3 ↔ L4 | approved kit, quality, entitlement | catalog and release metadata | registry and control plane |
| L3 ↔ L5 | tenant, identity, plan, SLO | authorization and support state | enterprise service |
| L4 ↔ L5 | org namespaces and private catalog | publication policy | enterprise governance |

## 17. Existing-versus-target legend

`SHIPPED` means code and a usable local path exist in this repository.

`PARTIAL` means a substrate or contract exists, but the product path is incomplete.

`EXTERNAL` means the design references a separate enterprise or cloud repository.

`BUILD` means a material platform capability is still required.

`OPTIONAL` means the local product does not depend on it.

A contract or file is not proof of a production service.

A test fixture is not proof of fleet operation.

The rest of this document applies this rule consistently.

## 18. Architecture invariants

1. Local reads and local memory do not require cloud connectivity.
2. Model calls can be executed through a local proxy.
3. Policy enforcement fails closed for required controls.
4. Raw prompts are not required for aggregate fleet cost views.
5. Tenant scope is carried on every commercial event.
6. Every agent instance has a bounded lease and heartbeat.
7. Every task has an idempotency key and a state machine.
8. Every kit is selected by digest after verification.
9. Every result has a receipt or an explicit failure receipt.
10. Every retry is bounded by policy and budget.
11. Every export is redacted by default.
12. Every signed artifact is offline-verifiable.
13. Commercial value claims require platform verification.
14. Local capabilities are never gated by plan.
15. Versioned contracts are the only cross-repository seam.

## 19. Non-goals

Thinkery is not a replacement for every agent harness.

Thinkery does not require one universal foundation model.

Thinkery does not put all customer data into one central vector database.

Thinkery does not make raw transcripts the default memory format.

Thinkery does not treat a marketplace listing as a trust decision.

Thinkery does not use a dashboard as an execution control bypass.

Thinkery does not claim that a signed estimate is verified savings.

Thinkery does not make remote orchestration a prerequisite for a local task.

## 20. Layer 1 overview — LeanCTX Engine

Layer 1 is the always-available context operating system.

It is a single Rust binary with multiple delivery surfaces.

The current architecture already has the majority of Layer 1 primitives.

The most important entry points are the local CLI, MCP server, HTTP MCP server,
daemon, shell hook, LLM proxy, SDKs, and dashboard.

The engine is the narrow waist of the platform.

Agent runtimes should consume it through MCP, HTTP, A2A, OCLA, or the thin SDK.

The commercial platform should not import internal engine modules.

It should use versioned wire contracts.
