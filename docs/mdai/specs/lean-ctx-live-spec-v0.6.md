# lean-ctx Live-Markdown — Projekt-Spec (v0.6)

> Direktiven sind dünne Markdown-Frontends für lean-ctx's bestehende Datenlayer. **Hauptziel: Token-Ersparnis** — beim
> Schreiben, Auslesen und Speichern. Folder-Name: **`lmd/`** (confirmed).

---

## 1. Was sich gegenüber v0.5 geändert hat

| v0.5                                  | v0.6                                                                                      |
|---------------------------------------|-------------------------------------------------------------------------------------------|
| Q-09 (sync vs. async engine) offen    | **Entschieden: sync Interface, async Wrapper** — passt zum existierenden lean-ctx Pattern |
| Config-Namespace `[live.*]` (Altlast) | **`[lmd.*]`** — konsistent mit Folder `lmd/` und Header `@lean-md`                        |

---

## 2. Drei Hebel der Token-Effizienz (unverändert aus v0.4)

| Hebel                         | Wirkung                                |
|-------------------------------|----------------------------------------|
| **`@consumer=ai`**            | Strippt Human-Prosa für AI-Konsumenten |
| **Progressive `@read`**       | Map → Signatures → Lines → Full        |
| **TDD (Token Dense Dialect)** | Symbol-Shorthand auf Outputs           |

Kombiniert in Skill-Dateien realistisch −65 bis −75% gegenüber Roh-Markdown.

---

## 3. Der `@lean-md` Header (unverändert)

```markdown
@lean-md
@lean-md v1 shell=allow profile=skill tdd=aggressive
```

| Option                     | Default  | Wirkung                      |
|----------------------------|----------|------------------------------|
| `v1`                       | implizit | Spec-Version pinnen          |
| `shell={allow,deny}`       | `deny`   | Master-Switch für `@query`   |
| `profile={skill,plan,doc}` | `doc`    | Voreinstellungen je Use-Case |
| `tdd={tdd,compact,off}`    | inherit  | Output-Compression-Mode      |

---

## 4. Direktiven-Set v1

### 4.1 Übersicht

| Direktive                              | Wrapt                                          | Form              |
|----------------------------------------|------------------------------------------------|-------------------|
| `@query`                               | Shell-Hook + Compression                       | Block             |
| `@read`                                | `ctx_read` mit Mode-Wahl                       | Block + inline    |
| `@search`                              | `ctx_search`                                   | Block             |
| `@list`                                | `ctx_tree`                                     | Block             |
| `@include`                             | File-Read, **Content sichtbar**                | Block             |
| `@import`                              | File-Read, **nur Definitions**                 | Block             |
| `@define` / `@call`                    | Eigene Engine                                  | Block / inline    |
| `@env`                                 | `std::env`                                     | Block             |
| `@if` / `@elseif` / `@else` / `@endif` | evalexpr                                       | Block (Container) |
| `@date`, `@count`                      | chrono, Glob                                   | Block + inline    |
| `@phase` / `@on complete`              | `ctx_session`                                  | Block / inline    |
| `@remember` / `@recall`                | `ctx_knowledge`                                | inline            |
| `@graph`                               | `graph_index` + `call_graph` + `graph_context` | Block + inline    |
| `@consumer={ai,human}`                 | Render-Filter                                  | Block (Container) |
| `{{ expr }}`                           | Expression-Eval                                | Inline            |
| `\|` Pipe + `@render`                  | AstTransformer                                 | Postfix           |

### 4.2 `@include` vs. `@import` (unverändert)

**`@include`** fügt Content sichtbar in den Output ein.  
**`@import`** zieht nur Definitions (Macros, env-Defaults) in den Scope; kein Content im Output.

### 4.3 Token-Efficiency-Direktiven (unverändert)

`@consumer=ai`/`@consumer=human` als Container-Filter. `@on complete` strikt inline. `@recall` strikt inline, rendert
direkt. `@remember` strikt inline mit Side-Effect.

### 4.4 `@read` — Progressive Reads (unverändert)

Default ohne `mode=` ist immer `map`. `full` nur explizit.

### 4.5 `@phase` / `@on complete` → `lean-ctx session` (unverändert)

```markdown
@phase pre-flight
@env DEPLOY_TOKEN required
@query git status --porcelain label=dirty
@if {{ dirty }} != ""
@on complete decision="Aborted — working tree dirty"
@else
@on complete finding="Pre-flight clean"
@endif
@end
```

### 4.6 `@graph` — vollständige API-Surface (überarbeitet)

`lean-ctx` hat heute schon eine sehr reiche Graph-API. `@graph` macht sie als Markdown-Direktive verfügbar — alle
Sub-Operationen mit voller Tiefe ab v1.

