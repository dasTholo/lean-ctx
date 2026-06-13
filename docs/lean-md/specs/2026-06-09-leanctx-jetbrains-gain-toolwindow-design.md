# lean-ctx JetBrains — Gain ToolWindow (Design-Spec)

- **Datum:** 2026-06-09 (überarbeitet 2026-06-13)
- **Branch:** feat-jetbrains-plugin
- **Status:** Design abgenommen, bereit für Implementierungsplan
- **Scope:** Neues andockbares ToolWindow im JetBrains-Plugin, das die reichen
  `lean-ctx gain`-Metriken anzeigt. Die bestehende StatusBar-Anzeige bleibt
  unverändert und dient zusätzlich als Trigger.

> **Revision 2026-06-13** (nach v2b/v2c/v2d): Datenpfad (Subprozess) gegen das
> inzwischen gereifte HTTP-Backend re-evaluiert und **bestätigt** — die
> Richtungs-Asymmetrie schließt HTTP aus (§2). Spec an die gewachsene
> Plugin-Struktur angeglichen: concern-basierte Subpackages (`dto/`,
> `toolwindow/`) statt flachem Package (§4, §10), `@SerializedName` zwingend
> wegen snake_case-Payload, bestehende `GainAction` umgebaut statt neuer Action
> (§6), eigenes 10-s-Timeout (§7), Drift-Test verbindlich (§8).

## 1. Ziel & Kontext

Die StatusBar zeigt heute nur eine Roh-Kennzahl (`⚡ … saved`) aus einem lokalen
Datei-Read von `~/.lean-ctx/stats.json` (`StatsReader`). Beim Klick auf die
Anzeige soll sich ein eigenes Fenster in der IDE öffnen, das deutlich mehr
Kontext liefert: Gain Score, Trend, Sub-Scores sowie Task- und Heatmap-Tabellen.

Der **Buddy** (Gamification-Feature) wird bewusst ausgenommen — er ist im
`gain --json`-Payload ohnehin nicht enthalten, daher ist kein Ausschluss-Code
nötig.

## 2. Architektur-Entscheidung: Datenpfad

**Direkte Kommunikation per Subprozess — kein HTTP.**

`BinaryResolver.runCommand("gain", "--json")` (existiert bereits,
`BinaryResolver.kt:41`) spawnt das `lean-ctx`-Binary, captured stdout/stderr
getrennt, 30 s Timeout, setzt `NO_COLOR=1` + `LEAN_CTX_ACTIVE=0`. Das Ergebnis
(stdout) wird mit **gson** (bereits Dependency) in DTOs geparst. (§7 ergänzt für
das ToolWindow ein kürzeres 10-s-Timeout via Overload.)

### Warum nicht das vorhandene HTTP-Backend? (Richtungs-Asymmetrie)

**Primärgrund — geprüft am Code (Revision 2026-06-13):** Das seit v2b/v2c/v2d
gereifte Backend fließt in der **falschen Richtung**.

- Bestehendes Backend (`lsp/backend.rs`, `lsp/jetbrains_backend.rs:52`):
  **Plugin = HTTP-Server** (`server/BackendHttpServer.kt` lauscht, schreibt
  Port-File), **Rust = Client** (`JetBrainsHttpBackend` liest Port-File →
  `http://127.0.0.1:{port}`). Zweck: **Rust fragt die laufende IDE** nach PSI
  (nav/edit/rename/move/inline/reformat).
- Gain-ToolWindow braucht die **Gegenrichtung**: Plugin = Konsument,
  Rust (`core/gain/mod.rs` `GainScore`) = Produzent.

∴ Das gereifte Backend ist **nicht wiederverwendbar** — es ist
Plugin-als-Server. HTTP-Gain erforderte einen **neuen Rust-HTTP-Server**, den
das Plugin abfragt, inkl. Port/Token/Lifecycle/Health — exakt der Overhead, den
dieses Spec verwirft. Es gibt keinen persistenten Rust-Server zum Abfragen. Der
Subprozess ist richtungs-korrekt (Plugin spawnt, Rust antwortet, Prozess endet).

