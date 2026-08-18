# Thinkery
# Executive Summary

## Die Context Intelligence Platform für Enterprise AI Agents

**Version:** v5 · **Stichtag:** 18. August 2026 · **Arbeitswährung:** CHF

**Ein Satz:** Thinkery macht aus einzelnen, teuren und schwer steuerbaren AI
Agents eine zuverlässige, messbare und governable Intelligence Platform.

LeanCTX ist die Open-Source-Engine darunter.

Thinkery ist die kommerzielle Platform darüber.

Context Kits sind die deploybaren Domain-Fähigkeiten dazwischen.

Der Control Plane macht Agent Intelligence operable.

Der Marketplace macht Context zu einem Ecosystem.

Dieses Dokument ist die definitive Zusammenfassung von Vision, Strategie,
Produkt, Architektur, Go-to-Market, Economics, Teamplan und Ask.

Es ist für eine erste Lektüre von ungefähr fünfzehn Minuten geschrieben.

Die vielen kurzen Zeilen halten die Argumentkette scanbar.

Die vertiefenden Details bleiben in den referenzierten Dokumenten.

## Die Investorenfrage, direkt beantwortet

**Ist Thinkery nur ein weiteres AI Tool?**

Nein.

Ein Tool verbessert einen Workflow.

Thinkery standardisiert die Context Layer, auf der viele Workflows laufen.

Ein Tool hat eine Funktion.

Thinkery verbindet Engine, Runtime, Kits, Governance und Distribution.

Ein Tool wird gekauft, wenn ein einzelnes Problem schmerzt.

Thinkery wird standardisiert, wenn eine Organisation eine Agent-Flotte
betreibt.

Ein Tool kann kopiert werden.

Eine Platform mit local-first Trust, Context Graph, Evidence und Kit Ecosystem
compoundiert.

Unser Ziel ist nicht, den besten Agenten zu bauen.

Unser Ziel ist, der Grund zu werden, warum jeder Agent besser, billiger und
kontrollierbarer arbeitet.

## Die Kernthese

Foundation Models werden leistungsfähiger.

Agentic Workflows werden länger.

Tool Calls werden zahlreicher.

Memory wird persistenter.

Budgets werden variabler.

Risiken werden systemischer.

Der Engpass verschiebt sich von Model Capability zu Context Infrastructure.

Der Gewinner besitzt nicht zwingend das größte Model.

Der Gewinner besitzt den richtigen Context zur richtigen Zeit.

Thinkery macht diesen Context:

- relevant;
- komprimiert;
- versioniert;
- provenance-aware;
- policy-aware;
- lokal ausführbar;
- evaluierbar;
- kommerziell messbar.

Das ist die neue Kategorie: **Context Intelligence Platform**.

---

# 1. THE OPPORTUNITY

## 1.1 Der Markt bewegt sich von Chat zu Agent Operations

AI wird vom Antwortsystem zum Ausführungssystem.

Ein Chatbot beantwortet eine Frage.

Ein Agent liest Quellen.

Ein Agent ruft Tools auf.

Ein Agent plant Zwischenschritte.

Ein Agent delegiert an andere Agents.

Ein Agent verändert externe Zustände.

Ein Agent muss seine Arbeit erklären können.

Enterprise adoptiert deshalb nicht nur Models.

Enterprise adoptiert neue digitale Workforce-Strukturen.

Diese Workforce braucht Identity.

Diese Workforce braucht Budget.

Diese Workforce braucht Memory.

Diese Workforce braucht Policies.

Diese Workforce braucht Observability.

Diese Workforce braucht Evaluations.

Diese Workforce braucht einen gemeinsamen Operating Layer.

Der Markt hat viele Agent Frameworks.

Der Markt hat viele Model Providers.

Der Markt hat viele Vector Stores.

Der Markt hat viele Prompt Libraries.

Der Markt hat wenige Systeme, die Context als durchgängige Runtime-Verantwortung
behandeln.

Genau dort liegt Thinkery.

## 1.2 Adoption steigt, Production skaliert nicht automatisch

Unternehmen können Agents schneller prototypen.

Sie können Agents noch nicht mit derselben Geschwindigkeit productionisieren.

Der Grund ist kein einzelner fehlender API Call.

Der Grund ist ein Systemproblem.

Context, identity, integration, evaluation, governance und Kosten hängen
zusammen.

Ein Workflow mit zehn Tool Calls hat mehr Failure Surface als ein einzelner
Prompt.

Ein Multi-Agent Workflow hat mehr Verantwortungsketten als ein einzelner
Process.

Ein persistentes Memory hat mehr Datenschutzfragen als eine Session.

Ein autonomer Agent erzeugt mehr variable Kosten als ein Assistant.

Ein Enterprise-Betrieb braucht daher eine neue Abstraktion.

Die Abstraktion ist nicht noch ein Prompt.

Die Abstraktion ist ein Context Contract.

## 1.3 Das Problem: Context Chaos

Heute startet jeder Agent mit leerem Kurzzeitgedächtnis.

Jede Integration baut Retrieval erneut.

Jedes Team speichert Traces anders.

Jede Organisation misst Tokens, aber nicht Signal pro Token.

Jede Memory-Lösung kennt nur eine Anwendung.

Jede Domain Knowledge Base bleibt passiv.

Jede Compliance-Anforderung kommt zu spät.

Jeder Fehler wird als Prompt-Bug missverstanden.

Wissen wird kopiert.

Wissen wird veraltet.

Wissen wird zu grob eingespeist.

Wissen wird ohne Herkunft präsentiert.

Wissen wird ohne Berechtigungsgrenze weitergereicht.

