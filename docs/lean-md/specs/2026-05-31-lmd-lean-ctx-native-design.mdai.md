---
title: lmd — native lean-ctx Live-Markdown-Engine (rushdown)
slug: lmd-lean-ctx-native
status: draft
date: 2026-05-31
supersedes: docs/mdai/specs/lean-ctx-live-spec-v0.6.md
consumer: ai
markdownai_directives_omitted: >
  Design-Spec über die lmd-Engine selbst — wird von writing-plans und Menschen
  am Stück gelesen, nicht phasen-isoliert dispatcht. Live-Direktiven (@tree/@call)
  würden den markdownai-Renderer voraussetzen, dessen Ablösung diese Spec gerade
  beschreibt. Bewusst statisches Markdown.
---

# lmd — native lean-ctx Live-Markdown-Engine

> lmd ist **kein** externer Konsument von lean-ctx — lmd **erweitert** lean-ctx.
> Die nativen Direktiven sind der lean-ctx/lmd-Core. Hauptziel: kurze, aber sichere
> Pläne/Skills, die strukturell nur lean-ctx-Tools und unter lean-lmd-Tooling adressierte tools, plus
> Subagent-Dispatch, der die Tool-Disziplin erzwingt statt sie zu empfehlen.

Diese Spec erstellen lean-lmd Version 0.1.0.

---

## 1. Ziel & Abgrenzung

**Ziel:** Die in `mdai/` (Node-markdownai) bewiesene Skill-/Plan-Mechanik nativ in
lean-ctx nachbauen — mit rushdown als Parser — und dabei drei Dinge fest verankern:

1. **lean-ctx als oberste Direktive** — strukturell, nicht als Empfehlung.
2. **TDD-Potenzial** (Token Dense Dialect + Tool-Routing): künftige Pläne adressieren
   ab Geburt lean-ctx-Tools; es gibt kein Sprachkonstrukt für rohes `cat`/`grep`/`ls`.
3. **Subagent-Dispatch ohne Drift** — der beobachtete Fehler (Subagents nutzen native
   bash/Read) wird an der Dispatch-Grenze + per Hook geschlossen.

**Strangler, kein Big-Bang:** lmd wird der native Pfad (`ctx_md_*`-MCP-Tools +
`lean-ctx md`-CLI). Node-markdownai bleibt übergangsweise für nicht-migrierte
`.mdai.md`. Voraussetzung jeder Skill-Migration: der Benchmark-Pfad
(Phase-Isolation, `read_file(phase=)`, −92 %) ist in lmd reproduziert **bevor** ein
Skill umgezogen wird.

**Pilot-Reihenfolge:** `mdai-brainstorm` zuerst (verhaltenserhaltende Re-Portierung
markdownai→lmd als Referenz), danach die übrigen Skills mit Hand-over auf lmd.

---

## 2. Verifizierter Code-Befund (Stand 2026-05-31)

### 2.1 Bestätigt — die teure Arbeit existiert bereits

| Annahme der Spec    | Beleg (Datei:Zeile)                                                                                                                                                                                                                                                 | Status        |
|---------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------|
| Graph-Datei-API     | `core/graph_index.rs`: `file_count:304`, `edge_count:312`, `get_reverse_deps:320`, `get_related:343`, `load_or_build:373`                                                                                                                                           | ✅ exakt       |
| Graph-Symbol-API    | `core/call_graph.rs`: `callers_of:416`, `callees_of:424`, `load_or_build:666`                                                                                                                                                                                       | ✅ exakt       |
| Graph-Kontext-API   | `core/graph_context.rs`: `build_graph_context:77`, `build_related_hint:241`, `graph_neighbor_ranks_for_recent_files:263`, `format_graph_context:320`                                                                                                                | ✅ exakt       |
| Session-Bridge      | `core/session/state.rs`: `add_finding:217`, `add_decision:233`                                                                                                                                                                                                      | ✅             |
| Knowledge-Bridge    | `core/knowledge/core.rs:remember:92`, `query.rs:recall:7`/`recall_by_category:40`/`recall_for_output:100`; `retrieval_count` vorhanden                                                                                                                              | ✅             |
| TDD-Dialect-Basis   | `core/tdd_schema.rs`: `tdd_schema_value`, `default_tdd_schema_path`, `write_if_changed`; `bin/gen_tdd_schema.rs`                                                                                                                                                    | ✅             |
| Sync-Tool-Trait     | `server/tool_trait.rs`: `fn handle(&self, args, ctx) -> Result<ToolOutput, ErrorData>` (sync)                                                                                                                                                                       | ✅ §8a korrekt |
| rushdown extensibel | crates.io, CommonMark 0.31.2 + GFM; custom block/inline parsers, `NodeKind`, `RenderNode`, AST-Transformer, `parser_extension(\|p\| p.add_inline_parser(Ctor, Opts, prio))`, `trigger()->&[u8]`, `parse()->Option<NodeRef>`, `.and()` → `new_markdown_to_html(...)` | ✅ §5 gedeckt  |