**Datei-Ebene (via `graph_index`):**

```markdown
@graph build root=.
> Built: {{ graph_files }} files, {{ graph_edges }} edges

@graph dependents file=src/auth.rs depth=2
> Wer importiert auth.rs (transitiv bis Tiefe 2)?
> Backend: graph_index::get_reverse_deps(path, depth)

@graph related file=src/auth.rs depth=3
> Alles strukturell mit auth.rs verbunden (beide Richtungen, jede Edge-Art)
> Backend: graph_index::get_related(path, depth)
```

**Symbol-Ebene (via `call_graph`):**

```markdown
@graph callers symbol=auth_check
> Welche Funktionen rufen auth_check() auf?
> Backend: call_graph::callers_of(symbol)

@graph callees symbol=handle_login
> Welche Funktionen werden von handle_login() aufgerufen?
> Backend: call_graph::callees_of(symbol)
```

**Kontext-Ebene (via `graph_context`):**

```markdown
@graph context file=src/auth.rs
> Strukturierter Graph-Kontext (Imports, Exports, Dependent-Files)
> Backend: graph_context::build_graph_context

@graph hint file=src/auth.rs limit=5
> Top-5 wahrscheinlich relevante Files (rang-gewichtet)
> Backend: graph_context::build_related_hint

@graph recent-neighbors
> Files in Nachbarschaft der zuletzt berührten Dateien
> Backend: graph_context::graph_neighbor_ranks_for_recent_files
> (Direkter Lookup im Session-State — keine separate Param-Liste)
```

**Argumente:** alle Datei-Operationen unterstützen `depth=N` (default je nach Op: 1 für dependents, 2 für related);
Symbol-Operationen brauchen `symbol=name`.

**Output-Form:** Default TDD-komprimierte Liste mit `file:line` oder `symbol@file`-Refs. Mit `\| @render type=table` als
Tabelle.

---

## 5. Architektur (Folder-Name confirmed)

```
rust/
└── src/
    ├── (bestehend)
    ├── cli/
    │   └── md_cmd.rs              # NEU: lean-ctx md <subcmd>
    ├── tools/
    │   └── ctx_md.rs              # NEU: ctx_md_* MCP-Tools
    └── lmd/                       # confirmed
        ├── mod.rs
        ├── header.rs              # @lean-md Parser
        ├── nodes.rs               # Custom NodeKind
        ├── parser/
        │   ├── block.rs           # DirectiveBlockParser
        │   ├── inline.rs          # {{ }}, !`cmd`, @recall, @on complete
        │   └── transformer.rs     # AstTransformer (Container + Pipes)
        ├── engine/
        │   ├── resolver.rs
        │   ├── expression.rs
        │   ├── context.rs
        │   ├── security.rs
        │   └── consumer.rs        # @consumer=ai/human Filter
        ├── bridges/
        │   ├── session.rs         # @phase / @on complete
        │   ├── knowledge.rs       # @remember / @recall
        │   ├── graph.rs           # @graph (alle Sub-Operationen)
        │   ├── read.rs            # @read
        │   └── search.rs          # @search
        └── renderer.rs            # mit TDD-Compression-Hook
```

Neue Cargo-Deps: `rushdown = "=0.17"`, `evalexpr = "12"`.

---

## 6. Entscheidung Q-04: Counter an, mit `no_track=true` Opt-out

`ctx_knowledge` incrementiert heute `retrieval_count` und `last_retrieved` bei jedem `recall`. Drei native Features
hängen daran:

| Feature           | Wie genutzt                                                                |
|-------------------|----------------------------------------------------------------------------|
| **Stale-Cleanup** | `age_days > 30 && retrieval_count == 0` → `stale_candidates`-Kandidat      |
| **Ranking**       | Recall-Output sortiert nach `retrieval_count` (häufig genutzte Facts oben) |
| **Quality-Score** | `quality_score()` enthält Recency-Bonus aus `last_retrieved`               |

**Behaltene Defaults:** Counter wird incrementiert. Write erfolgt async im Background-Thread.

**Neuer Opt-out:**

```markdown
@recall category="deploy" key="last_version" no_track=true
```

Use-Case: AI-Agent exploriert mehrere Recalls beim Nachdenken; nur das finale „echte" Recall soll zählen.

**Empfehlung an Skill-Autoren:** Default belassen. `no_track=true` nur setzen, wenn das Recall innerhalb einer
Exploration-Schleife steht.

---

## 7. Entscheidung Q-08: volle Graph-API ab v1

Begründung: `graph_index::get_reverse_deps(path, depth)` und `get_related(path, depth)` machen schon nativ BFS mit
konfigurierbarer Tiefe. Plus `call_graph` für Symbol-Granularität und `graph_context` für gerankte Hints. **Die teure
Arbeit ist getan** — `@graph` Bridge ist Routing, kein neuer Algorithmus.

