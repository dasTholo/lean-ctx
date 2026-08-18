# Thinkery Context Kits

Product concept and implementation specification

## 0. Executive definition

1. A Context Kit is a versioned, signed, executable knowledge product for an AI agent.
2. A kit combines facts, vocabulary, procedures, constraints, examples, and retrieval indexes.
3. A kit also declares when its knowledge is relevant and how much context it may consume.
4. A kit is closer to an image than to a folder of documents.
5. An image has a manifest, layers, dependencies, reproducible bytes, and a runtime contract.
6. A kit has the same properties for agent intelligence.
7. Thinkery owns the kit lifecycle; LeanCTX supplies the context operating system.
8. The product promise is domain competence without rebuilding a prompt for every task.
9. The engineering promise is deterministic loading, bounded context cost, and auditable provenance.
10. The safety promise is explicit scope, conflict handling, redaction, and fail-closed validation.

## 1. Product thesis

1. Agents fail when relevant expertise is absent, stale, or presented at the wrong granularity.
2. Plain retrieval finds text but rarely supplies procedures, constraints, or decision boundaries.
3. Context Kits package expertise into runtime-addressable layers.
4. LeanCTX compresses those layers before they reach a model.
5. The kit remains useful when the model context window is small.
6. The kit remains useful when a task touches many files or systems.
7. The kit becomes better through evaluations and approved operational feedback.
8. A kit is not a persona, a system prompt, or a hidden chain of thought.
9. A kit contains inspectable artifacts that a customer can test and govern.
10. A kit must degrade safely when its data is missing or incompatible.

## 2. Product goals

1. Let an agent acquire SAP expertise with one activation.
2. Let a security team publish internal controls once and reuse them everywhere.
3. Let a company share coding conventions without copying private documents into prompts.
4. Let an engineer inspect exactly why a kit influenced an answer.
5. Let a platform owner pin versions and reproduce an agent run.
6. Let a kit author measure task success before distribution.
7. Let a customer keep private kit bytes inside its own environment.
8. Let marketplace kits coexist with private company overlays.
9. Let LeanCTX spend context tokens according to task value.
10. Let a kit declare limits instead of assuming unlimited retrieval.

## 3. Non-goals

1. A kit is not a general-purpose model fine-tune.
2. A kit is not a replacement for a source-of-truth transactional system.
3. A kit is not permission to bypass customer access controls.
4. A kit is not a guarantee that an agent gives professional advice.
5. A kit is not an unreviewed dump of scraped web pages.
6. A kit is not a mechanism for hiding proprietary instructions from administrators.
7. A kit is not allowed to execute arbitrary native code by default.
8. A kit does not override user, workspace, or safety policy.
9. A kit does not silently send private data to a marketplace.
10. A kit does not replace human approval for regulated decisions.

## 4. Core vocabulary

1. Kit: the product-level bundle installed and activated by LeanCTX.
2. Kit source: the authoring directory before deterministic packaging.
3. Kit artifact: the immutable distributable .lctxkit file or registry object.
4. Manifest: machine-readable identity, compatibility, scope, integrity, and policy metadata.
5. Layer: one typed collection of knowledge consumed by the runtime.
6. Rule: a compact instruction with trigger, action, evidence, and exception fields.
7. Fact: a normalized claim with provenance, confidence, validity, and scope.
8. Pattern: a reusable solution shape with examples and failure modes.
9. Gotcha: a known trap with trigger signals and a safe resolution.
10. Procedure: an ordered workflow with prerequisites, checks, and stop conditions.
11. Graph: typed entities and relations used for expansion and conflict detection.
12. Corpus: source documents retained for citation, search, or evidence inspection.
13. Index: derived retrieval state; it is rebuildable and never the source of truth.
14. Trigger: a declarative signal that makes a kit eligible for activation.
15. Activation: the runtime decision to make kit layers available for a task.
16. Injection: the selected, compressed kit material placed into model-visible context.
17. Overlay: a higher-priority layer that refines or restricts a base kit.
18. Adapter: a connector that maps kit concepts to a local tool or data source.
19. Evaluation: a test case and expected behavior used to validate a kit.
20. Provenance: evidence describing where a claim came from and when it was verified.
21. Trust level: a policy label controlling how strongly kit material may influence output.
22. Kit lock: a reproducibility file pinning kit versions, hashes, and source registry.
23. Registry: a service or directory that stores, resolves, signs, and distributes kits.
24. Kit family: all versions sharing the same stable name and identity namespace.
25. Capability: a declared runtime feature required by a kit.