### Subprozess `gain --json` — Vor- & Nachteile

**Vorteile**

- **Single Source of Truth:** `GainScore`/`TaskClassifier`/`ModelPricing` bleiben
  nur in Rust — kein Kotlin-Nachbau, kein Drift.
- **Kein neuer Server:** kein Port/Token/Lifecycle/Health; `runCommand` existiert
  und ist battle-tested (trägt schon `setup`/`doctor`/`gain`).
- **Richtungs-korrekt:** Plugin = Konsument, Rust = Produzent (siehe oben).
- **Sauber isoliert:** `LEAN_CTX_ACTIVE=0` + `NO_COLOR=1` → der Abruf verfälscht
  die Stats nicht und liefert reines JSON.
- **Vollständig in einem Aufruf:** Score+Sub-Scores+Trend+Heatmap; ein
  Datei-Read bräuchte ≥3 Dateien + Kotlin-Logik (siehe Tabelle unten).

**Nachteile (+ Mitigation)**

- **Spawn-Kosten** ~10–50 ms pro Poll (30 s, nur bei Sichtbarkeit) — unkritisch.
- **Kein Push/Live:** reines Pull; <30-s-Live bräuchte IPC — für ein
  on-demand-Fenster irrelevant, bewusst akzeptiert.
- **Kopplung an CLI-Schema:** `gain --json` ist de-facto API → **Drift-Test
  verbindlich** (§8), sonst bricht das Plugin still bei Schemaänderung.
- **Timeout-Latenz:** hängendes Binary → spätes Feedback → **10-s-Timeout** fürs
  ToolWindow statt 30 s (§7).
- **Binary-Abhängigkeit:** kein `lean-ctx` im PATH → `BinaryNotFound`-Zustand
  (§7) fängt es ab.
- **Cold-Start:** `resolve()` probiert 5 Pfade je 5 s, danach gecacht — erster
  Aufruf potenziell träge.

### Warum nicht direkter Datei-Read?

Die gewählten Sektionen lassen sich **nicht** vollständig aus `stats.json`
ableiten:

| Sektion                                      | Quelle                                                                           |
|----------------------------------------------|----------------------------------------------------------------------------------|
| Hero `tokens_saved` / `gain_rate_pct`        | `stats.json` (ableitbar)                                                         |
| Task-Spalte `tokens_saved`                   | `stats.commands` (in `stats.json`)                                               |
| **Score + 4 Sub-Scores + Trend**             | `GainScore::compute(stats, costs, pricing)` — Rust-Logik (`core/gain/mod.rs:91`) |
| Task-Spalten `tool_calls` / `tool_spend_usd` | `cost_attribution.json` (`mod.rs:142-153`)                                       |
| **Heatmap-Tabelle**                          | separate Datei `heatmap.json` (`mod.rs:160-173`)                                 |

Ein Datei-Read-Ansatz erforderte ≥3 Dateien plus in Kotlin nachgebaute
`GainScore`/`TaskClassifier`/`ModelPricing`-Logik → **Drift-Risiko** gegen die
Rust-Implementierung. Der Subprozess hält Rust als **Single Source of Truth**;
Kotlin rendert nur. Kosten: Prozess-Spawn ~10–50 ms, on-demand unkritisch.

### Verworfene Alternativen

- **HTTP-Endpoint** (`jetbrains_backend.rs`): siehe „Richtungs-Asymmetrie" oben —
  das vorhandene Backend ist Plugin-als-Server; ein Stats-Pfad bräuchte einen
  neuen Rust-HTTP-Server mit Port/Token/Server-Overhead ohne Mehrwert.
- **Echtes IPC** (Unix-Socket/Named Pipe/Daemon): Overkill für ein on-demand
  per Klick geöffnetes Fenster.
- **Hybrid** (Hero aus Datei, Rest aus Subprozess): mehr Code, feinere
  Kostenkontrolle — verworfen zugunsten von Einfachheit/Konsistenz.

## 3. UI-Aufbau

