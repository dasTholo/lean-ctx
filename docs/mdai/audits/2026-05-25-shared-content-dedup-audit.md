---
title: Shared-Content-Dedup-Audit über 4 Kandidaten
date: 2026-05-25
status: ready-for-review
authors: claude
related_specs:
  - docs/mdai/specs/2026-05-26-mdai-v0.1.2-refit-v1.0-audit-design.mdai.md
related_plans:
  - docs/mdai/plans/2026-05-26-mdai-v0.1.2-part-b-findings-v2-and-dedup.md
related_findings:
  - docs/mdai/findings/2026-05-26-mdai-engine-constraints-v2-vs-v1.0.md
related_audits:
  - docs/mdai/audits/2026-05-26-mdai-v1.0-adoption-audit.md
sourced_from:
  - ctx_session [t5-cluster-map] [t5-cluster-N-option-X] (N=1..3, X=A..E)
  - ctx_session [t5-cluster-1-option-A-tested] [t5-cluster-3-option-A-tested]
library_root_variant: B
---

# Shared-Content-Dedup-Audit

## Übersichtstabelle

| Cluster | Topic              | Vorkommen (file:line)                                                                                                                            | Empfehlung des Audits          |
| ---     | ---                | ---                                                                                                                                              | ---                            |
| 1       | 6-Anchor-Liste     | canonical: `mdai/core/lean-context-audit.md:19-26`; derivative: `mdai/skills/mdai-brainstorm/spec-self-review.md:38-43`                          | A (chosen by user 2026-05-26) |
| 2       | mode="full"-Rule   | canonical: `mdai/core/lean-context.md:14-16`; derivatives: `mdai/core/lean-context-audit.md:21`, `mdai/skills/mdai-brainstorm/spec-self-review.md:38` | E (chosen by user 2026-05-26) |
| 3       | Anti-Pattern-Liste | canonical: `mdai/core/lean-context.md:14-20` (Tabelle); derivatives: `mdai/core/lean-context-audit.md:22-26`, `mdai/skills/mdai-brainstorm/spec-self-review.md:39-43` | A (chosen by user 2026-05-26) |

`mdai/core/library-spec-audit.md` ist **kein** Cluster-Mitglied — separate
7-Check-Pack-Mechanics-Audit-Datei ohne Inhalt-Overlap mit den drei oben.

## Neue Engine-Finding — `@include` von Files ohne `@markdownai`-Header

**Constraint:** Wenn ein per `@include` referenziertes File NICHT mit
`@markdownai` startet (oder das im Frontmatter deklariert), liefert
`executeInclude` (Source: `markdownai/packages/engine/src/engine-include.ts:133`,
`if (!ast.isMarkdownAI) return ''`) stillschweigend einen leeren String — ohne
Warning, ohne Error.

