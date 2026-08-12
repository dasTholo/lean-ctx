# LeanCTX — Detailed Roadmap with OSS/Commercial Strategy

**Version:** 1.0  
**Date:** 12 August 2026  
**Purpose:** Phase-by-phase implementation plan with explicit OSS/Commercial boundaries per deliverable.  
**Audience:** Planning agents, implementation agents

---

## Current State (as of 12 August 2026)

### What Is DONE (Sprint R34/R35)

- TriageEngine with RuleAnalyzer + Fusion (OSS, Class A)
- Knowledge Router V0: deterministic reference routing (OSS, Class A)
- Value Gate V0: local cost + outcome + CPAO (OSS, Class A)
- Shadow Mode: reference counterfactual (OSS, Class A)
- Decision Loop: E2E orchestration (OSS, Class A)
- Evidence Ledger + Integration Proof (OSS, Class A)
- 250→500-task Gold Validation Set (OSS, Class A) ✅ expanded R35
- Distillation Pipeline scaffolding (OSS, Class A)
- Confidence Calibration framework (OSS, Class A)
- CLI commands: savings, prove, value-report, shadow, evidence-export, output-savings (OSS, Class A)
- Protocol contracts: triage.rs, knowledge_routing.rs (OSS, Class B)
- Tool count unified across registry/docs/README (OSS, Class A) ✅ R35
- Reproducible token benchmark module (10 canonical tasks, 3 profiles) (OSS, Class A) ✅ R35
- Deprecated tool registry cleanup + backward compat (OSS, Class A) ✅ R35
- CLI completions spec hardened (OSS, Class A) ✅ R35
- Context Kernel E2E tests strengthened (OSS, Class A) ✅ R35
- SemanticAnalyzer stub with ONNX hook (OSS, Class A) ✅ R36
- benchmark CLI command with --real flag (OSS, Class A) ✅ R36
- Pro conversion triggers + metering (OSS, Class A) ✅ R36
- Phase 1-3 Exit Gates formally verified with integration tests ✅ R36
- Distillation pipeline: train/val/test split, data augmentation (OSS, Class A) ✅ R36
- ONNX Model Loader + `lean-ctx model install/status/remove` CLI (OSS, Class A) ✅ R37
- Semantic Shadow Parallel Mode + accuracy tracker (OSS, Class A) ✅ R37
- A/B Measurement Framework (baseline/treatment recorder, comparison) (OSS, Class A) ✅ R37
- `lean-ctx measure` CLI (baseline-start/stop, treatment-start/stop, compare, report) ✅ R37
- `lean-ctx triage accuracy` CLI (shadow agreement tracking) ✅ R37
- Enterprise Control Plane contracts in protocol crate (OSS, Class B) ✅ R37
  - ControlPlaneContract, OutcomeEngineContract, FleetControlContract, ValueShareContract
  - Local (OSS) implementations + trait-based extension points

### What Is NOT Done

- Semantic ONNX Model (not trained)
- Configured live provider operation in Knowledge Router (needs OAuth/tokens)
- First Customer POC
- Pro subscription product
- Enterprise Control Plane (adaptive intelligence)
- Verified attribution / billing

---

## Phase 0: Claim Hygiene (Week 1–2) ✅ COMPLETED (Sprint R35)

**Goal:** Make every pitch claim defensible. Without this, everything else is theater.

| Deliverable | Repo | Class | Exit Gate |
|-------------|------|-------|-----------|
| Tool-count unified (README = Registry = Cargo) | OSS | A | Single number everywhere |
| Reproducible token benchmark (10 tasks, before/after, raw data) | OSS | A | Script + results published |
| Traction evidence pack (install definition, time series, methodology) | Internal | — | Document exists |
| Claim register (every pitch number → source + allowed phrasing) | Internal | — | Document exists |
| Remove deprecated tools from registry | OSS | A | Count matches |

### OSS/Commercial for Phase 0

Everything in this phase is OSS or internal documentation. No commercial code needed.

---

## Phase 1: Prove the Value — E2E Decision Loop (Week 2–6) ✅ EXIT GATE VERIFIED (R36)

**Goal:** One single task traceable from context to accepted outcome. This is the MINIMUM viable proof.