**ToolWindow + native Swing.** Native Komponenten themen automatisch mit der IDE
(Darcula/Light) und sind leichtgewichtig. Registrierung über neuen
`<toolWindow>`-Eintrag in `plugin.xml`, `anchor="bottom"`. ToolWindows sind frei
verschiebbar (bottom/left/right/float); IntelliJ merkt sich die Position
projektweise — `anchor` ist nur die Startposition.

### Layout — Variante B (vertikal gestapelt, scrollend)

Robust für jede Andock-Seite (auch schmal rechts oder als Float-Window):

```
┌ lean-ctx Gain ───────────────────  ● live  ⟳ ┐
│ 68  GAIN SCORE                     ▲ Rising   │
│ Saved 7.6M · Rate 68.6% · $19.02              │
│ Compression  ▆▆▆▆▆▆▆░░░  69                    │
│ Cost-Eff.    ▏          3                      │
│ Quality      ▆▆▆▆▆▆▆▆░░  76                    │
│ Consistency  ▆▆▆░░░░░░░  29                    │
│ TASKS NACH KATEGORIE                           │
│ Kategorie     Cmds  Saved  Calls   $           │
│ Exploration   2281  6.3M   4352   43.9         │
│ …                                              │
│ HEATMAP · TOP-DATEIEN                           │
│ Datei        Zugr.  Saved   %                  │
│ backend.rs   3      518K   99.8                │
│ …                                              │
├────────────────────────────────────────────────┤
│ Modell: fallback-blended · aktualisiert vor 4 s │
└────────────────────────────────────────────────┘
```

### Farbstil — Stil 1 „Ruhig"

- Gain Score in Grün.
- Alle 4 Sub-Score-Balken in einheitlichem Blau (keine Ampel-Farbkodierung).
- Trend als Pfeil + Wort (▲ Rising / → Stable / ▼ Declining).
- Dezenter Footer: Modellname + „aktualisiert vor X s".
- `live`-Indikator im Header signalisiert aktiven Poll.

### Sektionen (gewählt)

1. **Hero:** Gain Score (0–100), Trend, `tokens_saved`, `gain_rate_pct`,
   `avoided_usd`.
2. **Sub-Scores:** 4 `JProgressBar` — `compression`, `cost_efficiency`,
   `quality`, `consistency`.
3. **Tabellen (`JBTable`):**
    - Tasks: Kategorie, commands, tokens_saved, tool_calls, tool_spend_usd.
    - Heatmap: path, access_count, tokens_saved, compression_pct.

**Nicht enthalten:** Impact-Kennzahlen (Energy/CO2/ROI/tool_spend_usd-Gesamt) —
werden auch **nicht** ins DTO geparst (gson ignoriert sie).

## 4. Komponenten (Kotlin)

**Package-Layout — concern-basiert** (angeglichen an die gewachsene Struktur:
`dto/`, `psi/`, `endpoint/`, `server/`, `actions/`; flach liegen nur
Infra-Singletons). DTOs → `dto/`, UI+Service → neues `toolwindow/`:

```
com/leanctx/plugin/
  dto/
    Wire.kt                          (bestehend, unverändert)
    GainData.kt                      ← neu
  toolwindow/                        ← neu
    GainService.kt
    LeanCtxGainToolWindowFactory.kt
    GainPanel.kt
  actions/
    LeanCtxActions.kt                ~ GainAction umgebaut (§6)
```