**Geltung:** Universell. Gilt überall wo `@include` verwendet wird — NICHT
spezifisch auf `@include` innerhalb `@define`-Bodies beschränkt. `@include`
selbst überlebt die Substitution in `@define`-Bodies (`macros.ts
substituteNode case 'include'` behält den Node-Typ) und wird zu `call_macro`-Zeit
ausgeführt (`engine.ts:378` → `walkNodes` → `case 'include': executeInclude`).
Die Ursprungs-Hypothese ("`@include` verschwindet stillschweigend in
`@define`-Body bei `call_macro`") war daher mechanisch falsch — der
tatsächliche Trigger ist der fehlende `@markdownai`-Header im
Include-Target-File.

**Symptom in T5a Option-A-Smoke:** Das Prototype-Fragment
`mdai/core/lean-context-anchors.md` wurde per Plan-Instruktion als
"pure body, kein Frontmatter, kein `@markdownai`-Header" geschrieben. Folge:
der `@include`-Block im `lean_context_audit`-Macro-Body lieferte einen leeren
Abschnitt. Beobachtbar: `warnings: ["Unresolvable expression: spec_path"]` —
nur der erwartete Parameter-Binding-Warning, KEIN Include-Warning.

**Implikation für Option A:** Option A ist nicht intrinsisch broken — sie ist
viable IF the fragment file carries `@markdownai v1.0` header (oder
Frontmatter-Deklaration). Engine-OK-Score für Option A wird daher in der
Optionen-Tabelle pro Cluster von `1 (FAIL)` auf `2` angehoben (works with
header-discipline footgun — undocumented und nicht-auffindbar ohne Source-Read).

**Empfehlung:** Aufnehmen in
`docs/mdai/findings/2026-05-26-mdai-engine-constraints-v2-vs-v1.0.md` als §13
(T5c oder v0.1.3-Task), mit Title z.B. "`@include` von Files ohne
`@markdownai`-Header → silent empty return".

## Cluster 1 — 6-Anchor-Liste

### Vorkommen

- (canonical) `mdai/core/lean-context-audit.md:19-26` — `## 6 Anchors`-Block
  innerhalb `@define lean_context_audit(spec_path) … @end`.
- (derivative) `mdai/skills/mdai-brainstorm/spec-self-review.md:38-43` —
  Check #6-Block innerhalb `@define spec_self_review(spec_path) … @end`,
  Wording leicht abweichend (z.B. `(§0)`-Pointer in Z 1; `… needs a @note
  visible consumer="human" block` statt `… needs @note visible consumer="human"`).

### Optionen-Tabelle

| Option                         | Engine-OK | Robustheit | Self-Containment | MCP-Roundtrips | Total | Empfehlung |
| ---                            | ---       | ---        | ---              | ---            | ---   | ---        |
| A) Fragment-File + @include    | 2         | 3          | 2                | 3              | 10    | tie        |
| B) @include lines=N-M          | 3         | 1          | 2                | 3              | 9     | no         |
| C) @define + @call             | 2         | 2          | 2                | 2              | 8     | no         |
| D) call_macro Delegate         | 3         | 3          | 1                | 1              | 8     | no         |
| E) Status Quo + Drift-Tracking | 3         | 1          | 3                | 3              | 10    | tie        |

### Empfehlung des Audits + Begründung

Option A und Option E sind nach Re-Scoring (siehe Engine-Finding oben) gleich
auf (Total = 10). Option A ist viable, sofern das Fragment-File
`@markdownai v1.0` als Header trägt — sonst silent-empty-Return. Option E
(Status Quo) vermeidet diesen Footgun komplett auf Kosten einer
Drift-Surface (1 Derivative-Konsument). Drift-Risiko wird durch
`<!-- drift-source: mdai/core/lean-context-audit.md:19-26 -->`-Kommentar im
Derivative-Konsumenten transparent gemacht. **Top-Option wird im User-Gate
finalisiert; Option A erfordert `@markdownai`-Header in Fragment-File.**

**User-Decision 2026-05-26:** Option A — gewählt für durchgängiges Fragment-File-Pattern (konsistent mit Cluster 3). Header-Disziplin-Footgun akzeptiert.

### Action für v0.1.3-Plan

- **Wenn E gewählt:** Keine Strukturänderung an `lean-context-audit.md`.
  Drift-Tracking-Kommentar oberhalb des 6-Anchor-Blocks in
  `mdai/skills/mdai-brainstorm/spec-self-review.md` einsetzen:
  ```
  <!-- drift-source: ${MDAI_LIBRARY_ROOT}/core/lean-context-audit.md:19-26
       Re-sync this list if the canonical source changes.
       Engine-Constraint: @include of files without @markdownai header returns
       silent-empty (engine-include.ts:133). Option A viable only with header. -->
  ```
- **Wenn A gewählt:** Fragment-File `mdai/core/lean-context-anchors.md` mit
  `@markdownai v1.0`-Header (oder Frontmatter-Deklaration) anlegen; im
  Derivative-Konsumenten den 6-Anchor-Block durch
  `` `@include ${MDAI_LIBRARY_ROOT}/core/lean-context-anchors.md` `` ersetzen.
- Wording-Drift (Z 1 `(§0)`-Pointer + Z 2 `… block`) bewusst akzeptieren oder
  in v0.1.3-Cleanup an canonical angleichen.