| Deliverable | Module | Repo | Class | Exit Gate |
|-------------|--------|------|-------|-----------|
| TaskEnvelope created at MCP ingress | `core/task_spine.rs` | OSS | A | Every MCP call gets task_id |
| TriageEngine produces TaskProfile | `core/triage/` | OSS | A | Rules classify every task |
| Execution → Receipt → Outcome join | Kernel wiring | OSS | A | Receipt references task_id |
| Value Gate: live cost + outcome + evidence | `core/value_gate/` | OSS | A | CPAO calculable per task |
| `lean-ctx prove` runs E2E with real tasks | `cli/prove.rs` | OSS | A | 5/5 tasks pass |
| CPAO visible in CLI | `cli/value_report.rs` | OSS | A | Number displayed |

### Exit Gate Phase 1

> A real coding task with: task_id propagated → cost measured → outcome evaluated → CPAO calculated → auditably exported.

### OSS/Commercial for Phase 1

**100% OSS.** This phase builds the LOCAL measurement infrastructure. No adaptive intelligence, no billing, no cross-customer learning.

---

## Phase 2: Prove the Context — Knowledge Routing V0 (Week 5–8) ✅ EXIT GATE VERIFIED (R36)

**Goal:** LeanCTX shows it understands what a task needs to know and can assemble targeted context.

| Deliverable | Module | Repo | Class | Exit Gate |
|-------------|--------|------|-------|-----------|
| Reference Resolver (Jira-Keys, PRs, files, symbols) | `knowledge_router/reference_resolver.rs` | OSS | A | Extracts IDs from task text |
| Source Manifest (providers declare capabilities) | `knowledge_router/source_manifest.rs` | OSS | A/B | 7 providers have manifests |
| Deterministic Query Planner | `knowledge_router/planner.rs` | OSS | A | Budget-aware source selection |
| Context Bundle materialization | `knowledge_router/context_bundle.rs` | OSS | A | Packed with token accounting |
| Context Receipt (audit trail) | `knowledge_router/receipt.rs` | OSS | A/B | Shows what/why/from-where |
| Provider Bridge (real queries via ContextProvider) | `knowledge_router/provider_bridge.rs` | OSS | A | Actually calls Jira/GitHub |
| Live OAuth for at least Jira + GitHub | Provider config | OSS (BYOK) | A | Real data returned |

### Exit Gate Phase 2

> Demo: Task with Jira reference → LeanCTX resolves Jira issue + relevant files + security policy → assembles Context Bundle with token budget → Context Receipt shows the decision.

### OSS/Commercial for Phase 2

**100% OSS.** All routing here is deterministic (pattern-based, reference-based). No adaptive source selection, no authority governance, no outcome-trained ranking.

**What would be Commercial (NOT built in this phase):**
- "Based on past outcomes, Jira was useful for this task class 82% of the time" → Class D
- "This policy supersedes that one because of organizational authority" → Class D
- "Source X has been less valuable lately, reduce its priority" → Class D

---

## Phase 3: Prove the Decision — Shadow Recommendations (Week 7–10) ✅ EXIT GATE VERIFIED (R36)

**Goal:** LeanCTX can show what it WOULD have done differently and explain WHY.

| Deliverable | Module | Repo | Class | Exit Gate |
|-------------|--------|------|-------|-----------|
| Shadow Scheduler produces alternative plans | `core/shadow/` + OCLA | OSS | A | Plans generated |
| Baseline vs. Treatment comparison report | `core/shadow/comparison.rs` | OSS | A | Report shows delta |
| Multi-Bundle evaluation (joint context + compute) | Shadow + Knowledge Router | OSS | A | Multiple bundles scored |
| Quality Floor Guardrail | Policy engine | OSS | A | Never recommends below quality |
| `lean-ctx shadow` shows recommendations | `cli/shadow_report.rs` | OSS | A | Human-readable report |
| CPAO comparison (baseline vs. treatment) | Evidence pipeline | OSS | A | Numbers match |

### Exit Gate Phase 3

> Report shows: "For 50 tasks, LeanCTX would have saved $X at equal or better quality."

### OSS/Commercial for Phase 3