Wissen wird in jedem Run erneut bezahlt.

Das ist Context Waste.

Das ist Context Drift.

Das ist Context Risk.

Das ist Context Chaos.

## 1.4 Die Marktindikatoren

Die folgende Evidenz stammt aus der Markt- und Production-Research.

Sie trennt beobachtete Fakten von Prognosen und Inferences.

McKinsey berichtet Experimentation, aber begrenztes Scaling und seltenen
messbaren EBIT-Impact.

Gartner prognostiziert, dass über 40% der agentic-AI-Projekte bis Ende 2027
wegen Kosten, Business Value oder Risk Controls beendet werden könnten.

Das ist eine Prognose, keine historische Ausfallquote.

Datadog beobachtet steigende Agent-Framework-Adoption und wachsende Tokenmengen
pro Request.

McKinsey beschreibt agentic tasks, die wesentlich mehr Tokens als einfache
Coding- oder Chat-Aufgaben verbrauchen können.

Gartner verbindet fehlende Semantics mit ungenauen Agents und wasted spending.

Dynatrace berichtet, dass viele agentic decisions weiterhin human-verifiziert
werden.

Anthropic zeigt, warum Tool Calls, State Changes und Error Propagation Agent
Evaluations schwieriger machen als reine Text-Evaluations.

Google Cloud beschreibt Business Context und Semantic Meaning als zentralen
Bottleneck.

Deloitte warnt vor unmonitored agents, die Fehler, conflicting work,
Sensitive-Data-Leaks und Customer Harm verursachen können.

Die Quellen sind in [01-MARKET-PAIN](./01-MARKET-PAIN.md) dokumentiert.

## 1.5 Die relevante Schlussfolgerung

Der Markt braucht keine weitere Oberfläche für Chat.

Der Markt braucht eine Infrastruktur, die Agents production-ready macht.

Diese Infrastruktur muss zwischen Model und Business Workflow liegen.

Sie muss lokale Daten respektieren.

Sie muss Information budgetieren.

Sie muss Memory operationalisieren.

Sie muss Tools begrenzen.

Sie muss Entscheidungen verfolgen.

Sie muss Outcomes messen.

Sie muss Kosten attribuieren.

Sie muss bei Unsicherheit eskalieren.

Thinkery erfüllt diese Anforderungen in einer einzigen Platform-Story.

## 1.6 Warum Context der missing layer ist

Models wissen viel, aber nicht automatisch das Richtige.

Models sehen Daten, aber nicht automatisch die richtige Berechtigung.

Models können handeln, aber nicht automatisch mit einem nachvollziehbaren
Budget.

Models können erinnern, aber nicht automatisch mit Freshness und Provenance.

Models können orchestrieren, aber nicht automatisch mit Ownership.

Context ist die Schicht, die aus Capability eine zuverlässige Leistung macht.

Context beantwortet sechs operative Fragen.

**Was** darf der Agent wissen?

**Warum** ist dieses Wissen relevant?

**Woher** stammt die Information?

**Wie frisch** ist die Information?

**Wie viel** davon passt in das aktuelle Budget?

**Was** darf der Agent als Nächstes tun?

Ein guter Context Contract macht diese Fragen ausführbar.

Thinkery macht den Contract versionierbar.

Thinkery macht den Contract testbar.

Thinkery macht den Contract auditierbar.

Thinkery macht den Contract kommerziell skalierbar.

## 1.7 Warum jetzt

Der Moment ist besonders, weil fünf Kurven gleichzeitig steigen.

Die Model Capability Curve steigt.

Die Agent Adoption Curve steigt.

Die Context Size Curve steigt.

Die Governance Pressure Curve steigt.

Die AI Spend Curve steigt.

Wenn nur eine Kurve stiege, wäre die Chance kleiner.

Zusammen erzeugen sie einen Infrastructure Moment.

Der erste Agent war ein Experiment.

Die nächste Agent-Flotte ist ein Operating Model.

Operating Models brauchen Standards.

Standards brauchen einen neutralen Layer.

LeanCTX kann dieser neutrale Layer werden.

Thinkery kann daraus die Platform machen.

## 1.8 Der Economics-Trigger

Agentic Costs sind nicht linear.

Mehr Steps erzeugen mehr Context.

Mehr Context erzeugt mehr Tokens.

Mehr Tokens erzeugen mehr Provider Cost.

Mehr Tool Calls erzeugen mehr Retries.

Mehr Retries erzeugen mehr Latency.

Mehr Latency erzeugt mehr Human Intervention.

Mehr Intervention senkt den realisierten ROI.

Eine Compression Engine senkt den ersten Kostenhebel.

Eine Runtime senkt den Retry- und Ownership-Hebel.

Ein Control Plane senkt den Governance- und Budget-Hebel.

Context Kits senken den Rebuild-Hebel.

Marketplace Distribution senkt den Acquisition-Hebel.

Thinkery greift die komplette Economics-Kette an.

## 1.9 Der Trust-Trigger

Enterprise kauft keine Autonomy ohne Accountability.

Enterprise verlangt nachvollziehbare Actions.

Enterprise verlangt Datenresidenz.

Enterprise verlangt Deployment-Optionen.

Enterprise verlangt Access Controls.

Enterprise verlangt Evidence.

Enterprise verlangt Rollback.

Enterprise verlangt Support.

Local-first ist deshalb kein Architekturdetail.

Local-first ist ein Sales Advantage.

Local-first reduziert den Data-Exfiltration-Einwand.

Local-first verkürzt Security Review.

Local-first ermöglicht offline und customer-controlled Deployments.