## Cluster 2 — mode="full"-Rule

### Vorkommen

- (canonical) `mdai/core/lean-context.md:14-16` — Rules-Table-Rows für
  `ctx_read (cross-file scan)`, `ctx_read (after ctx_search / find_symbol)`,
  `ctx_read (spec-review target)` mit `mode="full"`-Exception.
- (derivative) `mdai/core/lean-context-audit.md:21` — Anchor #1 (eine Zeile:
  `mode="full" — only allowed for the one spec-source read; flag all others.`),
  innerhalb `@define lean_context_audit`.
- (derivative) `mdai/skills/mdai-brainstorm/spec-self-review.md:38` — Z 1 von
  Check #6, fast identisch zur lean-context-audit-Variante (zusätzlich `(§0)`),
  innerhalb `@define spec_self_review`.

### Optionen-Tabelle

| Option                         | Engine-OK | Robustheit | Self-Containment | MCP-Roundtrips | Total | Empfehlung |
| ---                            | ---       | ---        | ---              | ---            | ---   | ---        |
| A) Fragment-File + @include    | 2         | 3          | 2                | 3              | 10    | no         |
| B) @include lines=N-M          | 3         | 1          | 2                | 3              | 9     | no         |
| C) @define + @call             | 2         | 2          | 2                | 2              | 8     | no         |
| D) call_macro Delegate         | 3         | 3          | 1                | 1              | 8     | no         |
| E) Status Quo + Drift-Tracking | 3         | 2          | 3                | 3              | 11    | yes        |

### Empfehlung des Audits + Begründung

Option E klarer Sieger (Total = 11 vs. Option A = 10 nach Re-Scoring). Eine
einzeilige Rule rechtfertigt keine zusätzliche Datei oder Macro-Indirektion,
selbst wenn Option A nun engine-viable wäre (mit `@markdownai`-Header-Disziplin).
Drift-Surface ist eine Zeile in zwei Konsumenten — Drift-Tracking-Kommentar
genügt.

**User-Decision 2026-05-26:** Option E — Default akzeptiert (klarer Score-Vorsprung 11 vs 10).

### Action für v0.1.3-Plan

- **Keine Strukturänderung.** In beiden Derivative-Files (Z 21 von
  `lean-context-audit.md` bzw. Z 38 von `spec-self-review.md`) je einen
  Drift-Tracking-Kommentar einfügen:
  ```
  <!-- drift-source: ${MDAI_LIBRARY_ROOT}/core/lean-context.md:14-16 (rules-table mode="full" rows) -->
  ```

## Cluster 3 — Anti-Pattern-Liste

### Vorkommen

- (canonical) `mdai/core/lean-context.md:14-20` — Rules-Tabelle mit Spalten
  Tool/Default/Exception, abdeckt `ctx_shell raw=true`, `find_symbol body=true`,
  `ctx_read fresh=true` etc.
- (derivative) `mdai/core/lean-context-audit.md:22-26` — 5 Anchors als
  Checkliste, innerhalb `@define lean_context_audit`.
- (derivative) `mdai/skills/mdai-brainstorm/spec-self-review.md:39-43` — Z 2-6
  von Check #6, identische 5 Anti-Pattern als Checkliste, innerhalb
  `@define spec_self_review`.

### Optionen-Tabelle

| Option                         | Engine-OK | Robustheit | Self-Containment | MCP-Roundtrips | Total | Empfehlung |
| ---                            | ---       | ---        | ---              | ---            | ---   | ---        |
| A) Fragment-File + @include    | 2         | 3          | 2                | 3              | 10    | tie        |
| B) @include lines=N-M          | 3         | 1          | 2                | 3              | 9     | no         |
| C) @define + @call             | 2         | 2          | 2                | 2              | 8     | no         |
| D) call_macro Delegate         | 3         | 3          | 1                | 1              | 8     | no         |
| E) Status Quo + Drift-Tracking | 3         | 1          | 3                | 3              | 10    | tie        |

### Empfehlung des Audits + Begründung