## 5. Kit contract

1. Every kit MUST have a globally unique name within its registry namespace.
2. Every kit MUST have an immutable version.
3. Every kit MUST declare a schema version.
4. Every kit MUST declare supported LeanCTX versions or a compatibility range.
5. Every kit MUST declare whether it is industry, function, company, or addon content.
6. Every kit MUST declare data residency and visibility.
7. Every kit MUST declare its activation triggers.
8. Every kit MUST declare its maximum context budget.
9. Every kit MUST declare provenance requirements for high-risk claims.
10. Every kit MUST pass structural validation before installation.
11. Every kit SHOULD include task evaluations.
12. Every kit SHOULD include a human-readable README.
13. Every kit MAY include adapters, but adapters require explicit capability approval.
14. A runtime MUST reject unknown mandatory capabilities.
15. A runtime MAY ignore optional layers it cannot support.
16. A runtime MUST preserve the kit hash in run provenance.
17. A runtime MUST record activation and injection decisions.
18. A runtime MUST make deactivation reversible without deleting the artifact.
19. A kit MUST never mutate its own immutable payload.
20. A kit update MUST create a new artifact version.

## 6. Reference architecture

1. The authoring layer turns source material into typed kit layers.
2. The build layer normalizes, redacts, validates, chunks, and indexes content.
3. The signing layer canonicalizes bytes and signs the manifest plus content hash.
4. The registry stores immutable artifacts and signed metadata.
5. The resolver chooses compatible versions and dependencies.
6. The runtime mounts selected kits into a session-scoped context namespace.
7. The activation engine scores kit relevance against task and workspace signals.
8. The compression engine selects a budgeted representation of active layers.
9. The injection planner orders kit material with user and workspace context.
10. The model client receives only the planned, compressed representation.
11. The evidence ledger records claims, source handles, and injection reasons.
12. The feedback loop captures outcomes without silently changing a published kit.

## 7. Logical runtime flow

1. task.open creates or resumes a LeanCTX session.
2. Workspace detectors emit language, framework, path, tool, and repository signals.
3. The kit resolver reads explicit kit requests from the client or agent.
4. The activation engine scores installed kits against the current task.
5. Policy filters remove kits forbidden by tenant, project, or data boundary.
6. Dependency resolution adds required base kits and compatible overlays.
7. The runtime creates an activation set with stable ordering.
8. Each layer exposes candidates through the shared retrieval interfaces.
9. LeanCTX computes marginal information gain under the token budget.
10. The compression engine emits a kit context capsule.
11. The injection planner places capsule sections into the appropriate context slots.
12. The model turn includes provenance handles, not raw registry internals.
13. Tool results update task signals and may trigger reevaluation.
14. The runtime logs why a kit was activated, skipped, or evicted.
15. Session completion records outcome metrics for later evaluation.

## 8. Source directory layout

1. kit.toml is the human-authored manifest.
2. README.md explains purpose, scope, installation, and limitations.
3. LICENSE records distribution terms.
4. NOTICE records third-party attribution.
5. content/ contains normalized source and authored knowledge.
6. content/facts/ contains JSONL or YAML facts.
7. content/rules/ contains compact activation and behavior rules.
8. content/patterns/ contains problem, solution, example, and failure shapes.
9. content/procedures/ contains ordered operational workflows.
10. content/gotchas/ contains known traps and stop conditions.
11. content/glossary/ contains aliases, acronyms, and canonical terms.
12. content/graph/ contains typed nodes and edges.
13. corpus/ contains source documents retained for evidence.
14. adapters/ contains declarative connector specifications.
15. prompts/ contains bounded templates, never hidden model reasoning.
16. compression/ contains rules for salience and representation selection.
17. evals/ contains task fixtures, rubrics, and expected evidence.
18. schemas/ contains optional domain validation schemas.
19. generated/ contains build outputs and MUST be ignored by authors.
20. kit.lock pins build inputs when reproducibility is required.