**OSS:** Local reference counterfactual. "Given static model prices and deterministic rules, this is what would have happened."

**What would be Commercial (NOT built in this phase):**
- Verified attribution for billing ("we provably saved you $X, invoice coming") → Class D
- Outcome-trained recommendations ("model Y is better for this because…") → Class D
- Settlement calculation → Class D

---

## Phase 4: First Paid Deployment — Customer POC (Week 8–12)

**Goal:** One real customer with measurable value. Revenue.

| Deliverable | Owner | Repo | Exit Gate |
|-------------|-------|------|-----------|
| POC scope and measurement agreement signed | Business | — | Legal document |
| Provider keys configured (real OpenAI/Anthropic) | DevOps | OSS config | API calls work |
| Baseline measurement (1 week without LeanCTX) | Customer + Engineering | — | Raw data collected |
| Treatment measurement (1 week with LeanCTX) | Customer + Engineering | — | Raw data collected |
| Value Report generated | Product | OSS tool output | Savings + CPAO + evidence |
| Customer testimonial / case study | Business | — | Written approval |

### Exit Gate Phase 4

> Signed value report with: Baseline $X → Treatment $Y → Savings $Z → Quality maintained.

### OSS/Commercial for Phase 4

The POC uses **OSS tools** (prove, value-report, shadow) to generate evidence. No commercial intelligence needed yet — the measurement is deterministic.

What makes this COMMERCIAL is the BUSINESS relationship: deployment services, support, and eventually the Value Share contract.

---

## Phase 5: Semantic Triage Model (Week 10–16) — 🔨 SCAFFOLDING COMPLETE (R37)

**Goal:** Semantic understanding for ambiguous tasks where rules are insufficient.

| Deliverable | Module | Repo | Class | Exit Gate |
|-------------|--------|------|-------|-----------|
| Teacher labels 50k+ tasks (4 languages) | Training pipeline | Internal/OSS scaffold | A (pipeline) / E (data) | Dataset exists |
| leanctx-triage-tiny-v1 trained | ML training | Training infra | A (frozen model) | Model exists |
| ONNX export + INT8 quantization | Model pack | OSS distribution | A | < 8 MB |
| `lean-ctx model install triage` | CLI | OSS | A | Downloads + verifies |
| Shadow mode (parallel to rules, no behavior change) | `core/triage/semantic.rs` | OSS | A | Accuracy logged |
| Confidence calibration on gold set | `core/triage/calibration.rs` | OSS | A | Brier score acceptable |
| Promotion gate: better than rules on ambiguous tasks | Release gate | — | — | >10% F1 improvement |

### Exit Gate Phase 5

> Semantic triage beats rules on ambiguous tasks by >10% F1, no regression on clear tasks, <5ms latency, <8MB model.

### OSS/Commercial for Phase 5

**OSS:** The frozen trained model (Class A). Once published, it does not improve without explicit new release.

**Commercial (future, NOT this phase):**
- Continuously updated model from customer outcomes → Class D/E
- Per-customer fine-tuned classifier → Class E
- Cross-customer aggregate training → Class D

---

## Phase 6: Pro Subscription (Week 12–20)

**Goal:** Convert single developers from free to $19–49/mo.

| Deliverable | Repo | Class | Exit Gate |
|-------------|------|-------|-----------|
| Managed OAuth connectors (GitHub, Jira) | Commercial/Pro | D | One-click connect |
| Background delta sync | Commercial/Pro | D | Sources stay fresh |
| Personal Knowledge Graph (cross-device) | Commercial/Pro | D | Graph persists across sessions |
| Personal adaptive source routing | Commercial/Pro | D | Evidence-based improvement |
| Personal AI Value Gate dashboard | Commercial/Pro | D | "Pro saved you $X" |
| Encrypted backup/sync | Commercial/Pro | D | Cross-device works |
| Stripe checkout + entitlements | Commercial/Pro | D | Full lifecycle |
| Conversion triggers in free product | OSS | A | Visible after 5 sessions |

### OSS/Commercial for Phase 6

**OSS:** Conversion triggers ("Pro would sync this across devices"), basic metering that shows potential Pro value.

