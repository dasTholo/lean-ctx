# Runbook: runIde-Editor-Focus-Gate (#500-Producer-Parität, Live-Verifikation)

Verifiziert den JetBrains-Producer für #500 live: das Plugin meldet den
fokussierten Editor-Pfad via `lean-ctx editor-signal --file <path>`, sodass
`~/.lean-ctx/editor_signal.json` und die Dashboard-„Editor focus"-Kachel den
aktiven File spiegeln — 1:1 zum VS-Code-Verhalten.

Bezug: Spec `docs/lean-md/specs/2026-06-13-leanctx-jetbrains-editor-focus-design.md`.

## Voraussetzungen
- `lean-ctx` gebaut/installiert mit `editor-signal`-Subcommand (3.8.3+):
  `lean-ctx editor-signal --help` zeigt `--file`.
- Plugin-Modul: `packages/jetbrains-lean-ctx`.
- Ein Test-Projekt mit mindestens zwei Dateien A und B.

## 1. Launch — Sandbox-IDE
```
./gradlew runIde
```
(cwd=`packages/jetbrains-lean-ctx`)
Test-Projekt öffnen, **Indizierung abwarten** (Statusleiste idle).

## 2. Gate-Checks

| # | Schritt | Soll-Ergebnis |
| 1 | Datei A öffnen, ~2s warten | `~/.lean-ctx/editor_signal.json` → `active_file` endet auf A |
| 2 | Datei B öffnen, ~2s warten | `active_file` → B; A taucht in `recent_files` auf |
| 3 | Dashboard öffnen (`lean-ctx dashboard`), „Editor focus"-Kachel | zeigt B als frisch (innerhalb Freshness-Fenster 120s) |
| 4 | `leanctx.editor.signal.enabled = false` setzen (Sandbox: `Help → Find Action → Registry…`), dann A↔B wechseln, ~2s warten | `editor_signal.json` **ändert sich nicht** (kein neues Signal) |
| 5 | Registry-Key wieder `true`, Binary-Pfad temporär unauffindbar machen (z.B. `lean-ctx` aus PATH/Standardorten entfernen) oder mit fehlendem Binary starten, Tab wechseln | **kein Crash**, IDE bleibt stabil, keine Fehler-Notification-Flut |

`editor_signal.json` inspizieren:
```
cat ~/.lean-ctx/editor_signal.json
```
(Felder: `active_file`, `recent_files[(path, ts)]`, `updated_at`.)

## 3. Teardown
- Sandbox-IDE schließen (Alarm + MessageBus-Connection werden disposed).
- `~/.lean-ctx/editor_signal.json` darf liegen bleiben (globaler Ranking-Hinweis).

## Notizen für die PR-/Merge-Beschreibung
- Beobachtete Werte aus #1–#3 notieren (Beleg der Producer-Parität).
- Bekannte #500-Grenze (Spec §5): mehrere IDE-Fenster → globale Datei,
  last-write-wins (gilt identisch für VS Code, kein JetBrains-Regress).