## 9. Canonical artifact layout

1. The distributable extension is .lctxkit.
2. The artifact is a deterministic ZIP container with normalized path separators.
3. The first entry is manifest.json.
4. The second entry is content.json.
5. Optional entries are compressed corpus blobs and index shards.
6. ZIP timestamps MUST be zeroed or fixed to the source epoch.
7. ZIP entry ordering MUST be lexical after the two required entries.
8. File permissions MUST be normalized to read-only regular files.
9. Symlinks MUST be rejected during packaging.
10. Absolute paths MUST be rejected during packaging.
11. Path traversal components MUST be rejected during packaging.
12. The package hash covers canonical manifest and canonical content.
13. The package signature covers the package hash and signer identity.
14. Derived indexes MAY be omitted and rebuilt after installation.
15. A registry MAY store the same canonical payload outside ZIP for streaming.
16. Runtime import MUST support the existing JSON package path for migration.
17. A kit artifact MAY contain an existing context_package payload.
18. A context package MUST declare kind = context when used as a base layer.
19. A document-only bundle MUST declare kind = skills.
20. An executable addon MUST remain separate from ordinary kit content.

## 10. Authoring manifest: kit.toml

1. TOML is chosen for author ergonomics and stable human review.
2. JSON is chosen for canonical runtime serialization and signature verification.
3. YAML is accepted only as an ingestion format, never as canonical bytes.
4. The build converts TOML to a normalized JSON manifest.
5. Unknown fields are warnings in development and errors in release mode.
6. Duplicate keys are always errors.
7. All identifiers use Unicode NFC normalization before validation.
8. Names are lowercase DNS-like slugs with optional namespace.
9. Versions use SemVer with no mutable aliases in the artifact.
10. Dates use RFC 3339 and are normalized to UTC.
11. Local paths are resolved relative to the kit root.
12. A manifest cannot reference files outside the kit root.
13. A kit can declare multiple target domains and one primary domain.
14. A kit can declare several target languages or frameworks.
15. A kit can declare a minimum evidence age for volatile claims.
16. A kit can declare whether network refresh is required at runtime.
17. Network refresh defaults to disabled in offline and enterprise mode.
18. A kit can declare a human owner and an operational owner.
19. A kit can declare a support channel without exposing private content.
20. The full manifest is inspectable before activation.

## 11. Minimal manifest example

~~~toml
schema_version = 1
kind = "kit"
name = "thinkery/security-audit"
version = "1.0.0"
description = "Evidence-aware application security review expertise."
category = "function"
visibility = "public"
license = "Apache-2.0"
min_lean_ctx = "3.9.18"
max_context_tokens = 3200
default_activation = "eligible"

[owner]
publisher = "Thinkery"
maintainer = "security@thinkery.example"

[scope]
domains = ["application-security"]
languages = ["rust", "typescript", "python", "go", "java"]
frameworks = ["web", "api", "graphql", "cloud"]
file_globs = ["**/*"]

[activation]
keywords = ["security", "audit", "vulnerability", "threat model"]
tools = ["git", "ctx_search", "ctx_review"]
task_kinds = ["review", "audit", "incident-response"]
minimum_score = 0.58

[policy]
trust = "reviewed"
allow_network = false
require_citations = true
allow_tool_adapters = false
conflict_mode = "surface"