**Commercial (Pro backend):** All adaptive intelligence, sync infrastructure, OAuth management, personal learning.

---

## Phase 7: Enterprise Control Plane V1 (Week 16–24)

**Goal:** First recurring enterprise software contracts.

| Deliverable | Repo | Class | Exit Gate |
|-------------|------|-------|-----------|
| Adaptive model/provider routing | Enterprise | D | Better CPAO than static |
| Outcome Engine (outcome → learning signal) | Enterprise | D | Predictions improve |
| Knowledge Hub MVP (authority + freshness) | Enterprise | D | Org knowledge governed |
| Enterprise Trust (identity, policy, audit) | Enterprise | D | Roles enforced |
| Fleet Control basics (agent policies, budgets) | Enterprise | D | Quotas work |
| Enterprise Economics dashboard | Enterprise | D | Central visibility |
| Stripe enterprise billing | Enterprise | D | Invoices generated |

### OSS/Commercial for Phase 7

**100% Commercial (Class D).** This is where the business lives.

The OSS Runtime continues to be the edge that collects evidence and executes locally. The Enterprise Control Plane makes the intelligent decisions and provides governance.

---

## Phase 8: Joint Optimization + Value Share (Week 24–36)

**Goal:** Prove joint optimization AND launch verified Value Share billing.

| Deliverable | Repo | Class | Revenue Impact |
|-------------|------|-------|----------------|
| Joint Context + Compute Scheduler | Enterprise | D | Better CPAO → higher provable savings |
| Multi-bundle evaluation with outcome prediction | Enterprise | D | More savings → more Value Share |
| Provider economics (real-time pricing, capacity) | Enterprise | D | Enables provider margin capture |
| Verified Attribution Engine | Enterprise | D | **Enables Value Share billing** |
| Baseline Reconstruction (proven methodology) | Enterprise | D | "Without LeanCTX it would have cost $X" |
| Settlement Engine (calculate, invoice, reconcile) | Enterprise | D | **Automated billing** |
| Capability ecosystem (external optimizers as OCLA) | Enterprise | D | More optimization surface |

### Value Share Revenue Target

```
By end of Phase 8:
- 5+ enterprise customers with verified savings
- Average proven savings: $50k/month per customer
- Thinkery Value Share (20%): $10k/month per customer
- Phase 8 Value Share ARR target: $600k–$1.2M
```

### Exit Gate Phase 8

> Joint optimization PROVES lower Cost per Accepted Outcome than context-only OR gateway-only. Value Share invoices are being sent and paid.

---

## Phase 9: Managed Inference (Week 36–52)

**Goal:** Become the execution layer. Capture margin on AI compute that flows through LeanCTX.

| Deliverable | Repo | Class | Revenue Impact |
|-------------|------|-------|----------------|
| Provider commitment contracts (wholesale pricing) | Enterprise | D | Lower cost basis |
| Multi-provider execution routing (optimized) | Enterprise | D | Route to cheapest sufficient |
| Quality SLA engine (guarantee outcome quality) | Enterprise | D | Justifies premium |
| Customer billing (usage-based, less than direct) | Enterprise | D | **Usage revenue** |
| Sovereign/private execution options | Enterprise | D | Enterprise upsell |
| Fleet Control (multi-team, multi-agent policies) | Enterprise | D | Scale multiplier |

### Managed Inference Economics

```
WITHOUT LeanCTX:
  Customer pays $1M/mo directly to OpenAI/Anthropic/Google
  No optimization. No bulk discount. No routing intelligence.

WITH LeanCTX MANAGED:
  Customer pays $700k/mo to Thinkery (30% less than their current bill)
  Thinkery pays $500k/mo to providers (wholesale + intelligent routing)
  ────────────────────────────────────────────────────────
  Customer saves: $300k/mo
  Thinkery margin: $200k/mo = $2.4M/year PER CUSTOMER
```

The customer pays LESS. Thinkery earns MORE. Everyone wins except the providers' margin.

### Exit Gate Phase 9

> LeanCTX manages inference for 10+ enterprise accounts. Monthly managed compute volume: $5M+. Gross margin on managed inference: >30%.

---

