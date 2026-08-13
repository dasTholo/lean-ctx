# Implementation Progress Tracker

> Stand: 2026-08-12 — verified against all three repositories.

## Current round: Sprint R38 — Commercial Features

**Status: ✅ COMPLETE.** Pro backend implemented in `lean-ctx-cloud`, Enterprise
compilation fixed and test coverage expanded in `lean-ctx-enterprise`.
All three repos pass quality gate (clippy clean, tests green).

## Repository health

| Repo | Tests | Clippy | Role |
|---|---|---|---|
| `lean-ctx` (OSS) | 10,048 ✓ | clean | Local engine: compression, tools, triage, value gate |
| `lean-ctx-cloud` | 119 ✓ | clean | SaaS billing: Stripe, plans, metering, Pro features |
| `lean-ctx-enterprise` | 434 ✓ | clean (`--all-features`) | Single-tenant: gateway, routing, governance |

## Implemented OSS modules (`lean-ctx`)

| Area | Implemented surface | Status |
|---|---|---|
| Protocol | `task`, `execution`, `outcome`, `capability`, `decision`, `knowledge`, `triage`, `knowledge_routing` | ✅ |
| Semantic triage | Rules, profiles, fusion, confidence, calibration, validation, distillation scaffolding | ✅ deterministic V0 |
| Task spine | `task_spine`, `decision_loop`, runtime dispatch integration | ✅ |
| Knowledge routing | Resolver, manifests, planner, context bundle, receipt, provider bridge, gate integration | ✅ deterministic V0 |
| Value gate | Cost tracking, outcome evaluation, CPAO, reports, persistent local store | ✅ local V0 |
| Shadow | Baseline, comparison, recommendation, runtime and report persistence | ✅ reference counterfactual |
| Evidence and savings | Integration proof, evidence ledger, session savings tracking | ✅ |
| CLI and dashboard | `prove`, `savings`, `value-report`, `shadow`; local Value Gate API/component | ✅ |

## Implemented Pro features (`lean-ctx-cloud`)

| Feature | API Routes | Status |
|---|---|---|
| Stripe Checkout + Entitlements | `/api/billing/checkout`, webhooks | ✅ |
| Managed OAuth Connectors | `/api/connectors/*` (CRUD, enable/disable) | ✅ |
| Success-Fee Billing | finalize → collect → paid E2E | ✅ |
| Org SSO (OIDC) | `/api/org/sso/*` | ✅ |
| Enterprise License (offline) | `/api/admin/license/*` | ✅ |
| Personal Knowledge Graph | `/api/pro/knowledge/*` (5 endpoints) | ✅ R38 |
| Cross-Device Sync/Backup | `/api/pro/sync/*` (4 endpoints) | ✅ R38 |
| Background Delta Sync Worker | tokio::spawn, 15min interval | ✅ R38 |
| Personal Value Gate Dashboard | `GET /api/pro/dashboard` | ✅ R38 |

## Implemented Enterprise features (`lean-ctx-enterprise`)

| Crate | LOC | Tests | Role |
|---|---|---|---|
| `auto-routing` | 1,364 | 24 | Adaptive model/provider routing with calibration |
| `control-plane-scheduler` | 1,773 | 23 | Candidate generation, budget/quality filters, ranking |
| `intelligence` | 2,370 | — | Decision engine, observation store, circuit breaker |
| `knowledge-hub` | 2,220 | 36 | Authority, provenance, reconciliation, supersession |
| `govern` | 2,033 | 23 | Fleet control, budgets, policy engine, audit |
| `identity` | 1,944 | — | OAuth, OIDC, SCIM, session, team service |
| `optimize` | 1,363 | 21 | Attribution, baseline, fee engine, verification |
| `billing-settlement` | 899 | — | Settlement calculations |
| `wallet-stripe` | 1,148 | — | Enterprise Stripe integration |
| `commercial-core` | ~5,000 | 77 | Shared primitives: plans, license, OIDC, SCIM, policy |
| `entitlements` | 1,564 | — | Plan enforcement |
| `evidence-ledger` | 449 | — | Evidence storage and retrieval |
| `pricing-wallet` | 632 | — | Model pricing tables |
| `verified-savings-contract` | 104 | — | Savings verification contracts |

## Phase status

| Phase | State | Evidence / remaining boundary |
|---|---|---|
| 0 — Claim hygiene | ✅ COMPLETED | Tool count unified; benchmark module + 500-task gold set. |
| 1 — E2E decision loop | ✅ EXIT GATE VERIFIED | Integration test: UUID lineage, cost, CPAO, evidence, audit. |
| 2 — Knowledge routing V0 | ✅ EXIT GATE VERIFIED | Integration test: Jira resolve, plan/bundle, Context Receipt. |
| 3 — Shadow recommendations | ✅ EXIT GATE VERIFIED | Integration test: savings %, quality floor, deterministic report. |
| 4 — Customer POC | ⏳ Business pending | Requires customer, provider keys, baseline/treatment, signed report. |
| 5 — Semantic triage model | 🔨 IN PROGRESS | Scaffolding complete. ONNX model not trained (requires GPU). |
| 6 — Pro subscription | ✅ BACKEND COMPLETE | Cloud: Stripe, Knowledge Graph, Sync, Delta Worker, Dashboard. OSS: triggers + metering. |
| 7 — Enterprise control plane | ✅ CODE COMPLETE | Enterprise repo: auto-routing, scheduler, intelligence, knowledge-hub, govern, identity. 434 tests. |
| 8 — Joint optimization/value share | ✅ CODE COMPLETE | Enterprise: optimize (attribution, baseline, fee_engine, verification) + billing-settlement + wallet-stripe. Cloud: success-fee E2E. |

## Explicitly pending (non-code)

1. Trained, packaged `leanctx-triage-tiny-v1` ONNX model (Phase 5 — needs GPU training).
2. Live provider operation with customer OAuth/BYOK credentials.
3. First paying customer POC with signed value report.
4. Production deployment hardening (HA, backups, monitoring).
5. Enterprise CI green on GitLab (runner toolchain fix needed).

## Sprint history

| Sprint | Status | Note |
|---|---|---|
| 1–6 | ✅ Complete | Contracts, evidence spine, capability/outcome architecture. |
| R34 | ✅ Complete | Control-plane foundation and local decision-loop modules. |
| R35 | ✅ Complete | Phase 0; Phase 1–3 exit gates verified (108 tests). |
| R36 | ✅ Complete | Phase 4-6 foundation: SemanticAnalyzer, benchmark, pro triggers. |
| R37 | ✅ Complete | A/B Measurement, ONNX loader, semantic shadow, accuracy tracker. |
| R38 | ✅ Complete | Pro backend (cloud), Enterprise compile fix + 51 new tests. |