[layers]
rules = "content/rules"
facts = "content/facts"
patterns = "content/patterns"
procedures = "content/procedures"
gotchas = "content/gotchas"
glossary = "content/glossary"
graph = "content/graph"
corpus = "corpus"
compression = "compression/rules.toml"

[dependencies]
base = ["thinkery/software-engineering-core@^1.0.0"]

[eval]
suite = "evals/suite.yaml"
minimum_pass_rate = 0.90
maximum_unsupported_claim_rate = 0.02
~~~

## 12. Runtime manifest mapping

1. schema_version maps to the existing package manifest schema version.
2. kind = kit maps to a composed context package family.
3. name maps to the existing package name field.
4. version maps to the existing immutable package version field.
5. description maps to package description metadata.
6. min_lean_ctx maps to compatibility.min_lean_ctx_version.
7. languages maps to compatibility.target_languages.
8. frameworks maps to compatibility.target_frameworks.
9. Layer names map to PackageLayer values or kit extension layers.
10. content_hash maps to the existing integrity content hash.
11. package_hash maps to the existing SHA-256 package integrity field.
12. provenance includes builder version and source session where applicable.
13. signature uses the existing Ed25519 verification path.
14. documents may use the existing zstd plus base64 document blob encoding.
15. context_graph may use the existing graph composition model.
16. Kit-only fields live under a versioned kit extension object.
17. Older runtimes ignore optional kit fields only when safe.
18. Older runtimes MUST reject a kit requiring unsupported behavior.
19. The importer emits a migration report for every legacy package.
20. The lock file records both logical and physical package hashes.

## 13. Layer model

1. Layers are independently addressable so the runtime can budget them separately.
2. Each layer has a stable name, schema, priority, and evidence policy.
3. Each layer exposes a compact summary and a retrievable body.
4. Each layer records source handles for every material claim.
5. Each layer can be disabled without uninstalling the kit.
6. Rules are small and high-signal.
7. Facts are normalized and deduplicated.
8. Patterns preserve reusable reasoning structure without private chain of thought.
9. Procedures preserve order, branching, and stop conditions.
10. Gotchas protect against recurrent and costly mistakes.
11. Glossaries improve query expansion and terminology alignment.
12. Graphs express relationships that text retrieval misses.
13. Corpus documents provide citations and detailed evidence.
14. Compression rules tell LeanCTX how to represent the layer.
15. Adapters expose narrowly scoped local capabilities.
16. Evaluations define expected behavior for release gates.
17. A layer can be authored manually or generated from a source.
18. Generated claims require a source and a transformation record.
19. A layer is invalid when it contains unsupported executable directives.
20. A layer is stale when its freshness policy has expired.

## 14. Common record envelope

1. Every typed record has an id stable within the kit family.
2. Every record has a kind that selects its schema.
3. Every record has a title intended for compact display.
4. Every record has a body intended for retrieval or evidence.
5. Every record has tags for filtering and activation.
6. Every record has a scope describing where it applies.
7. Every record has confidence in the range 0.0 through 1.0.
8. Every record has trust derived from provenance and review status.
9. Every record has source references.
10. Every record has created and updated timestamps.
11. Every record MAY have a valid_from timestamp.
12. Every record MAY have an expires_at timestamp.
13. Every record MAY declare supersedes identifiers.
14. Every record MAY declare contradicts identifiers.
15. Every record MUST be safe to omit from a compressed answer.
16. The runtime does not treat confidence as factual truth.
17. A reviewer can override confidence only with an audit entry.
18. Record IDs are content-addressed when generated from source.
19. Author-assigned IDs remain stable across wording edits.
20. Deleted records remain visible in release diffs and audit history.

## 15. Fact schema