## Phase 10: Ecosystem + Cross-Industry (Week 52+)

**Goal:** From coding agents to every agentic workload. Capability marketplace. Global scale.

| Deliverable | Repo | Class |
|-------------|------|-------|
| Capability Marketplace (third-party optimizers) | Enterprise | D |
| Cross-customer learning (anonymized, privacy-safe) | Enterprise | D/E |
| Cross-industry expansion (beyond coding: legal, finance, ops) | Enterprise | D |
| Outcome API ("outcome in, efficient compute out") | Enterprise | D |
| Global fleet orchestration | Enterprise | D |
| Partner program (system integrators, consultancies) | Business | — |

### Revenue at Scale (Year 3–4 Target)

```
Platform ARR:        100 customers × $200k = $20M
Value Share:         100 customers × $100k = $10M  
Managed Inference:   50 customers × $2.4M margin = $120M
Pro Subscriptions:   10,000 devs × $588/year = $5.9M
────────────────────────────────────────────────────
Total: $155M ARR → $3B+ Valuation
```

This is the unicorn math. Managed Inference is the lever that turns a $30M SaaS into a $150M+ infrastructure business.

---

## Summary: OSS vs Commercial per Phase

| Phase | Timeline | OSS Content | Commercial Content | Revenue Target |
|-------|----------|-------------|-------------------|----------------|
| 0 | Week 1–2 | Claim hygiene, benchmark | — | $0 |
| 1 | Week 2–6 | E2E Decision Loop (local) | — | $0 |
| 2 | Week 5–8 | Knowledge Routing V0 (deterministic) | — | $0 |
| 3 | Week 7–10 | Shadow Recommendations (reference) | — | $0 |
| 4 | Week 8–12 | POC tooling | Business relationship | First $15k |
| 5 | Week 10–16 | Frozen semantic model | — | $0 |
| 6 | Week 12–20 | Conversion triggers | Pro backend (adaptive) | $50k MRR (Pro) |
| 7 | Week 16–24 | — | Full Enterprise Control Plane | $200k MRR (Platform) |
| 8 | Week 24–36 | — | Joint optimization + Value Share | $500k MRR |
| 9 | Week 36–52 | — | Managed Inference | $2M+ MRR |
| 10 | Week 52+ | — | Ecosystem + cross-industry | $5M+ MRR |

### The Pattern — Three Acts

```
ACT I (Phases 0–5): PROVE THE EDGE — Weeks 1–16
  → Prove the runtime works (benchmark)
  → Prove the loop closes (E2E)
  → Prove the measurement is credible (shadow)
  → Get the first paying customer
  → Revenue: $0 → first $15k

ACT II (Phases 6–8): BUILD THE BRAIN — Weeks 16–36
  → Pro: personal adaptive intelligence (subscription revenue)
  → Enterprise: organizational intelligence (ARR)
  → Value Share: proven savings attribution (performance revenue)
  → Revenue: $50k → $500k MRR

ACT III (Phases 9–10): OWN THE COMPUTE — Weeks 36+
  → Managed Inference: execution layer with margin
  → Fleet: every AI task in an org flows through LeanCTX
  → Ecosystem: third-party capabilities as supply
  → Revenue: $2M+ MRR → Unicorn territory
```

### The Revenue Inflection Points

```
Inflection 1: First paid deployment (Phase 4)
  → Proves product-market fit exists
  → Investor signal: "someone pays"

Inflection 2: Value Share billing live (Phase 8)
  → Proves the economic model works
  → Revenue compounds with customer AI spend growth

Inflection 3: Managed Inference launch (Phase 9)
  → Transforms from SaaS to infrastructure business
  → Revenue scales with total AI compute volume
  → This is where unicorn economics emerge
```

---

## Critical Path

```
Phase 0 → Phase 1 → Phase 3 → Phase 4
                ↗
Phase 2 ──────┘ (parallel from week 5)
Phase 5 ────────────────────┐ (parallel from week 10)
                            ↓
Phase 6 ────────────────────┤ (parallel from week 12)
                            ↓
Phase 7 ────────────────────┘ (starts after Phase 4 customer proof)
```

**Shortest path to revenue:** Phase 0 → 1 → 3 → 4 (12 weeks)

