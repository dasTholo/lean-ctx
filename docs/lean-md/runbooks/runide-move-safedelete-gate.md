# Runbook: runIde-Move/Safe-Delete-Gate (v2c Live-Verifikation)

Verifiziert den vollen v2c-Two-Phase-Stack live: Rust-Gate (`plan_hash`/TOCTOU,
Konflikt-Gate, **3-Stufen**-PathJail, `INVALID_TARGET`, Cache-Evict) **und** das
JetBrains-Plugin (`MoveFilesOrDirectoriesProcessor`/`SafeDeleteProcessor`-Naht,
Multi-File-Transaktion, ein Undo) gegen ein sauberes Kotlin-Gradle-Fixture.

Bezug: Spec `docs/lean-md/specs/2026-06-10-leanctx-jetbrains-v2c-move-safedelete-design.md` §9.1.

## Voraussetzungen — frisches Binary (Daemon-Stopp ist PFLICHT)

Die neuen Actions (`move_*`/`safe_delete_*`) existieren erst nach Neubau. Ein
**laufender** lean-ctx-Daemon hält den **alten** Action-Satz im Speicher →
`Unknown action`. Reihenfolge **vor** dem Gate:

1. `lean-ctx serve --stop` — Daemon stoppen (gibt Binary frei + entlädt alten Action-Satz).
2. `cargo build` (cwd=`rust`) [+ ggf. Binary neu installieren].
3. `lean-ctx serve --daemon` neu starten **oder** ersten `lean-ctx call` den Daemon auto-starten lassen.

> **Achtung MCP-Session:** In einer aktiven Agent-/MCP-Session ist dieser Daemon
> zugleich der `ctx_*`-Server — `serve --stop` unterbricht die eigenen `ctx_*`-Tools.
> Das Gate als **separaten** Schritt fahren, nicht mitten in einer ctx_*-Aufgabe.

- Plugin-Modul gebaut: `./gradlew buildPlugin` (cwd=`packages/jetbrains-lean-ctx`).

## 1. Setup — Fixture materialisieren
```
./scripts/runide-move-safedelete-gate-setup.sh
```
Notiere `FIX=<abs>/tmp/runide-move-safedelete-gate`.

## 2. Launch — Sandbox-IDE auf dem Fixture
```
./gradlew runIde --args="$FIX"
```
(cwd=`packages/jetbrains-lean-ctx`) — **Indizierung abwarten** (Statusleiste idle).
> Falls `runIde --args` das Projekt nicht öffnet: einmal manuell `File → Open` auf `$FIX`.

## 3. Gate-Checks
Jeder Check: `lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'`.
Für force-/TOCTOU-Fälle zuerst das passende `*_preview` ausführen, um den aktuellen
`plan_hash` zu holen.

| # | Fall | Aufruf (`--json`, Auszug) | Soll-Ergebnis |
| 1 | move Preview (`target_path`) | `{"action":"move_preview","name_path":"Widget","target_path":"src/main/kotlin/app/moved"}` | usages cross-file (Usage.kt), `files≥2`, `plan_hash` gesetzt |
| 2 | move Apply + Undo | `{"action":"move_apply","name_path":"Widget","target_path":"src/main/kotlin/app/moved","plan_hash":"<#1>"}` | `Widget.kt` umgezogen, Refs/Imports in `Usage.kt` angepasst; **ein** Undo (Strg+Z revertet komplett) |
| 3 | move Member (`target_parent`) | `{"action":"move_preview","name_path":"Helper/calc","target_parent":"OtherClass"}` | sprach-abhängig: Member-Move-Plan **oder** `UNSUPPORTED_LANGUAGE` (Kotlin best-effort, vgl. Plan Task 10) — kein Crash |
| 4 | INVALID_TARGET | (a) `{"action":"move_preview","name_path":"Widget"}` (kein Ziel); (b) beide Ziele gesetzt; (c) `{"action":"move_preview","name_path":"Widget","target_path":"../escape"}` | je `INVALID_TARGET`, **vor** Backend-Call, kein Apply |
| 5 | move TOCTOU | eine usage-Stelle in `Usage.kt` zwischen #1 und Apply ändern, dann Apply mit altem `plan_hash` | `CONFLICT` |
| 6 | safe_delete Preview (ungenutzt) | `{"action":"safe_delete_preview","name_path":"Unused"}` | keine blockierenden usages, `plan_hash` gesetzt |
| 7 | safe_delete Apply ohne force (genutzt) | `{"action":"safe_delete_apply","name_path":"Widget","plan_hash":"<preview Widget>"}` | `CONFLICT` mit blockierenden Refs, **kein** Löschen |
| 8 | safe_delete Apply mit force | wie #7 + `"force":true` | gelöscht; Refs bleiben dangling (Rust-Gate hat `force` akzeptiert; IntelliJ-`SafeDeleteProcessor` in IC-2026.1.3 kennt kein `deleteEvenIfUsed` — `run()` löscht immer durch) |
| 9 | INDEXING | Projekt neu öffnen, sofort `move_preview`/`safe_delete_preview` während Indizierung | `INDEXING`, kein Teil-Edit (best-effort beim Mini-Fixture; deterministisch via Rust-Unit abgesichert) |
| 10 | UNSUPPORTED_LANGUAGE | `{"action":"move_preview","path":"notes.txt","line":1,"target_path":"src/main/kotlin/app/moved"}` (`path`+`line`-Fallback, **nicht** `name_path`) | `UNSUPPORTED_LANGUAGE`, kein Crash |
| 11 | BACKEND_REQUIRED | IDE schließen, dann preview **und** apply (move + safe_delete) | `BACKEND_REQUIRED` in beiden Phasen |

> `safe_delete_preview` für `Unused` (#6) liefert den `plan_hash`; für den genutzten
> `Widget` (#7) zuerst `safe_delete_preview name_path=Widget` für dessen `plan_hash`.

## Audit-Ergebnis (Headless-Konflikt, 2026-06-10)

- **rename:** grün — Guard `CapturingProcessor.prepareConflictsDialog` deckt den Modal; keine Änderung.
- **safe_delete:** Fix umgesetzt — direkte PSI-Löschung statt `SafeDeleteProcessor` (kein Modal mehr).
- **move:** Subklassierbarkeit: ja — `MoveFilesOrDirectoriesProcessor` ist eine öffentliche Java-Klasse mit öffentlichem Konstruktor, wird direkt in `SymbolMover.kt:104` instanziiert (nicht final). Test-Modus-Charakterisierung (Step 3/4): CONFLICT, Call kehrt zurück, kein Hang (7/7 Tests grün, `testMoveCollisionReturnsConflictHeadless_characterization` 0.823s). runIde-Live-Provokation (Step 5): **PENDING — manuelle runIde-Reverify durch User ausstehend.** Default bis dahin: KEIN move-Code-Change (Task 3 gated/skipped, YAGNI Spec §4).

## 4. Teardown
- Sandbox-IDE schließen.
- `tmp/runide-move-safedelete-gate/` kann liegen bleiben (gitignored) oder via
  `./scripts/runide-move-safedelete-gate-setup.sh` zurückgesetzt werden.
- Daemon wieder hochfahren, falls für die MCP-Session gestoppt.