1. A fact is one atomic claim that can be cited.
2. A fact SHOULD contain one subject and one predicate.
3. A fact MAY contain an object, value, or condition.
4. A fact MUST identify a source or explicitly declare authored knowledge.
5. A fact MUST distinguish normative from descriptive language.
6. A fact MUST declare whether it is stable or time-sensitive.
7. A fact MUST include a conflict key for duplicate detection.
8. A fact SHOULD include a verification method.
9. A fact SHOULD include a reviewer status.
10. A fact MUST not encode credentials or raw secrets.
11. A fact may contain a redacted placeholder with a secret class.
12. A fact can be scoped to a product release.
13. A fact can be scoped to a jurisdiction.
14. A fact can be scoped to a company repository.
15. A fact can be scoped to a user role.
16. Facts with unknown scope do not receive high-trust status.
17. Facts from a public kit cannot point to private file paths.
18. Facts from a company kit may point to private handles.
19. The loader deduplicates facts by conflict key and source precedence.
20. The loader preserves competing facts when policy says surface conflict.

~~~json
{
  "id": "fact:fhir:resource-patient",
  "kind": "fact",
  "title": "FHIR Patient is a resource type",
  "subject": "FHIR.Patient",
  "predicate": "is_resource_type",
  "object": "true",
  "claim": "Patient is a FHIR resource type with identity and lifecycle metadata.",
  "normative": false,
  "scope": {
    "domains": ["healthcare"],
    "standards": ["FHIR R4"]
  },
  "confidence": 0.99,
  "trust": "normative-source",
  "sources": [
    {
      "uri": "https://hl7.org/fhir/R4/patient.html",
      "locator": "Patient",
      "retrieved_at": "2026-01-10T00:00:00Z"
    }
  ],
  "freshness": "stable",
  "review": "approved",
  "conflict_key": "FHIR.Patient:is_resource_type"
}
~~~

## 16. Rule schema

1. A rule is the smallest unit of kit behavior.
2. A rule MUST state the conditions that make it relevant.
3. A rule MUST state the recommended action.
4. A rule SHOULD state the evidence required before acting.
5. A rule SHOULD state exceptions and non-applicable cases.
6. A rule MUST state its severity when ignored.
7. A rule MUST be bounded by a maximum injection size.
8. A rule MUST declare whether it is advisory or blocking.
9. A rule MUST never claim authority beyond its declared scope.
10. A rule MAY reference a procedure, pattern, or gotcha.
11. A rule MAY add query terms for retrieval expansion.
12. A rule MAY require a tool result before activation.
13. A rule MAY require human approval before execution.
14. A rule MAY emit a validation request rather than an answer.
15. A rule SHOULD have a negative trigger to reduce false positives.
16. Rules are sorted by priority, specificity, and evidence quality.
17. Conflicting rules create a visible conflict record.
18. A private overlay can narrow a public rule but not broaden permissions.
19. A rule update is evaluated against regression cases.
20. Rules with no usable trigger are rejected as context noise.

~~~yaml
id: rule:security:authz-before-authn
kind: rule
title: Check authorization after authentication
when:
  any:
    - task.kind: review
    - file.path.matches: "**/*auth*"
    - symbol.name.contains: "authorize"
  all:
    - workspace.domain: application-security
then:
  action: require-authorization-path-analysis
  inspect:
    - identity-source
    - subject-resource-binding
    - deny-default
    - tenant-boundary
evidence:
  minimum:
    - source-location
    - test-or-policy-reference
exceptions:
  - "Purely local CLI with no protected resource"
severity: high
mode: advisory
max_tokens: 180
sources:
  - ref: owasp:authorization
compression:
  summary: "Authn proves identity; authz proves permission for this resource."
~~~

## 17. Pattern schema

1. A pattern describes a recurring problem and a reusable solution shape.
2. A pattern starts with a problem signature.
3. A pattern names preconditions and constraints.
4. A pattern gives a solution outline.
5. A pattern includes at least one positive example.
6. A pattern includes at least one failure mode.
7. A pattern includes verification steps.
8. A pattern lists alternatives and tradeoffs.
9. A pattern declares language and framework variants.
10. A pattern can point to a procedure for execution.
11. A pattern can point to a gotcha for warnings.
12. Examples are illustrative unless marked as normative.
13. Code examples must pass configured secret scanning.
14. Code examples must carry license metadata when sourced externally.
15. Patterns are compressed by preserving shape before prose.
16. A pattern is retrieved by problem signature and graph neighbors.
17. A pattern can be selected without injecting its full examples.
18. The agent may request an example expansion explicitly.
19. Pattern quality is measured by task success and misuse rate.
20. A pattern with no verification step is low-trust.