### 2.2 Fehlt / abweichend — Risiken & Lücken

| Punkt                                          | Befund                                                                                                           | Konsequenz                                                                        |
|------------------------------------------------|------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------|
| `rushdown`, `evalexpr` als Deps                | **nicht** in `rust/Cargo.toml`/`Cargo.lock`; `rushdown` nur in der Spec erwähnt                                  | Phase-0-Spike nötig (API-Ergonomie 0.17)                                          |
| `tools/ctx_md.rs`, `cli/md_cmd.rs`, `src/lmd/` | existieren nicht                                                                                                 | Greenfield — wie erwartet                                                         |
| `session.recently_touched_files()`             | existiert **nicht** auf Session-State                                                                            | `@graph recent-neighbors` braucht neue kleine API **oder** wird in v1 weggelassen |
| Node-markdownai                                | unter `markdownai/` (node_modules, MCP-Server, `MDs/`) aktiv; mdai-Skills hängen an `mcp__markdownai__read_file` | Strangler: parallel halten, pro Skill ablösen                                     |

### 2.3 Hook-Befund — Enforcement ist bereits hart

Global in `~/.claude/settings.json` (PreToolUse):

- `read-tool-discipline.py` → **hard deny** für native `Read`/`Grep`/`ListFiles`,
  Reason verweist auf `ctx_read`/`ctx_search`/`ctx_tree` (+ Serena für Symbole).
  Ausnahme nur `Glob`.
- `bash-enforce-ctx-shell.py` → **hard deny** für Drift-Familien (`cat/head/sed/awk`,
  `grep/rg/find/ls`, `git/gh`, `cargo/npm`, `wc`, `mkdir`…) mit MCP-Ersatz-Hinweis;
  sonst pass-through-rewrite via `lean-ctx hook rewrite` + Nudge.
- `edit-tool-discipline.py`, `plan-discipline.py`, markdownai-`preToolUse.mjs`,
  `lean-ctx hook observe` auf Session-Events.

**Diagnose Subagent-Drift:** Im Hauptsession-Loop greift der Deny. An der
**Dispatch-Grenze** bricht die Kette: (a) der Subagent-Prompt trägt die Disziplin
nicht zwingend mit; (b) lean-ctx-MCP-Tools sind im Subagent oft *deferred* — er
bekommt einen Deny, hat das Ersatz-Tool aber nicht geladen und ruft kein
`ToolSearch(select:…)`; (c) ob PreToolUse-Hooks im Subagent-Loop des Ziel-Harness
feuern, ist unverifiziert.

### 2.4 Regel-/Macro-Bibliothek heute (`mdai/core`, `mdai/tooling`)

- `hard-rules.md` — Always-on-Regeln (`@markdownai v1.0`-Fragment).
- `tool-quick-ref.md` — Task → bevorzugtes `@call <macro>` → Fallback-Tabelle.
- `ctx-tools.md` — `mode: import-only`-Pack: `@define ctx_read(path,mode)` →
  `@query mcp lean-ctx ctx_read …`.
- `startup-check.md` — `mdai_bootstrap`/`detect_*`-Orchestrierung; löst pro Call
  Include-Expansion **plus mehrere `ctx_shell`-Round-Trips** aus, obwohl das Binary
  Sprach-/Tool-/Hook-/Session-Status nativ kennt → **Rust-Built-in-Material**.
- `tooling/serena.md`, `tooling/jetbrains.md` — Macro-Packs für externe Tools.

### 2.5 Benchmark-Abgleich (`mdai-benchmark.md`) — empirische Zielmarken