---

## What Is Explicitly NOT on This Roadmap (Yet)

1. ~~GPU Fleet~~ — not relevant until Managed Inference at scale
2. ~~Foundation Model Training~~ — use existing models as teacher for triage
3. ~~40+ Connectors~~ — start with Jira + GitHub + Repo; organic growth
4. ~~Universal Company Brain~~ — own the overlay, not the data
5. ~~Coding Agent~~ — LeanCTX makes agents better, not replaces them
6. ~~Consumer product~~ — B2B infrastructure play, not B2C

---

## Investor Timeline Alignment

| Pitch Promise | Phase | Proof | Revenue Implication |
|---------------|-------|-------|---------------------|
| "50–80% reduction" | Phase 0 | Reproducible benchmark | Credibility |
| "Context → Cost → Outcome" | Phase 1 | E2E task with receipt + CPAO | Value measurable |
| "Prove the value" Q3–Q4 2026 | Phase 4 | First paid deployment | First $15k |
| "Pro launch" Q4 2026 | Phase 6 | Subscribers paying | Recurring revenue |
| "Prove the decision" Q1 2027 | Phase 7 | Lower CPAO without quality loss | Enterprise ARR |
| "Value Share live" H1 2027 | Phase 8 | Verified savings invoiced | Performance revenue |
| "Managed Inference" H2 2027 | Phase 9 | Compute flowing through | Infrastructure revenue |
| "Unicorn" 2028+ | Phase 10 | $50M+ ARR run rate | Billion-dollar valuation |

---

## Fundraising Milestones

| Round | When | Proof Required | Valuation Range |
|-------|------|----------------|-----------------|
| **Pre-Seed** (current) | Now | OSS traction + enterprise pull + vision | CHF 4.5M |
| **Seed** | After Phase 7 (Q1 2027) | Recurring enterprise revenue + Pro MRR | CHF 15–25M |
| **Series A** | After Phase 9 (H2 2027) | $2M+ MRR + proven Value Share + Managed Inference POC | CHF 50–80M |
| **Series B** | 2028 | $10M+ ARR + international expansion | CHF 200–400M |

Each round funds the NEXT revenue inflection point, not the same features longer.

---

## Success Criteria by End of Q4 2026

| # | Proof | Artifact | Investor Impact |
|---|-------|----------|-----------------|
| 1 | Token reduction is measurable | Benchmark report with raw data | "The technology works" |
| 2 | Cost + Outcome are connected | E2E task with receipt + CPAO | "They can measure value" |
| 3 | LeanCTX can recommend better paths | Shadow report with alternatives | "The system can decide" |
| 4 | One customer has paid | Contract or invoice | "Someone pays for this" |
| 5 | Enterprise pipeline exists | LOIs, POC agreements | "Demand is real" |

---

## Appendix: Boundary Rules for Agents

### When implementing, ALWAYS ask:

1. **Does this EXECUTE or DECIDE?**
   - Execute → OSS (Class A)
   - Decide → probably Commercial (Class D)

2. **Does the value COMPOUND over time?**
   - No (static rules, frozen model) → OSS
   - Yes (learns from outcomes, adapts) → Commercial

3. **Does it need CUSTOMER DATA to work?**
   - No (works with local signals) → OSS
   - Yes (needs cross-task/cross-customer history) → Commercial

4. **Could this create a BILLING claim?**
   - No (informational, local reporting) → OSS
   - Yes (verified savings for invoicing) → Commercial

5. **Is this the SAME for all users?**
   - Yes (deterministic, same input → same output) → OSS
   - No (personalized, adaptive) → Commercial

### File Path Convention

```
OSS:
  rust/src/core/triage/          → A (local execution)
  rust/src/core/knowledge_router/ → A (deterministic)
  rust/src/core/value_gate/      → A (local observation)
  rust/src/core/shadow/          → A (reference counterfactual)
  lean-ctx-protocol/src/         → B (open contracts)

COMMERCIAL:
  lean-ctx-enterprise/           → D (private intelligence)
  Operational stores             → E (customer data)

INTERNAL PLANNING:
  docs/internal/                 → Not shipped, not in OSS surface
```