Local-first macht den Trust Anchor open source.

## 1.10 Die Opportunity in einem Satz

Wenn Agents die neue Enterprise Workforce sind, ist Thinkery die Context
Operating System Layer, die diese Workforce produktiv, sicher und wirtschaftlich
steuerbar macht.

## 1.11 Was wir nicht behaupten

Wir behaupten nicht, dass jede Compression 90% spart.

Wir behaupten nicht, dass jedes Model gleich gut mit jeder Compression arbeitet.

Wir behaupten nicht, dass ein Dashboard Governance ersetzt.

Wir behaupten nicht, dass ein Kit Fachverantwortung ersetzt.

Wir behaupten nicht, dass eine Signatur ein gutes Outcome garantiert.

Wir behaupten nicht, dass ein Marketplace automatisch Trust erzeugt.

Wir behaupten, dass die Platform die Messung, Begrenzung und Verbesserung
systematisiert.

Diese Trennung ist Teil unseres Vertrauensmodells.

## 1.12 Der Proof-Standard

Jede Savings-Aussage braucht eine Baseline.

Jede Baseline braucht einen definierten Workflow.

Jeder Workflow braucht ein Input- und Output-Maß.

Jedes Ergebnis braucht einen Quality Check.

Jede Quality Regression braucht eine sichtbare Markierung.

Jede kommerzielle Attribution braucht erhaltene Evidence.

Jede Public Benchmark-Aussage braucht eine Methodik.

Die Headline **60–90%** bleibt eine Hypothesen-Range.

Die Range wird erst als Claim verwendet, wenn Cohort-Daten sie tragen.

Bis dahin lautet die Aussage: Messe deinen eigenen Workflow.

---

# 2. WHAT WE BUILD

## 2.1 Thinkery in einem Satz

Thinkery ist die **Context Intelligence Platform für Enterprise AI Agents**.

Sie reduziert Context Waste.

Sie verteilt Domain Intelligence.

Sie governiert Agent Fleets.

Sie macht Outcomes messbar.

Sie lässt Kunden ihre Daten unter eigener Kontrolle ausführen.

## 2.2 Brand Architecture

**Thinkery** ist die Masterbrand und die kommerzielle Platform.

**LeanCTX** ist die Open-Source-Context-Engine.

**Thinkery Platform** ist die integrierte Enterprise-Oberfläche.

**Thinkery Control** ist die Control-Plane-Produktfläche.

**Thinkery Kits** ist die Portfolio-Bezeichnung.

**Context Kit** ist die Kategorie und das Artefakt.

**Thinkery Exchange** ist der kuratierte Marketplace.

**Thinkery Registry** ist der maschinenlesbare Package-Service.

**ctxpkg** bleibt eine technische Compatibility Namespace.

Die detaillierte Brand-Empfehlung steht in [15-BRANDING](./15-BRANDING.md).

## 2.3 Die fünf Layers

Die User Story folgt fünf Layers.

Die technische Architektur trennt zusätzlich Execution, Control, Distribution und
Enterprise-Zonen.

Für die Produktreife ist die folgende Reihenfolge entscheidend.

### Layer 1 — Context Engine

LeanCTX liest, klassifiziert, routet, dedupliziert und komprimiert Context.

Die Engine läuft local-first.

Sie hält Daten und Evidence am Execution Edge.

Sie ist transparent, composable und Open Source.

Ihr Proof ist messbare Context Efficiency.

### Layer 2 — Agent Runtime

Die Runtime startet, beaufsichtigt und koordiniert Agents.

Sie verwaltet Sessions, Budgets, Leases, Delegation und Recovery.

Sie macht Agent Communication observable.

Sie verbindet MCP, A2A, OCLA, Proxy, Memory und Evidence.

Ihr Proof ist zuverlässige Multi-Agent Execution.

### Layer 3 — Context Kits

Context Kits verpacken Domain Knowledge als versioniertes, signiertes und
evaluierbares Runtime-Artefakt.

Ein Kit enthält Facts, Rules, Procedures, Patterns, Gotchas, Glossary, Tools,
Policies und Evals.

Ein Kit ist kein Prompt Pack.

Ein Kit ist eine installable Domain Capability.

Ihr Proof ist schnellere Onboarding-Zeit bei weniger Review Noise.

### Layer 4 — Control Plane

Der Control Plane verwaltet Fleet Policy, Identity, Budgets, Traces, Evidence,
Entitlements und Outcomes.

Er zeigt eine gemeinsame System View für CTO, Platform, Security, Finance und
Domain Owner.

Er authorisiert gewünschte Zustände, während der Runtime Edge lokal erzwingt.

Sein Proof ist governable Intelligence ohne Rohprompt-Zwang.

### Layer 5 — Marketplace

Der Marketplace macht Context Kits discoverable, vergleichbar, kaufbar und
wiederverwendbar.

Publisher bringen Domain Expertise.

Partners bringen Integrations und Distribution.

Kunden behalten private Overlays und können externe Kits quarantänisieren.

Sein Proof ist Ecosystem Liquidity: mehr Kits, mehr Evals, mehr Adoption.

## 2.4 Das Layer-Prinzip

Jede Layer löst ein eigenständiges Problem.

Die unteren Layers bleiben open und extensible.

Die oberen Layers werden managed, governed und commercial.

Developer können unten einsteigen.

Enterprise kann oben skalieren.

Der Wert wächst mit jeder installierten Layer.

Die Layers sind kein Feature-Bundle.

Sie sind Verantwortungsschnittstellen.

## 2.5 Was LeanCTX bereits als Basis liefert