Maßgeblich ist **Benchmark v5** (markdownai **1.3.0** = `feat-mdai`-Dist, mdai-Library
**v0.1.4**, 2026-05-31, tiktoken `cl100k_base` exakt). Frühere Zahlen (v3 Bytes÷4) sind
überholt — der Engine-**Render ist über 1.0.0→1.3.0 byte-stabil** (≈0,4–1,5 % Drift);
die scheinbaren Token-Sprünge sind reine Tokenizer-Methodik, kein Verhaltens-Drift. lmd
re-misst mit tiktoken; die v5-Zahlen sind die Zielmarken.

| Messpunkt (v5, tiktoken)          | Zahl                         | Bedeutung für lmd                                                   |
|-----------------------------------|------------------------------|---------------------------------------------------------------------|
| Original-Plan S3a                 | 10 629 Tok                   | Baseline „Subagent kriegt alles"                                    |
| Voll-Render (−73 %)               | 2 852 Tok                    | **überwiegend manuelle Prosa-Verdichtung**, *nicht* der lmd-Gewinn  |
| S3a Einzelphase vs. Original      | −88…−93 % (A3 = 829 Tok)     | strukturell — **lmd-Zielmarke pro Phase**                           |
| body `handoff` / Part-B größte    | 291 (−95 %) / 3 234 (−63 %)  | Skill-Phase schlank vs. große Plan-Phase (strukturelle Untergrenze) |
| Hard-Rules + Constraints-Overhead | ~480 Tok/Phase               | by design nicht weglassbar; muss in jeden Dispatch (§3.5)           |
| 7-Subagent-Kosten (Sonnet)        | 61 677 → 9 585 Tok (−84.5 %) | Kostenmodell, das lmd-Dispatch erreichen muss                       |

**Kernaussage des Benchmarks (bestätigt unsere Architektur):** Der echte,
nicht-marginale Gewinn ist **Phase-Isolation beim Subagent-Dispatch**, nicht die
Source/Render-Verkleinerung. → rechtfertigt, dass `@dispatch` + `ctx_md_read_phase`
v1-Kern sind (§3.5, §4.4) und die `@define`/`@include`-Source-Ersparnis (~14–20 %)
nur sekundär ist.

**Drei Funde, die ins Design einfließen:**

