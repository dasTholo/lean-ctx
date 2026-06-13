# Runbook: runRustRover-Cross-IDE-Gate (sprach-neutrales Plugin, Live-Verifikation)

Verifiziert **live gegen die echte RustRover-IDE**, dass das JetBrains-Plugin nach der
Drei-Tier-Umstellung in einer Nicht-JVM-IDE lädt (kein `java-capable`-Fehler) und alle
sprach-neutralen Features für Rust funktionieren — `type_hierarchy` sauber degradiert.

> Die Verifikation **muss** gegen die `runRustRover`-Sandbox laufen, **nicht** gegen
> IDEA. Das IDEA-`runIde` würde den `java-capable`-Block nie reproduzieren und damit
> nichts beweisen.

Bezug: Spec `docs/lean-md/specs/2026-06-13-jetbrains-language-neutral-psi-design.md` §5.2.

## Voraussetzungen — frisches Binary (Daemon-Stopp ist PFLICHT)

1. `lean-ctx serve --stop` — Daemon stoppen (gibt Binary frei + entlädt alten Action-Satz).
2. `cargo build` (cwd=`rust`) [+ ggf. Binary neu installieren].
3. `lean-ctx serve --daemon` neu starten **oder** ersten `lean-ctx call` den Daemon
   auto-starten lassen.

> **Achtung MCP-Session:** In einer aktiven Agent-/MCP-Session ist dieser Daemon
> zugleich der `ctx_*`-Server — `serve --stop` unterbricht die eigenen `ctx_*`-Tools.
> Das Gate als **separaten** Schritt fahren, nicht mitten in einer ctx_*-Aufgabe.

- Plugin-Modul gebaut: `./gradlew buildPlugin` (cwd=`packages/jetbrains-lean-ctx`).

## 1. Setup — Cargo-Fixture materialisieren

```bash
bash scripts/runrustrover-cross-ide-gate-setup.sh
```

Notiere `FIX=<abs>/tmp/runrustrover-cross-ide-gate`.

## 2. Launch — RustRover-Sandbox auf dem Fixture

```bash
FIX="$(pwd)/tmp/runrustrover-cross-ide-gate"
./gradlew runRustRover --args="$FIX"
```

(cwd=`packages/jetbrains-lean-ctx`) — **Indizierung abwarten** (Cargo-Projekt erkannt,
Statusleiste idle).

> Falls `runRustRover --args` das Projekt nicht öffnet: einmal manuell `File → Open` auf
> `$FIX`. Alternativ eine lokal installierte RustRover via `local(file("<pfad>"))` in
> `build.gradle.kts` statt `version = "2026.1"`.

## 3. Gate-Checks

Jeder Check (sofern nicht anders notiert):
`lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'`.

| #  | Fall                                       | Soll-Ergebnis                                                                                              |
|----|--------------------------------------------|-----------------------------------------------------------------------------------------------------------|
| 0  | **Plugin lädt** in RustRover               | KEIN „java-capable"-Fehler; Port-File geschrieben; `GET /health` ok                                        |
| 1  | UI sichtbar                                | Status-Bar `⚡ lean-ctx`, Tools-Menü, Gain-Tool-Window vorhanden                                            |
| 2  | `references` auf `area` (Rust)             | Usages über Impls/Call-Sites gefunden                                                                      |
| 3  | `definition` / `declaration` (Rust)        | korrekte Zielposition                                                                                      |
| 4  | `implementations` auf `trait Shape`        | `Circle`+`Square` als Impls (Trait→Impl-„Hierarchie")                                                      |
| 5  | `rename` (Two-Phase, Rust-Symbol)          | Preview liefert Usages; Apply benennt projektweit um (eine Transaktion)                                    |
| 6  | `reformat` der Datei `src/messy.rs`        | korrekt formatiert (CodeStyleManager, Rust)                                                                |
| 7  | `inspections mode=run` (Rust-Datei)        | Diagnostics geliefert oder sauber leer (kein Crash)                                                        |
| 8  | `symbols_overview` (Rust)                  | via lean-ctx tree-sitter — Top-Level-Symbole, kein IDE-PSI nötig                                           |
| 9  | `type_hierarchy` (Rust)                    | **sauber degradiert** (`UNSUPPORTED_LANGUAGE` bzw. Routing auf `implementations`/`ctx_callgraph`, kein Crash) |
| 10 | `ctx_callgraph` callers/callees (Rust-Fn)  | Call-Hierarchie für `total_area` geliefert (lean-ctx-Pfad)                                                 |
| 11 | Editor-Signal                              | Fokuswechsel auf Rust-Datei → `editor-signal` emittiert (Pfad-only)                                        |

### Durchlauf <DATUM> — Ergebnis

RustRover-<VERSION>-Sandbox, Fixture wie oben. Befund je Check (✅/⏭️):

| #  | Ergebnis | Notiz |
|----|----------|-------|
| 0  |          |       |
| 1  |          |       |
| …  |          |       |
| 11 |          |       |

## 4. Teardown

- Sandbox-IDE schließen.
- `tmp/runrustrover-cross-ide-gate/` kann liegen bleiben (gitignored) oder via
  `bash scripts/runrustrover-cross-ide-gate-setup.sh` zurückgesetzt werden.
- Daemon wieder hochfahren, falls für die MCP-Session gestoppt.