~~~json
{
  "id": "pattern:etl:idempotent-load",
  "kind": "pattern",
  "title": "Idempotent incremental load",
  "problem": "A retryable ETL job can duplicate records after partial success.",
  "signature": ["incremental", "retry", "checkpoint", "upsert"],
  "preconditions": ["stable source key", "load watermark", "transaction boundary"],
  "solution": [
    "Read source rows after the committed watermark.",
    "Write into a staging relation keyed by source identity.",
    "Merge staging rows with deterministic conflict policy.",
    "Commit data and watermark atomically."
  ],
  "positive_examples": [
    {"language": "sql", "ref": "examples/idempotent-merge.sql"}
  ],
  "failure_modes": [
    "Advancing the watermark before the data commit loses rows.",
    "Using arrival time instead of source identity duplicates retries."
  ],
  "verification": [
    "Replay the same batch twice.",
    "Kill the worker between write and commit.",
    "Assert row count and watermark are unchanged after replay."
  ],
  "alternatives": ["append-only with downstream deduplication"],
  "tradeoffs": ["requires durable keys and transactional metadata"],
  "trust": "reviewed"
}
~~~

## 18. Procedure schema

1. A procedure is an ordered workflow, not a paragraph of advice.
2. Each step has an id and a human-readable action.
3. Each step can require inputs, tools, or evidence.
4. Each step can emit artifacts and facts.
5. Each step can branch on a typed condition.
6. Each step can stop the workflow with a reason.
7. Procedures declare side-effect class for every tool invocation.
8. Read-only steps are the default.
9. Mutating steps require explicit capability and approval.
10. Destructive steps require an independent confirmation policy.
11. Procedures declare timeouts and retry behavior.
12. Procedures declare which outputs are safe to inject.
13. Procedure text is not permission to execute a tool.
14. The agent still needs the runtime capability and user authorization.
15. A procedure can link to a checklist for human review.
16. A procedure can link to domain schemas for validation.
17. Procedure versions are independently diffable.
18. A procedure with a changed step order requires regression evaluation.
19. The runtime records completed and skipped steps.
20. Procedure state remains session-local unless explicitly exported.

## 19. Gotcha schema

1. A gotcha captures a high-value negative lesson.
2. A gotcha MUST identify a trigger that can be observed.
3. A gotcha MUST state the failure or risk.
4. A gotcha MUST state the safe alternative.
5. A gotcha SHOULD include a minimal counterexample.
6. A gotcha SHOULD include a verification command or test.
7. A gotcha MUST declare severity and confidence.
8. A gotcha can target paths, symbols, tools, or task kinds.
9. A gotcha can be visible before the agent asks a question.
10. A gotcha can be injected as a one-line warning.
11. Gotchas are a primary compression target.
12. The runtime prefers one relevant gotcha over ten generic tips.
13. A false-positive gotcha is tracked as an evaluation failure.
14. A stale gotcha is demoted, not silently deleted.
15. A company can add a local gotcha over a public pattern.
16. A company cannot change a public artifact in place.
17. Gotchas must not reveal confidential incident details in public kits.
18. A gotcha may reference a redacted incident identifier.
19. A gotcha with contradictory resolution enters conflict review.
20. Release gates require no unresolved critical gotcha regressions.

## 20. Glossary and terminology model

