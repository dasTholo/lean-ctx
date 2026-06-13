# lean-ctx JetBrains — Gain ToolWindow (Design-Spec)

- **Datum:** 2026-06-09
- **Branch:** feat-jetbrains-plugin
- **Status:** Design abgenommen, bereit für Implementierungsplan
- **Scope:** Neues andockbares ToolWindow im JetBrains-Plugin, das die reichen
  `lean-ctx gain`-Metriken anzeigt. Die bestehende StatusBar-Anzeige bleibt
  unverändert und dient zusätzlich als Trigger.

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
`BinaryResolver.kt:42`) spawnt das `lean-ctx`-Binary, captured stdout/stderr
getrennt, 30 s Timeout, setzt `NO_COLOR=1` + `LEAN_CTX_ACTIVE=0`. Das Ergebnis
(stdout) wird mit **gson** (bereits Dependency) in DTOs geparst.

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

- **HTTP-Endpoint** (`jetbrains_backend.rs`): Das vorhandene Backend ist für
  Symbol-Navigation/Edits (Plugin = Server, Rust = Client). Ein neuer Stats-
  Endpoint brächte Port/Token/Server-Overhead ohne Mehrwert.
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

## 4. Komponenten (Kotlin, Package `com.leanctx.plugin`)

| Komponente                     | Verantwortung                                                                                                                                                                                                                                   |
|--------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `GainData.kt`                  | gson-DTOs: `Summary { tokensSaved, gainRatePct, avoidedUsd, score }`, `Score { total, compression, costEfficiency, quality, consistency, trend }`, `TaskRow`, `FileRow`. Nur gerenderte Felder; `@SerializedName` mappt snake_case → camelCase. |
| `GainService.kt`               | `fun load(): Result<GainData>` — ruft `BinaryResolver.runCommand("gain","--json")`, parst via gson. **Läuft nie auf dem EDT.** Liefert typisierte Fehlerzustände.                                                                               |
| `LeanCtxGainToolWindowFactory` | `ToolWindowFactory`; baut `GainPanel` als Content, registriert Disposable.                                                                                                                                                                      |
| `GainPanel`                    | `SimpleToolWindowPanel`; rendert die 3 Sektionen + Toolbar (Refresh) + Footer; hält den Poll-Timer; kapselt die Zustands-Panels.                                                                                                                |

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
2. **Menü-Action:** zusätzliche `AnAction` (Tools-Menü / Search-Everywhere),
   damit das Fenster auch ohne StatusBar erreichbar ist.

Die StatusBar-Anzeige selbst bleibt beim billigen lokalen `StatsReader` —
unverändert.

## 7. Threading & Fehlerbehandlung

- Subprozess **strikt off-EDT** (z.B. `executeOnPooledThread`); UI-Update via
  `invokeLater` auf dem EDT. Ein blockierender Aufruf auf dem EDT würde die IDE
  einfrieren.
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
- **Rust:** prüfen, ob ein Drift-Test das `gain --json`-Schema gegen die DTOs
  absichert; falls nicht, ein Schema-Stabilitäts-Fixture ergänzen.
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

- **Neu:** `GainData.kt`, `GainService.kt`, `LeanCtxGainToolWindowFactory.kt`,
  `GainPanel.kt`, Menü-`AnAction`, Test-Fixtures.
- **Geändert:** `plugin.xml` (`<toolWindow>` + Action-Registrierung),
  `LeanCtxStatusBarFactory.kt` (`getClickConsumer`).
- **Unverändert:** `StatsReader.kt`, HTTP-Backend (`jetbrains_backend.rs` &
  `server/`-Klassen).

```