Die aktuelle Engine ist mehr als ein Tokenizer.

Sie enthält Context Compression.

Sie enthält Intelligent Triage.

Sie enthält Knowledge Routing.

Sie enthält lokale Memory- und Graph-Primitives.

Sie enthält Agent Registry und Agent Bus.

Sie enthält A2A- und OCLA-Bausteine.

Sie enthält LLM Proxy und Provider-Abstraktion.

Sie enthält Savings Ledger und Evidence-Ledger-Bausteine.

Sie enthält PathJail, Shell Allowlist und Redaction.

Sie enthält Dashboard-, CLI- und MCP-Surfaces.

Sie enthält Contract- und Conformance-Tests.

Das ist eine ungewöhnlich breite technische Startposition.

## 2.6 Der zentrale Object Model

Eine Platform braucht gemeinsame Objekte.

Thinkery standardisiert Tenant.

Thinkery standardisiert Workspace.

Thinkery standardisiert Agent Identity.

Thinkery standardisiert Agent Instance.

Thinkery standardisiert Task.

Thinkery standardisiert Context Kit.

Thinkery standardisiert Policy Bundle.

Thinkery standardisiert Execution Receipt.

Thinkery standardisiert Outcome.

Thinkery standardisiert Evidence Bundle.

Diese Objekte sind die Schnittstelle zwischen Engine, Runtime und Control Plane.

## 2.7 Der Context Contract

Ein Context Contract definiert, was ein Agent wissen darf.

Er definiert, warum Wissen relevant ist.

Er definiert, wie Wissen komprimiert wird.

Er definiert, wann Wissen verfällt.

Er definiert, welche Tools erlaubt sind.

Er definiert, wie Quality gemessen wird.

Er definiert, wann Menschen eingreifen.

Er definiert, welche Evidence gespeichert wird.

Thinkery macht diesen Contract ausführbar.

Thinkery macht ihn versionierbar.

Thinkery macht ihn offline-verifizierbar.

## 2.8 Das Open-Core-Prinzip

LeanCTX bleibt als Engine nutzbar.

Lokale Reads bleiben nicht hinter einer Subscription.

Lokale Memory bleibt nicht hinter einer Subscription.

Lokale Compression bleibt nicht hinter einer Subscription.

Kunden können die Engine selbst hosten.

Kunden können Cloud-Connectivity optional machen.

Thinkery monetarisiert Betrieb, Koordination, Governance, Support, Usage und
Distribution.

Open Source verbreitet das Primitive.

Commercial Platform captures organizational value.

Das schützt Adoption und Pricing Power gleichzeitig.

---

# 3. WHY WE WIN

## 3.1 Der Moat ist ein Stack, kein einzelnes Feature

Ein Wettbewerber kann Compression kopieren.

Ein Wettbewerber kann ein Dashboard kopieren.

Ein Wettbewerber kann einen Prompt-Katalog kopieren.

Ein Wettbewerber kann einen Marketplace starten.

Schwer kopierbar ist die Kombination.

Schwer kopierbar ist der Übergang von local edge zu governed fleet.

Schwer kopierbar ist die Evidence-Kette zwischen Context, Action und Outcome.

Schwer kopierbar ist ein Ökosystem, das echte Quality-Signale sammelt.

Schwer kopierbar ist Vertrauen, das durch Open Source beginnt.

Unser Moat entsteht aus fünf Verstärkern.

## 3.2 Verstärker eins: Context Engineering Depth

Die Positionierung umfasst ungefähr 718k LOC Context Engineering.

Das entspricht mehr als zwei Jahren kontinuierlicher technischer Arbeit.

Die Zahl beschreibt den Kontext-Engineering-Footprint, nicht nur eine einzelne
Compression-Funktion.

Die aktuelle Repository-Baseline zählt 707.206 tracked Rust LOC.

Sie zählt 2.318 Rust-Dateien.

Sie zählt 3.558 tracked files insgesamt.

Die Differenz zur 718k-Headline hängt an Scope, Generated Files und Counting
Definition.

Wir frieren vor externer Verwendung ein reproduzierbares LOC-Script ein.

Wir verwenden keine unprüfbare Größenangabe als alleinigen Moat.

Der substanzielle Punkt bleibt unabhängig vom Zähler gültig.

Es existiert eine außergewöhnlich breite Engine-Basis.

Die Basis deckt Compression, Routing, Memory, Graph, Agents, A2A, OCLA,
Evidence, Policy, Proxy und Security ab.

Ein neuer Anbieter müsste nicht nur eine Feature-Demo bauen.

Er müsste die Edge Cases, Contracts, Recovery Paths und Governance Invariants
nachbauen.

Das ist Engineering Leverage.

## 3.3 Verstärker zwei: Compression plus Orchestration plus Governance

Die meisten Anbieter besetzen einen Teil der Kette.

Ein Compression-Anbieter sieht Context Efficiency.

Ein Agent Framework sieht Developer Ergonomics.

Ein Observability-Anbieter sieht Telemetry.

Ein Governance-Anbieter sieht Policy.

Ein Marketplace sieht Discovery.

Thinkery verbindet diese Teile in einem Control Loop.

**Compression** reduziert Waste.

**Orchestration** verteilt Responsibility.

**Governance** begrenzt Risk.

**Evidence** beweist die Ausführung.

**Evaluation** misst den Outcome.

**Kits** übertragen die Verbesserung in neue Domains.

Die Verbindung erzeugt einen höheren Switching Cost als ein Einzelfeature.

## 3.4 Verstärker drei: Open Source als Distribution Engine

Developer können LeanCTX lokal installieren.

