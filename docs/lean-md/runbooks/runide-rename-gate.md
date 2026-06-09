# Runbook: runIde-Rename-Gate (v2b Live-Verifikation)

Verifiziert den vollen v2b-Two-Phase-Rename-Stack live: Rust-Gate
(`plan_hash`/TOCTOU, Konflikt, PathJail, Cache-Evict) **und** das JetBrains-Plugin
(`RenameProcessor`-Naht, Multi-File-Transaktion) gegen ein sauberes
Kotlin-Gradle-Fixture mit korrektem Find-Usages-Scope.

Bezug: Spec `docs/lean-md/specs/2026-06-09-leanctx-jetbrains-runide-rename-gate-harness-design.md`.

## Voraussetzungen
- `lean-ctx` gebaut: `cargo build` (cwd=`rust`) oder installierte Binary.
- Plugin-Modul: `packages/jetbrains-lean-ctx`.

## 1. Setup — Fixture materialisieren
```
./scripts/runide-gate-setup.sh
```
Notiere den absoluten Fixture-Pfad: `FIX=<abs>/tmp/runide-rename-gate`.

## 2. Launch — Sandbox-IDE auf dem Fixture
```
./gradlew runIde --args="$FIX"
```
(cwd=`packages/jetbrains-lean-ctx`)

Sandbox-IDE öffnet das Fixture. **Indizierung abwarten** (Statusleiste idle).
> Falls `runIde --args` das Projekt nicht zuverlässig öffnet (siehe Spec §8.1):
> einmal manuell `File → Open` auf `$FIX` — die Sandbox persistiert es für
> Folge-Läufe.

## 3. Gate-Checks
Jeder Check: `lean-ctx call ctx_refactor --project-root "$FIX" --json '<args>'`.
Notiere das beobachtete Ergebnis (für die v2b-PR-/Merge-Beschreibung).

| # | Fall | Aufruf (`--json`) | Soll-Ergebnis |
| 1 | Preview cross-file | `{"action":"rename_preview","name_path":"Widget","new_name":"Renamed"}` | usages über `Widget.kt`+`Usage.kt`, `files: 2`, `plan_hash` gesetzt |
| 2 | Apply + Undo | `{"action":"rename_apply","name_path":"Widget","new_name":"Renamed","plan_hash":"<aus #1>"}` | alle Stellen umbenannt; in IDE **ein** Undo-Eintrag (Strg+Z revertet komplett). Danach Strg+Z → Fixture reset |
| 3 | TOCTOU | eine **usage-Stelle** in `Usage.kt` zwischen #1 und apply ändern (z.B. eine Leerzeile davor einfügen → usage-range verschiebt sich), dann #2 mit altem `plan_hash` | `CONFLICT`. Hinweis: `plan_hash` deckt nur usage-Stellen ab (`path\|range\|text`), **nicht** die Deklarations-Datei — eine Änderung an `Widget.kt` allein triggert **kein** `CONFLICT`. |
| 4a | Konflikt ohne force | `{"action":"rename_apply","name_path":"Widget","new_name":"Gadget","plan_hash":"<preview Gadget>"}` | `CONFLICT` (Kollision mit `Gadget.kt`) Nach Fix A headless als CONFLICT-**Token** erwartet, **kein** modaler Dialog. |
| 4b | Konflikt mit force | wie 4a + `"force":true` | durchgereicht/angewandt |
| 5 | INDEXING | Projekt neu öffnen, sofort `rename_preview` während Indizierung | `INDEXING`, kein Teil-Rename Manuell/best-effort (kurzes Re-Index-Fenster beim Mini-Fixture); deterministisch durch Unit-Test (Dumb-Mode → INDEXING) abgesichert. |
| 6 | UNSUPPORTED_LANGUAGE | `{"action":"rename_preview","name_path":"notes","new_name":"x"}` (Ziel in `notes.txt`) | `UNSUPPORTED_LANGUAGE`, kein Crash Über den `path:"notes.txt"`+`line:1`-Fallback (Rust `resolve_rename_target`, `ctx_refactor.rs:354-367`); nach Fix B kommt `UNSUPPORTED_LANGUAGE` zuverlässig vor `NO_SYMBOL`. |
| 7 | BACKEND_REQUIRED | IDE schließen, dann preview **und** apply | `BACKEND_REQUIRED` in beiden Phasen |

> Für Fall 4 zuerst ein eigenes `rename_preview` mit `new_name=Gadget` ausführen,
> um dessen `plan_hash` zu erhalten.

## 4. Teardown
- Sandbox-IDE schließen.
- `tmp/runide-rename-gate/` kann liegen bleiben (gitignored) oder via
  `./scripts/runide-gate-setup.sh` zurückgesetzt werden.