1. A glossary entry maps aliases to a canonical concept.
2. A glossary entry can carry pronunciation or casing hints.
3. A glossary entry can define forbidden ambiguous expansions.
4. A glossary entry can declare jurisdiction or product release.
5. Query expansion uses glossary aliases only inside declared scope.
6. Acronyms are expanded in compact context when ambiguity is material.
7. The agent sees the canonical term and the user term when useful.
8. Glossary data is cheap and usually loaded in the warm tier.
9. Sensitive internal names can be kept in a private overlay.
10. Public glossary entries must not include customer identifiers.
11. Glossary changes can alter activation and require evaluation.
12. A term can have several typed meanings.
13. Type selection uses task, file, and graph context.
14. Unknown terms are candidates for kit discovery.
15. The runtime records applied expansions for reproducibility.
16. Glossary collisions are warnings during build.
17. A collision with incompatible scopes is an error in release mode.
18. Glossary entries support locale and spelling variants.
19. Domain kits should include common anti-pattern vocabulary.
20. The glossary is not a substitute for full source evidence.

## 21. Graph model

1. The graph stores entities, concepts, artifacts, and typed relations.
2. Nodes have stable IDs and optional aliases.
3. Nodes can point to code paths, standards, APIs, or procedures.
4. Edges have direction, type, weight, confidence, and provenance.
5. Graph activation starts from task and workspace anchor nodes.
6. Spreading activation is bounded by hop count and token budget.
7. Superseded nodes are retained for audit but deactivated for retrieval.
8. Contradictory nodes are both retained under surface-conflict policy.
9. Edge merge is idempotent.
10. Missing edge endpoints are rejected or recorded as unresolved.
11. A kit graph can merge with a repository code graph.
12. A company overlay can attach local implementation nodes to public concepts.
13. The graph supports impact and dependency questions.
14. The graph supports procedure prerequisite checks.
15. The graph supports domain-aware retrieval reranking.
16. Graph exports are portable and do not contain secrets by default.
17. Graph indexes are derived from canonical node and edge files.
18. Graph compaction preserves high-value relation types.
19. The graph serializer uses stable ordering for deterministic hashes.
20. Graph conflicts are exposed through kit explain.

## 22. Corpus and evidence model

1. Corpus files are evidence, not automatically trusted instructions.
2. Every document has a stable relative path or external URI handle.
3. Every document has a content hash.
4. Every document has a license and source owner.
5. Every document has a freshness class.
6. Every document has an ingestion timestamp.
7. Every document has a redaction report.
8. Every document can be excluded from model injection while remaining searchable.
9. High-risk domains require a citation for claims derived from corpus text.
10. Long documents are chunked using structure-aware boundaries.
11. Tables are represented as typed rows when extraction is reliable.
12. Code is indexed by symbols and structural regions.
13. PDFs retain page and section locators when available.
14. HTML retains canonical URL and heading path.
15. Corpus access can be on-demand to reduce cold-start cost.
16. The corpus can be stored locally, in a private registry, or in an approved vault.
17. Public artifacts exclude unlicensed copies.
18. Private artifacts can reference an internal document provider instead of embedding bytes.
19. A missing corpus source degrades evidence quality and is surfaced.
20. Corpus retention and deletion follow tenant policy.

## 23. Adapter model

1. An adapter is a declarative bridge to an approved local capability.
2. An adapter declares name, input schema, output schema, and side-effect class.
3. An adapter declares required environment permissions.
4. An adapter declares data residency.
5. An adapter declares whether output may enter model context.
6. An adapter declares timeout, retry, and rate limit.
7. Adapters default to read-only.
8. Write adapters require enterprise policy approval.
9. Native code execution is disabled for ordinary kits.
10. A kit can call an existing MCP tool by stable capability name.
11. The runtime validates adapter arguments against JSON Schema.
12. The runtime redacts adapter output before evidence storage.
13. Adapter output is marked observed, not authored.
14. A kit cannot grant a missing user permission.
15. A kit cannot exfiltrate data to its publisher.
16. Adapters are versioned separately from content when their contract changes.
17. An adapter failure does not invalidate unrelated static layers.
18. The kit explain view shows adapter calls and returned handles.
19. Offline mode removes adapters that require network.
20. Adapter conformance tests run in a sandbox before publication.