1. **Layout entscheidet über die Ersparnis (v5 Finding #4 — neu).** Hard-Rules
   *global* vor den Phasen (S3a) → flach −70…−75 % (jede isolierte Phase schleppt
   ~480 Tok Regel-Overhead). Hard-Rules in *einer* `pre-context`-Phase
   (`body.mdai.md`) → andere Phasen lean, bis **−95 %** (`handoff` = 291 Tok).
   **Design-Konsequenz:** lmd legt den Regel-Block in eine dedizierte `pre-context`-
   Phase; nur der **kompakte built-in `hard-rules`-Kern** (Tool-Disziplin) geht in
   *jeden* Dispatch (Sicherheit, §3.5), das schwere Autoren-Referenzmaterial nur nach
   `pre-context`. Weil lmd `tool-quick-ref`/`ctx-tools` streicht (§3.4), liegt der
   unvermeidbare Per-Dispatch-Overhead deutlich unter markdownais 2 513-Tok-Block —
   wir bekommen **Sicherheit *und* schlanke Phasen**.
2. **MCP-Envelope (in 1.3.0 erledigt):** Node brauchte historisch einen
   Envelope-Wrapper (`{content:[…], structuredContent:…}`, sonst „completed with no
   output"); in der `feat-mdai`-Dist enthalten. lmd bekommt das **gratis** über
   `server/tool_trait.rs` → `ToolOutput`. **Akzeptanzkriterium:** `ctx_md_*` nie rohe
   Objekte emittieren (Regression-Guard).
3. **Finding #3:** `superpowers:subagent-driven-development` macht *manuell* (Plan
   lesen, Per-Task-Text extrahieren, Prompt komponieren), was lmd *strukturell*
   automatisiert. Ein lmd-aware Controller ruft `ctx_md_read_phase(phase=…)` statt
   manueller Plan-Parsing-Logik. → validiert §3.5 Schicht 2.

---

## 3. Lasttragende Architektur-Entscheidungen

### 3.1 Notwendigkeits-Audit zuerst (R / H / E)

lmd ist ein **dünnes Frontend**. Jede Direktive wird vor dem Bau klassifiziert:

- **R (Router)** — lean-ctx-Tool/Core-API kann es schon → dünner Alias, keine neue Logik.
- **H (Hook)** — passiert besser/bereits im Hook-Layer → Engine macht nichts, nur
  **kein Doppel-Tracking** sicherstellen.
- **E (rushdown-Extension)** — echtes Engine-Konstrukt ohne lean-ctx-Äquivalent.

| Direktive                       | Klasse | Backing                                                                        |
|---------------------------------|--------|--------------------------------------------------------------------------------|
| `@read`                         | R      | `core::structured_read` / `ctx_read`                                           |
| `@search`                       | R      | `ctx_search`                                                                   |
| `@list`                         | R      | `ctx_tree`                                                                     |
| `@query` (shell)                | R      | `shell/exec` + compress (+ Security-Gate)                                      |
| `@graph` (Datei/Symbol/Kontext) | R      | `graph_index`/`call_graph`/`graph_context` (verifiziert)                       |
| `@remember`/`@recall`           | R      | `ctx_knowledge` (`remember`/`recall_for_output`, `no_track`)                   |
| `@env`/`@date`/`@count`         | R      | `std::env`/chrono/glob (trivial)                                               |
| `@phase`/`@on complete`         | R+H    | `ctx_session add_decision/add_finding` — **H-Check: schreibt Hook das schon?** |
| `@lean-md` Header               | E      | Config-Parse                                                                   |
| `@include`/`@import`            | E      | File-Inline / Definitions-Scope (fs + jail)                                    |
| `@define`/`@call`               | E      | Macro-Engine — kein lean-ctx-Äquivalent                                        |
| `@if`/`@consumer`               | E      | Container-Transformer (+ evalexpr)                                             |
| `{{ expr }}` / Pipe + `@render` | E      | Inline-Eval / AstTransformer                                                   |
| TDD-Output                      | R+E    | `tdd_schema` (R) + Render-Hook (E)                                             |

**Resultat:** Fast alle *Daten*-Direktiven sind R (Router von je wenigen Zeilen).
Echte rushdown-Arbeit (E) reduziert sich auf **~6 Primitive** (§4). "15 Direktiven,
3 Wochen" kollabiert auf "6 Engine-Primitive + N triviale Bridges".

**Phase 0 (Gate):** Dieses Audit als ausführbares Artefakt (Tabelle pro Direktive:
R/H/E + Backing-Symbol + Bridge-Zeilenschätzung) **+ rushdown-Spike** (1 Block- + 1
Inline-Direktive gegen die echte 0.17-API, Vorbild `tests/extension.rs`). Erst dessen
Ergebnis entscheidet den finalen v1-Umfang.

### 3.2 lmd erweitert lean-ctx — native Direktiven = lean-ctx/lmd-core

Die **einzigen nativen Direktiven sind lean-ctx-Direktiven**. Damit ist "lean-ctx als
oberste Direktive" wörtlich und strukturell: es gibt kein Sprachkonstrukt für rohes
`cat`/`grep`/`ls`. Das Schärfste ist `@query`/`@call ctx_shell`, und das ist
Security-gegatet (§7).

### 3.3 Fragment-Auflösung: built-in-first, Datei-Fallback

| Herkunft                         | Wofür                                                                                           | Kosten                                                                 | Präzedenz                                                          |
|----------------------------------|-------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|--------------------------------------------------------------------|
| **Built-in (Rust, eingebettet)** | stabile, logik-schwere Orchestrierung: `lmd_bootstrap`/`startup-check`, kanonische `hard-rules` | **null** Include/Expand; In-Process-Detektion; keine Shell-Round-Trips | `rules_canonical.rs`, `instructions.rs`, eingebettete `templates/` |
| **Datei `*.lmd.md`**             | autoren-/iterations-/projekt-spezifisch; neue Macro-Packs                                       | Disk-Read + Parse + Expand                                             | `mdai/core/*.md` heute                                             |

**Resolver-Regel:** `@include core/hard-rules` / `@call lmd_bootstrap()` → **erst
Built-in-Registry, dann Datei-Fallback**. Default-Pfad spart Einlesen/Aufruf komplett;
`*.lmd.md` bleibt jederzeit autorierbar (überschreibt/ergänzt Built-in oder rein neu).

- v1 **Built-in:** `lmd_bootstrap`/`startup-check`-Logik (kapselt Detektion, die das
  Binary ohnehin hat), `hard-rules`/`tool-quick-ref` (mit `.lmd.md`-Override).
- v1 **Datei:** externe Tool-Packs `tooling/serena.lmd.md`, `tooling/jetbrains.lmd.md`,
  Sprach-Packs `lang/*.lmd.md`, projektspezifische Fragmente.

### 3.4 Tool-Klassen → lmd-Ausdruck

| Tool-Klasse                                                             | lmd-Ausdruck                                                |
|-------------------------------------------------------------------------|-------------------------------------------------------------|
| lean-ctx/lmd-core (read/search/tree/shell/edit/graph/session/knowledge) | **native Direktive** (R-Router)                             |
| externe Tools (serena, jetbrains, cargo, rustfmt)                       | **`@define`/`@call`-Macro-Pack** (`.lmd.md`, via `@import`) |
| Regeln (hard-rules)                                                     | **`@include`-Fragment** (built-in-first)                    |

**Was portiert wird — und was die Härtung wegfallen lässt:**

| `mdai/core`-Fragment                             | lmd-Schicksal                       | Grund                                                                                                                                                                           |
|--------------------------------------------------|-------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `ctx-tools.md` (`@define ctx_read`→`@query …`)   | **entfällt**                        | `@read`/`@search`/`@tree`/`@shell`/`@edit` sind native Direktiven — ein Macro-Wrapper um ein Tool, das schon Direktive ist, ist redundant                                       |
| `tool-quick-ref.md` (Task→`@call`→Fallback)      | **kein Standalone**                 | lean-ctx-Teil entfällt (native Direktiven selbstdokumentierend + strukturell erzwungen); externe-Tools-Teil → in die Tooling-Packs; Disziplin-Einzeiler → built-in `hard-rules` |
| `hard-rules.md`                                  | → **built-in** (`.lmd.md`-Override) | stabil, in jeden Dispatch                                                                                                                                                       |
| `startup-check.md` (`mdai_bootstrap`)            | → **built-in** `lmd_bootstrap`      | In-Process-Detektion, kein Shell-Fanout (§3.3)                                                                                                                                  |
| `tooling/serena.md`, `jetbrains.md`, `lang/*.md` | → `.lmd.md`-Macro-Pack              | externe Tools haben keinen nativen Sprachkern                                                                                                                                   |

Die `@define`/`@call`-Engine bleibt **v1-Pflicht**, aber nur noch für externe Tools +
Nutzer-Macros, **nicht** für lean-ctx-lmd-core. Konsequenz für den Pilot: der Re-Port
schreibt `@call ctx_read(...)` → `@read …` um (**Output**-Parität, nicht
Source-Parität) und `@call ctx_tree(...)` → `@list …`. Header `@markdownai v1.0` →
`@lean-md`.

### 3.5 Dispatch-Enforcement-Kette (das eigentliche TDD-Ziel)

Drei Schichten, von "vermeiden" zu "blocken":

1. **Quelle (lmd-Skill/-Plan):** I/O *nur* über `@read`/`@search`/`@list`/`@graph`/
   `@query`. Künftige Pläne adressieren von Geburt lean-ctx-Tools. Es gibt kein
   `cat` zu schreiben.
2. **Dispatch (`@dispatch`-Direktive / Phase-Isolation):** beim Subagent-Spawn
   generiert lmd den Prompt aus (a) dem phasen-isolierten, TDD-komprimierten Inhalt
   **+** (b) eingebetteter Tool-Disziplin-Constraint **+** (c) explizitem
   `ToolSearch(select:mcp__lean-ctx__ctx_*)`-Bootstrap, damit deferred-Tools **vor**
   dem ersten Read geladen sind. Der Subagent bekommt ein gerendertes Vorbild statt
   einer Versuchung.
3. **Backstop (Hooks):** bestehende Deny-Hooks bleiben; **Lücke schließen** — der
   Read-Deny lässt kein "Edit braucht native Read"-Schlupfloch mehr offen, weil
   `ctx_edit` das löst; plus verifizieren, dass PreToolUse-Hooks im Subagent-Loop
   greifen (sonst ist der Dispatch-Prompt die einzige Verteidigung).

`@dispatch` + Constraint-Injektion sind **v1-Kern** (in v0.6 §12 fälschlich unter
"NICHT bauen") — sie sind der Grund, warum die Übung das Drift-Problem löst.

---

## 4. Engine-Primitive + Bridge-Trait

### 4.1 Die ~6 E-Primitive (alles andere ist R-Routing)

1. **Block-Direktiven-Parser** — ein rushdown-Block-Parser, Trigger `@` am
   Zeilenanfang, dispatcht alle `@<name>`-Blöcke an die Direktiven-Registry. **Nicht**
   ein Parser pro Direktive.
2. **Inline-Parser** — Trigger `{{` und `@` inline (`{{ expr }}`, `@recall`,
   `@on complete`).
3. **Container-Transformer** — `@if`/`@elseif`/`@else`/`@if-end` + `@consumer=ai/human`
   als Whole-AST-Transformer (rushdown AST-Transformer + evalexpr).
4. **Macro-Engine** — `@define`/`@call` mit Parameter-Substitution (`{{ param }}`),
   `@include` (Content sichtbar) / `@import` (nur Definitions). Kein lean-ctx-Äquivalent.
5. **Pipe + `@render`** — Postfix-AstTransformer (`… | @render type=table`).
6. **TDD-Render-Hook** — Renderer-Ausgang hängt an `tdd_schema` (Modi `tdd/compact/off`
   aus `@lean-md`-Header).

Dazu der **Header-Parser** (`@lean-md`) und die **Fragment-Registry** (§3.3).

### 4.2 Bridge-Trait (sync, wie alle lean-ctx-Tools)

```rust
// src/lmd/bridges/mod.rs
pub trait DirectiveBridge {
    fn name(&self) -> &'static str;
    // sync — async-Wrapping kommt gratis aus dem MCP-Worker-Pool (v0.6 §8a)
    fn execute(&self, ctx: &EngineContext, args: &DirectiveArgs)
               -> Result<String, BridgeError>;
}
```

Jede R-Direktive ist eine Bridge, die in eine existierende Core-API routet. Beispiel
`@graph` (alle Ops via vorhandene Symbole, ~80 Zeilen für 7 Ops — siehe v0.6 §7).
`@read`/`@search`/`@list` rufen dieselben Core-Funktionen wie `ctx_read`/`ctx_search`/
`ctx_tree`. Keine Neualgorithmik.

### 4.3 rushdown-Extension-Mapping

```rust
// src/lmd/parser/block.rs (Skizze)
fn lmd_extension() -> impl ParserExtension {
    parser_extension(|p| {
        p.add_block_parser(LmdBlockParser::new, NoParserOptions, PRIORITY_BLOCK_HIGH);
        p.add_inline_parser(LmdInlineParser::new, NoParserOptions, PRIORITY_EMPHASIS + 100);
    })
}
// LmdBlockParser: trigger() -> b"@"; parse() -> Option<NodeRef> (NodeKind::LmdDirective{name,args})
// RenderNode für NodeKind::LmdDirective ruft die Bridge-Registry und schreibt das Ergebnis.
```

### 4.4 Neue Integrationspunkte

- `src/lmd/` (Modul-Baum aus v0.6 §5).
- `src/tools/ctx_md.rs` — `ctx_md_render`/`ctx_md_read_phase`/`ctx_md_list_phases`/
  `ctx_md_constraints` MCP-Tools (Strangler-Ersatz für `mcp__markdownai__*`).
- `src/cli/md_cmd.rs` — `lean-ctx md render|read|phases <file>`.
- Cargo-Deps: `rushdown` (Version nach Spike fixieren), `evalexpr`.

---

## 5. Skill-Migrations-Kette

### 5.0 Bibliotheks-Migration (Voraussetzung — vor dem Pilot)

Vollständige Inventur von `mdai/` → lmd. Was kollabiert, ist in §3.4 begründet; hier
die komplette Abbildung inkl. `lang/` und `tooling/`:

| `mdai/`-Quelle                                                                                 | lmd-Ziel                           | Klasse                                                        |
|------------------------------------------------------------------------------------------------|------------------------------------|---------------------------------------------------------------|
| `core/hard-rules.md`                                                                           | **built-in** (`.lmd.md`-Override)  | Regel-Fragment                                                |
| `core/startup-check.md` (`mdai_bootstrap`, `detect_*`, `load_lang_pack`, `load_tooling_packs`) | **built-in** `lmd_bootstrap`       | In-Process-Orchestrierung                                     |
| `core/ctx-tools.md`                                                                            | **entfällt**                       | → native Direktiven                                           |
| `core/tool-quick-ref.md`                                                                       | **entfällt als Standalone**        | → §3.4 (Core nativ, extern in Packs, Disziplin in hard-rules) |
| `tooling/serena.md`                                                                            | `lmd/lib/tooling/serena.lmd.md`    | `.lmd.md`-Macro-Pack (externes MCP)                           |
| `tooling/jetbrains.md`                                                                         | `lmd/lib/tooling/jetbrains.lmd.md` | `.lmd.md`-Macro-Pack (externes MCP)                           |
| `lang/rust.md` (+ künftige `python`/`node`)                                                    | `lmd/lib/lang/rust.lmd.md`         | `.lmd.md`-Macro-Pack (cargo/rustfmt via `@query`)             |

`lang/`+`tooling/` **bleiben nötig** — sie wrappen externe Tools/Commands ohne nativen
Sprachkern und single-sourcen die exakten Flags (`-D warnings`, `nextest` statt `test`).
Der built-in **`lmd_bootstrap`** übernimmt die Detektion (Sprache, MDAI_HAS_*) und
`@include`'t die passenden Packs — ersetzt `load_lang_pack`/`load_tooling_packs`
in-process (kein Shell-Fanout). Header `@markdownai v1.0` → `@lean-md`, v2-Closer.

### 5.1 Pilot: `mdai-brainstorm` re-portieren (Referenz)

`body.mdai.md` → `body.lmd.md`; `mcp__markdownai__read_file` → `ctx_md_read_phase`;
`@call ctx_read`/`ctx_tree` → native `@read`/`@list`; `@call find_symbol`/`cargo_*`
bleiben (Packs aus §5.0). **Akzeptanz:** golden-output-Parity gegen den Node-Render
(gleicher sichtbarer Inhalt pro Phase) **+** Phase-Isolation-Token-Check (`read_phase`
liefert nur die Phase + Hard-Rules, kein Cross-Phase-Leak, §8.3).

### 5.2 Hand-over-Kette

Danach pro Skill, jeweils mit Hand-over-Notiz an den nächsten: `writing-plans`
(erzeugt lmd-Pläne, die `@read`/`@graph`/`@phase`/`@remember` adressieren) →
`executing-plans` / `test-driven-development` (Phasen-Execution + `ctx_session`-
Anbindung; hier wird Q-05 scharf, §9).

---

## 6. Phasenplan (Sequenz)

| Phase | Inhalt                                                                                     | Gate / Ergebnis                                                         |
|-------|--------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|
| **0** | Notwendigkeits-Audit (R/H/E je Direktive) + rushdown-0.17-Spike (1 Block + 1 Inline)       | entscheidet finalen v1-Umfang; bricht ab, falls rushdown-API ungeeignet |
| **1** | Header-Parser + Block/Inline-Parser + Bridge-Registry + Fragment-Resolver (built-in-first) | `@lean-md`, `@include`, ein R-Router (`@read`) rendern e2e              |
| **2** | R-Bridges: `@read`/`@search`/`@list`/`@query`/`@graph`/`@env`/`@date`/`@count`             | Daten-Direktiven live                                                   |
| **3** | E-Konstrukte: `@define`/`@call`, `@import`, `@if`/`@consumer`, `{{ }}`, Pipe/`@render`     | Macro-Engine + Container live                                           |
| **4** | Bridges `@phase`/`@on complete` (+ H-Check Doppel-Tracking), `@remember`/`@recall`         | Session/Knowledge live                                                  |
| **5** | `@dispatch` + Tool-Disziplin-Constraint-Injektion + Hook-Lücke schließen                   | Subagent-Dispatch ohne Drift                                            |
| **6** | TDD-Render-Hook (`tdd_schema`)                                                             | Output-Kompression                                                      |
| **7** | `ctx_md_*`-MCP-Tools + `lean-ctx md`-CLI                                                   | Strangler-Oberfläche                                                    |
| **8** | Pilot-Migration `mdai-brainstorm` + Parity-/Phase-Isolation-Tests                          | erster Skill auf lmd                                                    |

---

## 7. Security + Strangler-Schnitt

- **Jail:** `[lmd.security] mode=strict jail_root="."`; `@include`/`@import` nur
  innerhalb Jail (`max_chain_depth=16`), keine Symlinks aus dem Jail.
- **Shell-Gate:** `@query`/`@call ctx_shell` nur mit `@lean-md shell=allow` **und**
  `[lmd.security.shell] enabled=true`; deny-patterns (`rm *`, `sudo *`, `curl *`).
  `@query` läuft durch dieselbe Allowlist/Redaction wie `ctx_shell`.
- **Knowledge-Schreibrechte:** `profile=skill` darf `@remember`, `profile=doc` nicht
  (`doc_can_recall=true`).
- **Strangler-Schnitt:** Node-markdownai bleibt, bis ein Skill migriert ist; pro Skill
  Cutover erst nach bestandener Parity + Phase-Isolation-Test. `mcp__markdownai__*`
  und `ctx_md_*` koexistieren in dieser Zeit; kein gleichzeitiges Doppel-Rendern
  derselben Datei.
- **Audit-Log:** keins — `ctx_session`/`ctx_knowledge` haben eigene Stats
  (Doppel-Tracking vermeiden, vgl. §3.1 H-Check).

---

## 8. Test- / Parity-Strategie

1. **rushdown-Spike-Akzeptanz (Phase 0):** ein Custom-`@`-Block + ein Inline-`{{ }}`
   rendern korrekt; Abbruchkriterium dokumentiert, falls die Extension-API die
   Direktiven nicht sauber ausdrückt (Fallback-Pfad: Preprocessor vor rushdown).
2. **Golden-Output-Parity:** für jede migrierte Direktive/jeden Skill rendert lmd
   byte-nah identisch zum Node-markdownai-Output (Snapshot-Tests).
3. **Phase-Isolation-Token-Check (gegen Benchmark-Zielmarken §2.5):**
   `ctx_md_read_phase(file, phase)` liefert nur die Phase + Hard-Rules, kein
   Cross-Phase-Leak (Sentinel-Strings wie im Benchmark v4). Tiktoken-Re-Messung muss
   die Marken treffen: kleine Phase ≈ −88…−95 % vs. Original-Plan, 7-Subagent-Dispatch
   ≈ 9 585 Tok (vs. 61 677 ohne Isolation). Abweichung > 10 % ist ein Befund.
4. **MCP-Envelope-Test:** `ctx_md_*` liefern `{content:[…], structuredContent:…}`,
   nie rohe Objekte (Regression-Guard gegen die Node-`respondTool`-Bug-Klasse, §2.5).
5. **Bridge-Unit-Tests:** je R-Bridge gegen die Core-API (z. B. `@graph dependents`
   == `graph_index::get_reverse_deps`).
6. **Dispatch-Drift-Test:** ein dispatchter Subagent mit lmd-Prompt führt **keine**
   nativen `Read`/`bash cat`-Calls aus (Hook-Deny-Zähler == 0).
7. Tests via `cargo nextest run`.

---

## 9. Offene Punkte / deferred

| ID   | Frage                                                                        | Status                                                                                         |
|------|------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------|
| Q-05 | `@phase`-Fehlerverhalten (abort vs. continue)                                | **deferred** — wird in der `executing-plans`-Migration (§5.2) scharf, nicht in der Engine-Spec |
| G-1  | `@graph recent-neighbors` braucht `session.recently_touched_files()` (fehlt) | kleine neue Session-API **oder** Op in v1 weglassen — Entscheid in Phase 0                     |
| R-1  | rushdown-0.17-API-Ergonomie / exakte Version                                 | Phase-0-Spike fixiert Version + bestätigt Extension-Pfad                                       |

Übergangs-Default Q-05 (wie v0.6 §8): Phase läuft Body sequentiell; Error wird als
`decision`-Eintrag geschlossen, Render bricht nicht ab.

---

## 10. Was wir bewusst NICHT bauen

- `@http`, `@db` — kein neuer externer Code.
- `@graph export-html` — bleibt `lean-ctx graph export-html`-CLI.
- Eigene Cache-Schicht — Session-/mtime-Cache von lean-ctx reicht.
- Audit-Log — Session/Knowledge-Stats reichen.
- Custom `@consumer=*`-Audiences — nur `ai`/`human`.
- Parser pro Direktive — ein Block- + ein Inline-Parser dispatchen alle.

---

*Status: v0.7 draft — an verifizierten Code-Befund + Adoption (Hooks, Regel-Bibliothek,
Dispatch) gebunden. Nächster Schritt nach Review: writing-plans (Phase-0-Gate zuerst).*