Developer benötigen für den ersten Proof keinen Procurement-Prozess.

Developer sehen die Kernlogik.

Developer können Benchmarks reproduzieren.

Developer können Integrations beitragen.

Developer können die Engine selbst hosten.

Open Source senkt die Trust Barrier.

Open Source senkt die Switching Fear.

Open Source erhöht die Install Base.

Open Source erhöht die Anzahl der realen Workflows.

Reale Workflows erzeugen Proof.

Proof erzeugt Team Signals.

Team Signals erzeugen Platform Demand.

Die Commercial Layer muss deshalb nicht jeden Nutzer direkt akquirieren.

Sie monetarisiert die Organisation, die aus Nutzung einen Standard macht.

## 3.5 Verstärker vier: Local-first als Privacy Advantage

Die Execution Zone bleibt beim Kunden.

Rohprompts müssen nicht in den Control Plane.

Local Memory kann ohne Cloud betrieben werden.

Local Evidence kann offline exportiert werden.

Policy Enforcement kann bei Control-Plane-Ausfall lokal weiterlaufen.

Customer-controlled Deployment bleibt möglich.

Region Pinning bleibt möglich.

No-retention- und customer-managed-storage-Modi bleiben möglich.

Das ist keine Anti-Cloud-Position.

Es ist eine Authority-Split-Position.

Der Edge sieht Daten und führt aus.

Der Control Plane authorisiert, korreliert und governiert.

Der Edge sendet nur policy-konforme Observations.

Der Kunde entscheidet, welche Payloads exportiert werden.

Datensouveränität wird so mit Platform Value kompatibel.

## 3.6 Verstärker fünf: Context Kits als recurring Distribution

Kits speichern nicht nur Text.

Kits speichern Domain Rules.

Kits speichern Retrieval Policies.

Kits speichern Tool Constraints.

Kits speichern Evaluation Sets.

Kits speichern Quality Thresholds.

Kits speichern Provenance und Fallback Behaviour.

Ein Kit kann privat bleiben.

Ein Kit kann intern geteilt werden.

Ein Kit kann über Partner ausgeliefert werden.

Ein Kit kann im Exchange verkauft werden.

Jedes Kit erhöht den Nutzen der Runtime.

Jedes Kit erzeugt neue Evals.

Jede Eval verbessert Quality Transparency.

Mehr Quality Transparency verbessert Marketplace Conversion.

Mehr Conversion zieht weitere Authors an.

Das ist ein echter Network Effect.

## 3.7 Der Data- und Evidence-Moat

Thinkery speichert nicht blind alle Transcripts.

Thinkery speichert strukturierte Receipts.

Receipts enthalten Input Digest.

Receipts enthalten Output Digest.

Receipts enthalten Latency.

Receipts enthalten Token Counts.

Receipts enthalten Policy Status.

Receipts enthalten Agent, Task und Tenant Scope.

Evidence Bundles können offline geprüft werden.

Commercial Verification baut auf preserved Evidence.

Das schafft eine Attribution, die nicht nur behauptet wird.

Das schafft eine Savings Share, die messbar definiert werden kann.

Das schafft eine Quality Loop, die nicht auf Vanity Metrics basiert.

## 3.8 Der Context Graph

Der Context Graph verbindet Agents.

Der Context Graph verbindet Tasks.

Der Context Graph verbindet Kits.

Der Context Graph verbindet Quellen.

Der Context Graph verbindet Policies.

Der Context Graph verbindet Tool Calls.

Der Context Graph verbindet Outcomes.

Der Graph macht Ownership sichtbar.

Der Graph macht Provenance sichtbar.

Der Graph macht Wiederholungen sichtbar.

Der Graph macht Context Drift sichtbar.

Der Graph macht Expansion sichtbar.

Der Graph ist kein Lock-in durch geheime Daten.

Der Graph ist ein compoundierendes Arbeitsgedächtnis.

Die Daten bleiben customer-owned.

Die Struktur wird platform-measurable.

## 3.9 Der Product Moat in einer Tabelle

| Fähigkeit | Einzeltool | Thinkery-Verbund |
|---|---|---|
| Context Compression | Kostenoptimierung | Kosten plus Quality plus Evidence |
| Agent Runtime | Prozessstart | Lifecycle, Delegation und Recovery |
| Memory | History Store | Freshness, Provenance und Policy |
| Governance | Admin Dashboard | Signierte Bundles und lokale Enforcement |
| Observability | Request Logs | Context Lineage und Outcome Trace |
| Domain Knowledge | Prompt Pack | Versioniertes, evaluiertes Context Kit |
| Distribution | Download | Registry, Exchange, Partner und Private Overlay |
| Pricing | Seat oder API | Subscription, Usage, Kit und Marketplace |

Die Platform wird mit jeder Spalte stärker.

## 3.10 Warum incumbents nicht automatisch gewinnen

Model Provider besitzen Model Access.

Sie besitzen nicht automatisch Kunden-Context.

Cloud Provider besitzen Compute.

Sie besitzen nicht automatisch domain-spezifische Evals.

Agent Frameworks besitzen Developer Attention.

Sie besitzen nicht automatisch neutralen Cost und Governance Proof.

SI besitzen Kundennähe.

Sie besitzen nicht automatisch eine wiederverwendbare Context Runtime.

Thinkery kann mit allen diesen Kategorien integrieren.

Neutralität ist daher ein Vorteil.

Model Independence ist ein Vorteil.

Customer-controlled Deployment ist ein Vorteil.

## 3.11 Warum wir nicht alles selbst besitzen müssen

Wir bauen nicht jedes Domain Kit intern.