Option A und Option E sind nach Re-Scoring (siehe Engine-Finding oben) gleich
auf (Total = 10). Option A ist viable, sofern das Fragment-File
`@markdownai v1.0` als Header trägt; das eliminiert die Drift-Surface (3 Files
× 5 Zeilen) komplett, exponiert aber den `@markdownai`-Header-Footgun. Option
E bleibt drift-tolerant per Kommentar, ohne Footgun-Exposure. **Top-Option
wird im User-Gate finalisiert; Option A erfordert `@markdownai`-Header in
Fragment-File.**

**User-Decision 2026-05-26:** Option A — gewählt für größte De-Dup-Wirkung (3 derivatives) und Pattern-Konsistenz mit Cluster 1.

### Action für v0.1.3-Plan

- **Wenn E gewählt:** Keine Strukturänderung. Drift-Tracking-Kommentar
  oberhalb beider Derivative-Blöcke einsetzen:
  ```
  <!-- drift-source: ${MDAI_LIBRARY_ROOT}/core/lean-context.md:14-20 (rules-table anti-patterns)
       Re-sync this list if the canonical source changes.
       Engine-Constraint: @include of files without @markdownai header returns
       silent-empty (engine-include.ts:133). Option A viable only with header. -->
  ```
- **Wenn A gewählt:** Fragment-File
  `mdai/core/lean-context-anti-patterns.md` mit `@markdownai v1.0`-Header
  anlegen; in beiden Derivative-Konsumenten den Anti-Pattern-Block durch
  `` `@include ${MDAI_LIBRARY_ROOT}/core/lean-context-anti-patterns.md` ``
  ersetzen.
- Wording-Konsistenz-Check: derivative formuliert in Imperativ
  ("replace with `@call ctx_search(...)`"), canonical in Tabellen-Spalte
  ("Default" / "Exception"). Falls bei v0.1.3-Adoption beide an einer
  einheitlichen Wording ausgerichtet werden sollen, im selben Task tun.

---

## Adoption-Hinweise für v0.1.3

- **Cluster 1 (6-Anchor-Liste):** Option A — v0.1.3-Task: schreibe
  `mdai/core/lean-context-anchors.md` mit `@markdownai v1.0`-Header + 6-Anchor-Liste
  verbatim aus `mdai/core/lean-context-audit.md:19-26`. Ersetze inline-Listen in
  `mdai/core/lean-context-audit.md` und `mdai/skills/mdai-brainstorm/spec-self-review.md`
  durch `@include ${MDAI_LIBRARY_ROOT}/core/lean-context-anchors.md`.
- **Cluster 2 (`mode="full"`-Rule):** Option E — keine v0.1.3-Migration. HTML-
  Kommentar als Drift-Tracking-Marker in `mdai/core/lean-context-audit.md` und
  `mdai/skills/mdai-brainstorm/spec-self-review.md` einfügen, der auf
  `mdai/core/lean-context.md:14-16` als canonical zeigt.
- **Cluster 3 (Anti-Pattern-Liste):** Option A — v0.1.3-Task: schreibe
  `mdai/core/anti-patterns.md` (oder analog) mit `@markdownai v1.0`-Header +
  Anti-Pattern-Tabelle verbatim aus `mdai/core/lean-context.md:14-20`. Ersetze
  Vorkommen in `mdai/core/lean-context-audit.md:22-26` und
  `mdai/skills/mdai-brainstorm/spec-self-review.md:39-43` durch
  `@include ${MDAI_LIBRARY_ROOT}/core/anti-patterns.md`.
- Konsumenten-Migration ist immer Cross-File — pro Cluster: canonical-File + alle
  derivatives in EINEM v0.1.3-Task abarbeiten, nicht aufteilen.
- Variante-B-Prefix: `${MDAI_LIBRARY_ROOT}` — in Spawn-Env der Konsumenten setzen.
- **Header-Disziplin:** Fragment-Files (`lean-context-anchors.md`,
  `anti-patterns.md`) MÜSSEN mit `@markdownai v1.0` als erster Zeile beginnen,
  sonst silent empty return per neuer Findings-v2 §13 (siehe Section "Neue
  Engine-Finding" oben).