Konkrete Bridge-Mapping:

```rust
// lmd/bridges/graph.rs (Skizze)
pub fn execute_graph(ctx: &EngineContext, d: &GraphDirective)
                     -> Result<String, BridgeError>
{
    match d.op {
        GraphOp::Build => {
            let idx = crate::core::graph_index::load_or_build(&d.root);
            Ok(format!("Built: {} files, {} edges",
                       idx.file_count(), idx.edge_count()))
        }
        GraphOp::Dependents { file, depth } => {
            let idx = crate::core::graph_index::load_or_build(&ctx.project_root);
            Ok(format_paths(idx.get_reverse_deps(&file, depth)))
        }
        GraphOp::Related { file, depth } => {
            let idx = crate::core::graph_index::load_or_build(&ctx.project_root);
            Ok(format_paths(idx.get_related(&file, depth)))
        }
        GraphOp::Callers { symbol } => {
            let idx = crate::core::graph_index::load_or_build(&ctx.project_root);
            let cg = crate::core::call_graph::CallGraph::load_or_build(
                &ctx.project_root, &idx);
            Ok(format_edges(cg.callers_of(&symbol)))
        }
        GraphOp::Callees { symbol } => { /* analog */ }
        GraphOp::Context { file } => {
            let ctx_data = crate::core::graph_context::build_graph_context(
                &file, &ctx.project_root, /* options */);
            Ok(crate::core::graph_context::format_graph_context(&ctx_data))
        }
        GraphOp::Hint { file, limit } => {
            Ok(crate::core::graph_context::build_related_hint(
                &file, &ctx.project_root, limit).unwrap_or_default())
        }
        GraphOp::RecentNeighbors => {
            let recents = ctx.session().recently_touched_files();
            let ranks = crate::core::graph_context::graph_neighbor_ranks_for_recent_files(
                /* ... */);
            Ok(format_ranks(ranks))
        }
    }
}
```

Bridge-Größe: ~80 Zeilen für 7 Sub-Operationen, weil alle Algorithmen schon existieren.

---

## 8. Q-05 → deferred

Phase-Fehlerverhalten (`abort` vs. `continue` bei Exception in `@phase`-Body) wird **bewusst nicht in dieser Spec
entschieden**.

Begründung: Plan-Brainstorming, -Schreiben und -Ausführen sind ein eigenes Arbeitsfeld, das parallel oder später
entsteht. Die Fehler-Semantik gehört in den Plan-Execution-Spec, nicht in die Direktiven-Engine. Solange dieser Bereich
nicht definiert ist, ist Vorab-Entscheiden eine Quelle für Rework.

**Übergangs-Default:** `@phase` läuft den Body sequentiell ab. Wenn eine Direktive einen Error returniert, wird die
Phase mit dem Error-Text als `decision`-Eintrag geschlossen, der Render bricht nicht ab, weitere Phases laufen weiter.
Das ist konservativ und entspricht dem Verhalten von `lean-ctx session add_decision()` als Error-Pfad.

---

## 8a. Entscheidung Q-09: Sync-Interface, async Wrapper

Die lmd-Engine bekommt das **gleiche Sync/Async-Pattern wie alle existierenden lean-ctx-Tools**. Begründet durch
direkten Code-Befund.

### Was lean-ctx heute macht

| Schicht                           | Modus                                | Beleg                                                                                                                                  |
|-----------------------------------|--------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------|
| Binary-Entry (`fn main`)          | sync                                 | `src/main.rs`                                                                                                                          |
| CLI-Subcommands                   | sync                                 | `src/cli/dispatch.rs`                                                                                                                  |
| MCP-Server-Mode                   | **explizit async** via tokio-Runtime | `src/cli/dispatch.rs:1580` baut `Builder::new_multi_thread()` mit `LEAN_CTX_WORKER_THREADS`-konfigurierbaren Workern (8..=32 blocking) |
| MCP-Transport (stdin/stdout)      | async via rmcp + `tokio::io`         | `src/mcp_stdio.rs`                                                                                                                     |
| Tool-Handler-Interface            | sync                                 | `src/server/tool_trait.rs`: `fn handle(&self, args, ctx) -> Result<ToolOutput, ErrorData>`                                             |
| Bridge zwischen den Schichten     | `tokio::task::block_in_place`        | `src/server/dispatch/mod.rs:198`                                                                                                       |
| Fire-and-Forget (Stats, Autosave) | `tokio::task::spawn_blocking`        | `src/server/mod.rs` mehrfach                                                                                                           |

### Konsequenz für lmd