Wir bauen die Runtime, in der Kits sicher laufen.

Wir bauen nicht jede Integration selbst.

Wir bauen Adapter Contracts und Partner Economics.

Wir ersetzen nicht jeden Agent Harness.

Wir bieten einen Context Layer, den Harnesses konsumieren können.

Wir ersetzen nicht das System of Record.

Wir liefern Context, Policy und Evidence rund um dieses System.

Wir bauen keine Model Monoculture.

Wir route n nach Cost, Quality, Policy und Availability.

Diese Grenzen halten den Scope fokussiert.

## 3.12 Der Flywheel

Mehr Agents erzeugen mehr Traces.

Mehr Traces erzeugen bessere Metrics.

Bessere Metrics verbessern Compression Rules.

Bessere Compression Rules senken Costs.

Niedrigere Costs ermöglichen mehr Agents.

Mehr Agents erzeugen mehr Domain Signals.

Domain Signals verbessern Context Kits.

Bessere Kits erhöhen Adoption.

Höhere Adoption bringt mehr Contributors.

Contributors bringen Integrations.

Integrations machen Runtime stickier.

Stickiness macht Control Plane wertvoller.

Control Plane erzeugt bessere Evidence.

Evidence schafft Enterprise Trust.

Enterprise Trust finanziert weitere Distribution.

Das ist ein Platform Flywheel, kein Feature Roadmap Loop.

## 3.13 Die Eintrittsbarriere

Der erste Schritt ist leicht.

Ein Entwickler installiert LeanCTX.

Der erste Value Moment liegt unter fünf Minuten.

Der Expansion Path ist tief.

Ein Team teilt Reports.

Eine Platform standardisiert Policies.

Eine Domain aktiviert Kits.

Ein Enterprise governiert Fleet und Evidence.

Ein Partner publiziert reusable capability.

Ein Marketplace verbindet Nachfrage und Supply.

Leichter Entry plus tiefer Expansion ist die Kernspannung unseres Modells.

## 3.14 Die Positionierung

**Für Developers:** make every AI-agent token earn its place.

**Für Platform Teams:** operate context across the agent fleet.

**Für Security:** enforce policies at the runtime edge.

**Für Finance:** attribute spend to agent, workflow and outcome.

**Für Domain Owners:** package expertise as a governed Context Kit.

**Für Partners:** deliver production agent building blocks repeatedly.

**Für Executives:** turn agent adoption into reliable operating leverage.

## 3.15 Die glaubwürdigste Wette

Wir wetten nicht auf eine einzelne Model-Era.

Wir wetten auf die dauerhafte Notwendigkeit von Context Operations.

Models werden wechseln.

Provider werden wechseln.

Agent Frameworks werden wechseln.

Enterprise Context bleibt.

Policies bleiben.

Memory bleibt.

Evidence bleibt.

Budget bleibt.

Thinkery sitzt auf diesen langlebigen Verantwortungen.

---

# 4. THE PLAN

## 4.1 Roadmap-Logik

Die Roadmap baut von Proof zu Platform.

Jede Version löst ein sichtbares Kundenproblem.

Jede Version erzeugt die Voraussetzung für die nächste.

Jede Version hat einen Revenue- und einen Trust-Gate.

Keine Version wird nur wegen eines Datums veröffentlicht.

Keine Version wird mit unreifen Enterprise-Claims verkauft.

Die Phasen sind:

| Version | Zeitraum | Product Wedge | Business Wedge |
|---|---|---|---|
| v1 | Monat 1–3 | Context Engine | savings proof |
| v2 | Monat 4–6 | Context Kits | domain expertise |
| v3 | Monat 7–9 | Control Plane | fleet governance |
| v4 | Monat 10–14 | Marketplace | ecosystem liquidity |
| v5 | Monat 15–24 | Full Platform | category leadership |

## 4.2 v1 — Context Engine

### Ziel

LeanCTX muss in fünf Minuten installierbar sein.

LeanCTX muss einen realen Agent Workflow messen können.

LeanCTX muss einen Before-and-After-Report ausgeben.

LeanCTX muss Savings und Quality gemeinsam zeigen.

LeanCTX muss local-only funktionieren.

LeanCTX muss Model Provider nicht festlegen.

### Produktumfang

Context Read und Context Search.

Reversible Compression.

Deduplication und Triage.

Knowledge Routing.

Local Memory.

Savings Report.

Consent-aware Telemetry.

MCP, CLI und Hook Integration.

Lokal verifizierbare Evidence.

### Customer Outcome

Ein Developer sieht, wo Context Waste entsteht.

Ein Developer sieht, wie viel Input nach Compression ankommt.

Ein Developer sieht, ob Quality regressiert.

Ein Developer kann den Report teilen.

Ein Team kann einen Workflow als Baseline definieren.

### Go-to-Market

Open Source Launch.

npm und Homebrew Install.

GitHub Benchmark Fixtures.

Founder-led Design Partners.

Hacker News und technische Communities.

Weekly Office Hours.

Public Methodology.

### v1 Gates

Median Time to First Report unter fünf Minuten.

Mindestens 35% Install-to-Activation.

Mindestens 40% Activation-to-Second-Report.

Mindestens 15% Report-Share-Rate.

Quality Regression unter 5% der gemessenen Workflows.

300 Weekly Active Installations am Ende von Monat drei.

30 Team Signals am Ende von Monat drei.

Reproducible Savings über mehrere Fixtures.

### v1 Revenue Target

v1 ist Proof-first, nicht Maximization-first.

Target: 6 bezahlte Pilots im ersten Operating Year.

Target: 12 Team Customers am Year-1-Ende.