| Komponente                                   | Verantwortung                                                                                                                                                                                                                                                                                                                                                                               |
|----------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `dto/GainData.kt`                            | gson-DTOs: `Summary { tokensSaved, gainRatePct, avoidedUsd, score }`, `Score { total, compression, costEfficiency, quality, consistency, trend }`, `TaskRow`, `FileRow`. Nur gerenderte Felder. **`@SerializedName` zwingend** — Payload ist snake_case (`tokens_saved`, `gain_rate_pct`, `cost_efficiency`); die Wire-DTOs sind dagegen Einzelwörter, daher kein Präzedenzfall im Bestand. |
| `toolwindow/GainService.kt`                  | `fun load(): Result<GainData>` — ruft `BinaryResolver.runCommand(10, "gain","--json")` (10-s-Overload, §7), parst mit eigenem Gson (Idiom `disableHtmlEscaping()` aus `dto/Wire.kt`-`JsonCodec` übernommen; **kein** Eingriff in `Wire.kt`, da Gain ≠ Wire-Protokoll). **Läuft nie auf dem EDT.** Liefert typisierte Fehlerzustände.                                                        |
| `toolwindow/LeanCtxGainToolWindowFactory.kt` | `ToolWindowFactory`; baut `GainPanel` als Content, registriert Disposable.                                                                                                                                                                                                                                                                                                                  |
| `toolwindow/GainPanel.kt`                    | `SimpleToolWindowPanel`; rendert die 3 Sektionen + Toolbar (Refresh) + Footer; hält den Poll-Timer; kapselt die Zustands-Panels.                                                                                                                                                                                                                                                            |

## 5. Polling — sichtbarkeits-gekoppelt

- Timer-Intervall **30 s** (konsistent zur StatusBar).
- Poll läuft **nur, wenn das ToolWindow tatsächlich sichtbar ist** — Gate über
  `ToolWindowManagerListener.stateChanged` + `toolWindow.isVisible`. Nicht nur
  „existiert".
- **Sichtbar werden → sofort einmal laden**, danach 30-s-Takt (keine 30 s
  Anfangsverzögerung wie bei der StatusBar).
- Versteckt / abgekoppelt / Tab-Wechsel → Timer **stoppt sofort**, kein
  Subprozess-Spawn.
- Zusätzlich **manueller Refresh-Button** in der Toolbar.
- `Disposable` an den ToolWindow-Content gebunden → Timer-Cleanup bei
  Schließen/Projekt-Close.

## 6. Trigger

1. **StatusBar-Klick:** `LeanCtxStatusBarWidget` implementiert zusätzlich
   `getClickConsumer()` →
   `ToolWindowManager.getInstance(project).getToolWindow("LeanCtxGain")?.activate(null)`.
   (Aktuell hat das Widget keinen Klick-Handler.)
2. **Menü-Action — bestehende `GainAction` umbauen, keine neue anlegen:**
   `actions/LeanCtxActions.kt` hat heute schon
   `class GainAction : LeanCtxCommandAction("gain")` (läuft `lean-ctx gain`, zeigt
   stdout in einem `Messages.showInfoMessage`-Popup) und ist in `plugin.xml` als
   Tools→lean-ctx→**„Gain Report"** verdrahtet. `GainAction` wird umgebaut: statt
   Text-Popup aktiviert sie das ToolWindow
   (`ToolWindowManager…getToolWindow("LeanCtxGain")?.activate(null)`). Damit
   bleibt der Menüeintrag erhalten, ohne Parallel-Action. (Erbt dann nicht mehr
   von `LeanCtxCommandAction`, sondern direkt von `AnAction` — wie
   `DashboardAction`.)

Die StatusBar-Anzeige selbst bleibt beim billigen lokalen `StatsReader` —
unverändert.

## 7. Threading & Fehlerbehandlung

- Subprozess **strikt off-EDT** (z.B. `executeOnPooledThread`); UI-Update via
  `invokeLater` auf dem EDT. Ein blockierender Aufruf auf dem EDT würde die IDE
  einfrieren.
- **Eigenes 10-s-Timeout fürs ToolWindow** (statt 30 s der StatusBar): schnelleres
  Fehler-Feedback bei hängendem Binary. `BinaryResolver.runCommand` hat 30 s
  **hartcodiert** (`BinaryResolver.kt:54`, `waitFor(30, …)`) → Overload
  `runCommand(timeoutSeconds: Long, vararg args: String)` ergänzen; die alte
  Signatur delegiert mit 30 s. Hält `BinaryResolver` als **einzigen** Spawn-Pfad
  (kein separater `ProcessBuilder` im Service).