- **Resolver, Bridges, Direktiven-Handler: alle sync.** Keine `async fn`, kein `.await` in unserem Code.
- **Async-Wrapping bekommen wir gratis.** Wenn ein Agent `ctx_md_render` aufruft, läuft das im Worker-Pool, mehrere
  Renders parallel ohne dass wir etwas tun müssen.
- **Lernkurve = 0.** Wer einen `ctx_session`-Handler lesen kann, kann eine lmd-Bridge schreiben.

### Performance-Anmerkung (nicht-blockierend für Design)

Eine langlaufende Direktive wie `@query find / -type f` belegt einen Worker-Thread für die Dauer der Ausführung. Bei
wenigen Workern + vielen parallelen Agent-Sessions kann das Pool-Druck erzeugen. Das ist das identische Problem wie
heute bei `ctx_shell` mit langem Command. Wenn es relevant wird, ist das Standard-Mitigation:

```rust
// In bridges/shell.rs, für known-long-running commands:
let output = tokio::task::spawn_blocking( move | | run_shell(cmd)).await?;
```

Aber das ist **keine** Design-Entscheidung für v1 — wir folgen exakt dem bestehenden Pattern, und wenn Pool-Druck zum
Problem wird, hat lean-ctx als Ganzes das Problem zu lösen, nicht spezifisch lmd.

---

## 9. Sicherheits-Modell (gekürzt)

```toml
[lmd]
profile = "doc"
require_header = true

[lmd.security]
mode = "strict"
jail_root = "."

[lmd.security.shell]
enabled = false                  # plus @lean-md shell=allow nötig
allow_patterns = []
deny_patterns = ["rm *", "sudo *", "curl *", "wget *"]

[lmd.security.filesystem]
max_file_size_bytes = 10_485_760
follow_symlinks = false

[lmd.security.knowledge]
skill_can_remember = true
skill_can_recall = true
doc_can_remember = false
doc_can_recall = true

[lmd.security.imports]
max_chain_depth = 16
allow_outside_jail = false

[lmd.security.consumer]
default_audience = "ai"
allow_mixed_audiences = true
```

Audit-Log gestrichen — `ctx_session` und `ctx_knowledge` haben eigene Stats; doppeltes Tracking wäre Redundanz.

---

## 10. Verbleibende offene Punkte

| ID   | Frage                    | Status                                       |
|------|--------------------------|----------------------------------------------|
| Q-05 | `@phase`-Fehlerverhalten | **Deferred** — gehört in Plan-Execution-Spec |

(Q-01, Q-04, Q-06, Q-08, Q-09, Q-10, Q-11 alle entschieden.)

---

## 11. Erste konkrete Schritte (unverändert)

1. **`@lean-md` Header-Parser** in `lmd/header.rs`. 1 Tag.
2. **`@query` Spike** mit Shell-Compression-Bridge. 2 Tage.
3. **`@read` Bridge** mit Mode-Wahl. 1 Tag.
4. **`{{ env.VAR }}` + `@env`**. 1 Tag.
5. **`@if`/`@elseif`/`@else`/`@endif`** via AstTransformer + evalexpr. 2–3 Tage.
6. **`@consumer=ai/human`**. 1 Tag.
7. **`@phase`/`@on complete`** Bridge zu `ctx_session`. 1 Tag.
8. **`@remember`/`@recall`** Bridge zu `ctx_knowledge`. 1 Tag.
9. **`@search` + `@read lines=` Pattern**. 1 Tag.
10. **`@graph`** Bridge zu allen 7 Sub-Operationen. 1–2 Tage.
11. **`@import`** (Definitions-only). 1 Tag.
12. **`@include`** (Content). 0.5 Tage.
13. **`@define`/`@call` Macros**. 3–4 Tage.
14. **Pipe & Render**. 2 Tage.
15. **TDD-Integration in Renderer-Output**. 1–2 Tage.

Summe: ~3 Wochen für vollständige v1.

---

## 12. Was wir bewusst NICHT bauen

- `@http`, `@db` — kein neuer externer Code
- `@graph export-html` (visual) — `lean-ctx graph export-html` bleibt als CLI; nicht als Direktive nötig
- `@constraint`, `@prompt`, `@note` — späterer Bedarf
- Audit-Log (`@read full` etc.) — Session-Tracker reicht
- Phase-Fehler-Semantik — gehört in Plan-Execution-Spec
- Custom `@consumer=*` Audiences — nur ai/human
- Eigene Cache-Schicht — MD5-Session-Cache von lean-ctx reicht

---

*Status: v0.6 — Sync-Interface entschieden (Q-09). Nur Q-05 noch deferred (gehört in Plan-Execution-Spec). Bereit für
Spike-Start mit Schritt 1+2.*