Target: CHF 420k Exit ARR am Year-1-Ende.

Target: CHF 620k recognized Year-1 Revenue inklusive Services.

Diese Zahlen stammen aus dem Base Case.

Der Service-Anteil ist absichtlich höher, weil Pilots Trust erzeugen.

### v1 Management Decision

Wenn Savings nicht reproduzierbar sind, wird nicht aggressiv monetarisiert.

Wenn Activation nicht unter fünf Minuten gelingt, wird nicht in Enterprise Sales
skaliert.

Wenn Compression Quality verletzt, wird der Claim reduziert.

Wenn Local-only Trust funktioniert, wird die Distribution beschleunigt.

## 4.3 v2 — Context Kits

### Ziel

Aus Context Efficiency wird Domain Competence.

Ein Expert soll sein Wissen nicht in jeden Prompt kopieren.

Ein Platform Team soll Knowledge einmal authoren und mehrfach verwenden.

Ein Agent soll domain-relevanten Context mit klarer Scope-Grenze erhalten.

### Produktumfang

Kit Manifest.

Typed Facts.

Rules und Procedures.

Patterns und Gotchas.

Glossary und Graph.

Retrieval Policies.

Tool Adapters.

Compression Rules.

Evaluation Fixtures.

Provenance und Quality Breakdown.

Signed, immutable Artifacts.

Private Overlays.

Kit Lock und Rollback.

### Kit Contract

Jedes Kit hat einen global eindeutigen Namen.

Jedes Kit hat eine immutable Version.

Jedes Kit deklariert Schema und Compatibility.

Jedes Kit deklariert Visibility und Data Residency.

Jedes Kit deklariert Activation Triggers.

Jedes Kit deklariert Max Context Budget.

Jedes Kit deklariert Provenance Requirements.

Jedes Kit besteht Structural Validation.

Jede Runtime bewahrt den Kit Hash.

Jede Activation wird geloggt.

Jede Deactivation ist reversibel.

### Erste Kits

Code Review Kit.

SAP Context Kit.

Compliance Kit.

Security Audit Kit.

Customer Support Kit.

Finance Operations Kit.

Die ersten Kits verbinden Engineering Proof mit Enterprise Relevance.

### v2 Gates

28 aktive Pro Kits am Year-1-Ende.

Mindestens 50 Kit Evals.

Mindestens 70% Custom-Kit-to-Pro-Kit Conversion.

Quality Score mit Breakdown statt Vanity Score.

Kit Install in Minuten.

Kit Rollback reproduzierbar.

Private Customer Bytes bleiben im Kundenumfeld.

### v2 Revenue Target

Target: CHF 46k Pro-Kit Revenue in Year 1.

Target: CHF 520k Pro-Kit Revenue in Year 2.

Target: CHF 1.5M Exit ARR als v2 Operating Gate.

Target: vier Kit-Platform-Kunden in Year 2.

Target: CHF 216k Kit-Platform Revenue in Year 2.

Die v2-Targets sind ein Produkt- und Packaging-Test.

Sie sind kein Versprechen, dass jede Domain sofort Marketplace-Liquidität hat.

### v2 Management Decision

Wenn Kits nur Prompts bündeln, ist v2 nicht fertig.

Wenn Evals fehlen, ist v2 nicht distributable.

Wenn Private Overlays nicht funktionieren, ist v2 nicht enterprise-ready.

Wenn Kit Authors keinen klaren ROI sehen, wird das Packaging vereinfacht.

## 4.4 v3 — Control Plane

### Ziel

Ein Team soll seine Agent Fleet nicht aus verstreuten Logs steuern.

Ein CTO soll Context Utilization und Cost sichtbar machen.

Ein Security Team soll Policies authoren und Exceptions verfolgen.

Ein Finance Team soll Usage und Outcomes attribuieren.

Ein Domain Owner soll Kit Performance sehen.

### Produktumfang

Tenant und Workspace.

Agent Identity und Attestation.

Fleet Inventory.

Policy Bundles.

Budget und Rate Limits.

Trace Explorer.

Evidence Bundles.

Outcome Evaluation.

Usage und Savings Ledger.

SSO und SCIM Connectors.

Customer-controlled Deployment.

Signed Assignments.

Fail-closed Runtime Enforcement.

### Authority Split

Der Control Plane authorisiert Desired State.

Der Runtime Edge führt aus.

Der Runtime Edge sieht Rohdaten.

Der Control Plane erhält policy-konforme Observations.

Der Control Plane ist nicht die Execution Source of Truth.

Evidence und Receipts bleiben die Execution Source of Truth.

Eine lokale Schätzung wird nicht automatisch zur Billing Claim.

Eine kommerzielle Verification braucht preserved Evidence.

### v3 Gates

Team Hybrid Mode verfügbar.

Managed Cloud Mode optional.

Customer-controlled Mode contract-kompatibel.

Policy Acceptance und Rejection als Receipts.

Tenant Isolation in jedem Commercial Event.

Signed Artifact Verification.

Budget Exhaustion mit Partial Artifact Preservation.

Multi-Agent Task Lineage.

Human Approval für riskante Side Effects.

### v3 Revenue Target

Target: CHF 3.264M Year-2 Exit ARR.

Target: 60 Business Customers in der Year-3-Planung.

Target: 10 Enterprise Customers in der Year-3-Planung.

Target: CHF 1.06M Platform Subscription Revenue in Year 2.

Target: Enterprise ARPA von ungefähr CHF 26k pro Monat im Base Case.

### v3 Management Decision

Wenn Control Plane Rohdaten erzwingt, verlieren wir Local-first Trust.