- Vier Nicht-Daten-Zustände im Panel (kein Crash):
    1. **Laden** — Spinner + „Lade Gain-Daten…".
    2. **Binary nicht gefunden** (`BinaryNotFound`) — Hinweis auf `lean-ctx setup`
       / PATH.
    3. **Befehl fehlgeschlagen / Timeout** (`CommandFailed` / `Timeout` /
       `ParseError`) — Fehlermeldung + stderr-Auszug, Button „Erneut".
    4. **Leer** (`Empty`, 0 commands) — „Noch keine Daten erfasst".

## 8. Tests

- **Kotlin-Unit:** gson-Parsing gegen Fixture-JSON (Sample-`gain --json`) → DTOs
  korrekt; Fehlerpfade (leeres/kaputtes JSON, fehlende Felder).
- **Sichtbarkeits-Gate:** Timer startet/stoppt korrekt an `isVisible`-Übergängen
  (Logik von der UI entkoppelt testbar).
- **Rust — Drift-Test verbindlich (Mitigation, nicht optional):** Da
  `gain --json` damit zur de-facto API des Plugins wird, **muss** ein Test das
  Schema gegen die Kotlin-DTOs absichern (Schema-Stabilitäts-Fixture). Variante:
  Rust-Test serialisiert eine `GainScore`-/Summary-Probe nach JSON und
  assertiert die exakten snake_case-Keys, die `dto/GainData.kt` via
  `@SerializedName` erwartet — bricht das Schema, bricht der Test (statt das
  Plugin still). Beim Schreiben des Plans prüfen, ob ein solcher Test bereits
  existiert; falls nicht, neu anlegen.
- **Manuell:** `runIde`-Sandbox — Klick → Fenster öffnet/fokussiert; Poll nur bei
  Sichtbarkeit (per Prozess-Beobachtung verifizieren); Refresh-Button; alle vier
  Zustände provozieren.

## 9. `gain --json` Payload (Referenz)

```json
{
  "summary": {
    "tokens_saved": 7608645,
    "gain_rate_pct": 68.57,
    "avoided_usd": 19.02,
    "score": {
      "total": 68,
      "compression": 69,
      "cost_efficiency": 3,
      "quality": 76,
      "consistency": 29,
      "trend": "Rising"
    }
  },
  "tasks": [
    {
      "category": "Exploration",
      "commands": 2281,
      "tokens_saved": 6337663,
      "tool_calls": 4352,
      "tool_spend_usd": 43.92
    }
  ],
  "heatmap": [
    {
      "path": "…/backend.rs",
      "access_count": 3,
      "tokens_saved": 518625,
      "compression_pct": 99.84
    }
  ]
}
```

Weitere Felder (`model`, `energy_wh`, `co2_grams`, `roi`, …) existieren im
Payload, werden aber bewusst **nicht** geparst/gerendert.

## 10. Betroffene Dateien (Erwartung)

- **Neu:** `dto/GainData.kt`, `toolwindow/GainService.kt`,
  `toolwindow/LeanCtxGainToolWindowFactory.kt`, `toolwindow/GainPanel.kt`,
  Test-Fixtures (Sample-`gain --json`) + Rust-Schema-Drift-Test.
- **Geändert:**
    - `plugin.xml`: `<toolWindow id="LeanCtxGain" …>` ergänzen; **kein** neuer
      `<action>` — der bestehende `LeanCtx.Gain`-Eintrag bleibt, nur die Klasse
      verhält sich neu.
    - `actions/LeanCtxActions.kt`: `GainAction` umgebaut (Text-Popup → ToolWindow
      aktivieren; erbt von `AnAction` statt `LeanCtxCommandAction`).
    - `BinaryResolver.kt`: `runCommand`-Overload mit `timeoutSeconds` (§7).
    - `LeanCtxStatusBarFactory.kt`: `getClickConsumer` (§6).
- **Unverändert:** `StatsReader.kt`, `dto/Wire.kt` (kein Eingriff), HTTP-Backend
  (`jetbrains_backend.rs` & `server/`-Klassen).