Wenn Policies nicht lokal fail-closed enforced werden, verlieren wir Enterprise
Trust.

Wenn Evidence nicht offline prüfbar ist, verlieren wir Auditability.

Wenn Enterprise Support die Product Margin zerstört, wird Deployment enger
standardisiert.

## 4.5 v4 — Marketplace

### Ziel

Context Expertise soll auffindbar, vergleichbar und kaufbar werden.

Partner sollen aus Playbooks reusable Products machen.

Kunden sollen externe Kits sicher testen und private Overlays behalten.

Thinkery soll Distribution nicht nur über Direct Sales skalieren.

### Produktumfang

Thinkery Exchange.

Thinkery Registry.

Publisher Identity.

Signature Verification.

Quality Score Breakdown.

Compatibility Matrix.

Reviews und Evidence.

Pricing und Billing.

Creator Payouts.

Refund und Quarantine Flow.

Partner Attribution.

Private Catalog Mirror.

Staged Rollout und Rollback.

### Marketplace Gates

Mindestens 25 approved Creators.

Mindestens 50 listed Kits.

Mindestens 100 Monthly Buyers.

Repeat Purchase über 20%.

Refund Rate unter 8%.

Annualized GMV über CHF 1M.

Creator Payout Accuracy über 99.5%.

Trust Incident Rate unter vereinbartem Threshold.

### v4 Revenue Target

Target: CHF 150k Marketplace Commission in Year 2.

Target: CHF 800k Marketplace Commission in Year 3.

Target: 400 aktive Marketplace Creators am Year-3-Ende.

Target: CHF 3.2M Marketplace GMV in Year 3.

Default Take Rate: 25% auf Third-party Kit Sales.

Marketplace Take Rate bleibt ein getestetes Pricing Hypothesis.

### v4 Management Decision

Wenn Discovery ohne Quality funktioniert, wird der Katalog kleiner und besser.

Wenn Payout oder License unklar sind, wird nicht breiter gelauncht.

Wenn Marketplace nur Services ersetzt, fehlt echte Liquidity.

Wenn Creator nicht profitabel sind, wird die Supply-Seite nicht compoundieren.

## 4.6 v5 — Full Platform

### Ziel

Thinkery wird die Standard Context Layer für production-grade Agent Fleets.

Die Platform verbindet alle fünf Layers.

Kunden können lokal beginnen und enterpriseweit expandieren.

Partners können wiederholbare Lösungen liefern.

Domain Experts können Wissen als Product distribuieren.

### Produktumfang

Unified Tenant, Workspace und Agent Model.

Unified Context Graph.

Unified Policy and Evidence Model.

Unified Usage and Savings Attribution.

Unified Kit Registry and Exchange.

Unified Support and SLO Layer.

Unified Deployment Topology.

Model-agnostic Routing.

Fleet-level Intelligence Observability.

Outcome-based Expansion.

### v5 Gates

150 Team Customers.

60 Business Customers.

10 Enterprise Customers.

18 Kit-Platform Customers.

55 aktive Partners.

600 aktive Pro Kits.

20.000 aktivierte Starter Workspaces.

Portfolio NRR mindestens 115%.

Enterprise Revenue Concentration unter 35%.

Blended Gross Margin ungefähr 77%.

### v5 Revenue Target

North Star: USD 10M ARR innerhalb von drei Jahren.

Base Case: CHF 12.458M Year-3 Exit ARR.

Base Case: CHF 12.204M recognized Year-3 Revenue.

Base Case: CHF 9.174M recognized recurring Revenue.

Die USD-Zielgröße ist die externe Category-Scale-Zielgröße.

Die CHF-Zahlen sind das interne Operating Model.

FX-Umrechnung wird auf dem jeweiligen Reporting Date fixiert.

### v5 Management Decision

Wenn die Engine-Usage steigt, aber Expansion fehlt, wird Packaging korrigiert.

Wenn Enterprise ARR steigt, aber Concentration über 35% geht, wird Partner
Distribution priorisiert.

Wenn Marketplace GMV steigt, aber Trust sinkt, wird Quality vor Growth gesetzt.

Wenn Services schneller wachsen als Platform Revenue, wird Delivery
standardisiert.

## 4.7 Roadmap als Revenue Bridge

| Phase | Primärer Revenue Proof | Target |
|---|---|---:|
| v1 | Pilots, Team Subscription, early Usage | CHF 620k Y1 Revenue |
| v1 | Exit recurring run-rate | CHF 420k ARR |
| v2 | Pro Kits und Kit Platform | CHF 736k Y2 Revenue |
| v3 | Platform, Usage, Enterprise | CHF 3.0M+ Y2 Revenue |
| v4 | Marketplace und Partner | CHF 1.57M Y3 ecosystem revenue |
| v5 | Full Platform | CHF 12.204M Y3 Revenue |

Die Phasen überlappen in der Recognized-Revenue-Sicht.

Deshalb dürfen die Tabellenwerte nicht additiv als unabhängige Forecasts
interpretiert werden.

Die Entscheidungskriterien sind wichtiger als die exakte Monatszahl.

## 4.8 Capital-efficient Sequencing

v1 nutzt Open Source statt großer Sales Force.

v2 nutzt Domain Experts statt eigener Vertical Staff für jede Branche.

v3 nutzt vorhandene Contracts statt einen komplett neuen Runtime-Kern.

v4 nutzt Partner Distribution statt ausschließlich Direct Sales.

v5 skaliert bestehende Proofs statt neue Category Claims zu erfinden.

Diese Sequenz reduziert Execution Risk.

---
