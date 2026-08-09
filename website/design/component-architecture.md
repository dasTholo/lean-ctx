# lean-ctx website redesign — component architecture

## Purpose and scope

This is the implementation blueprint for the English-first redesign. The home page is rebuilt around the narrative: **problem → solution → proof → trust → install**. Marketing pages share the same conversion system; documentation retains its information architecture but adopts a getting-started-first chrome.

The current directory contains **33** Astro components, rather than the 34 stated in the brief. The inventory below covers every current `.astro` component, including `architecture/`.

### Architecture rules

- Astro remains the rendering layer; interactive controls use small, progressively enhanced browser scripts, not a framework island.
- `src/data/*.json` is the source of truth for public marketing facts. No public numeric claim is hard-coded in a component.
- `src/i18n/locales/en.json` owns translatable prose; the four proposed JSON files own structured product data. Add locale overlays only after English content and schema stabilize.
- Use `TerminalPanel` for every code, CLI, or output display. Do not create terminal variants in page CSS.
- Use `SectionContainer` for section spacing, heading hierarchy, and landmarks. A page may add only page-specific structural layout inside its default slot.
- Use emerald for interaction and status; indigo/purple remain data-visualization-only, consistent with the existing global tokens.
- Every interactive component must work with JavaScript disabled where a meaningful static fallback is possible. The demo defaults to the `map` view; install defaults to Cursor.

---

## 1. Component inventory — current to new

The disposition is for the redesigned site, not an instruction to delete a component on day one. “Remove” means remove it from the new composition and retire it after the old route/template has been replaced or redirected.

| Current component | Decision | Redesign destination / reasoning |
| --- | --- | --- |
| `ArchitectureDiagram.astro` | MODIFY | Keep as a technical deep-dive on How it works/docs architecture, but reduce it to the four bounded planes and make labels/data claim-backed. It is too detailed for the primary home proof. |
| `AsciiHeroBg.astro` | REPLACE | Its large static ASCII scenes compete with the new editorial hero. Replace the home use with `hero/HeroBackground.astro`; retain no new references. |
| `AudienceGrid.astro` | REMOVE | Buyer segmentation is not part of the one-idea-per-scroll landing arc. Move useful copy to `/about` or relevant docs/use-case content. |
| `AudienceTriad.astro` | REMOVE | It is a wrapper around the audience grid and creates a duplicate home section. Retire with `AudienceGrid`. |
| `BenchmarkScatterChart.astro` | MODIFY | Keep for benchmark methodology/docs and pricing evidence, sourced from a versioned benchmark dataset and accompanied by `BasisLabel`. It is not a homepage hero artifact. |
| `BuildDoors.astro` | REMOVE | Its broad product doorway pattern conflicts with the single install action. Route its useful links to docs/navigation. |
| `CanonicalDefinition.astro` | KEEP | Preserve the canonical, shared definition for SEO, about, and docs entry points; its wording is already governed by `lib/positioning.ts`. |
| `CompressionTable.astro` | REPLACE | A static comparison table does not satisfy the inspectable product-proof requirement. `ContextDemo` supplies exact mode output, retained information, and an expand path. |
| `CtaSection.astro` | MODIFY | Retain for secondary long-form page endings only. Add an `agentAware` action option so the primary action renders “Install for [agent]”; never use it as a competing CTA on the home viewport. |
| `DocsCallout.astro` | KEEP | It is a clear, accessible semantic aside for documentation. Continue using its note/warn/danger API. |
| `DocsFooter.astro` | MODIFY | Keep a docs-specific compact footer, but reduce links and add a contextual quickstart/install path. It should not reproduce the legacy dense resource map. |
| `DocsSidebar.astro` | MODIFY | Preserve the navigation behavior and `docsTree` data source, but order it around Quickstart, first read, configuration, then reference. Keep desktop sticky/mobile disclosure behavior. |
| `DocsTopBar.astro` | MODIFY | Keep the docs-specific top bar with search and docs navigation; align its logo, search, theme toggle, and install link with `HeaderV2`. |
| `FaqSection.astro` | MODIFY | Keep native `<details>` disclosure and FAQPage schema contract. Change heading markup to use `SectionContainer`/`EyebrowLabel` and receive structured entries from `faq.json`. |
| `Footer.astro` | REPLACE | The present five-column, many-link footer violates the two-column destination. Replace with `layout/FooterV2.astro`. |
| `Header.astro` | REPLACE | The current header imports `MegaDropdown`, `SearchModal`, and `LanguageSwitcher` and exposes a large navigation surface. Replace with five direct destinations in `layout/HeaderV2.astro`. |
| `HowItWorksSteps.astro` | REPLACE | Its five-pillar presentation does not match the new causal three-step loop (read → search/execute → remember/share). Replace with `sections/WorkflowLoop.astro` and `WorkflowStep.astro`. |
| `LanguageSwitcher.astro` | MODIFY | Keep the locale-availability logic but move the selector out of the marketing header into `FooterV2`; expose it in the docs chrome only when needed. |
| `LiveCompressionDemo.astro` | REPLACE | The existing demo uses embedded sample data and different modes. Replace it with `ContextDemo`, backed by a versioned fixture and claims registry, using `full/map/signatures`. |
| `MegaDropdown.astro` | REMOVE | The product, use-case, and resource mega navigation is intentionally removed from primary navigation. Preserve destination routes in docs/footer/search. |
| `PageHero.astro` | MODIFY | Keep as the constrained inner-page hero for How it works, Pricing, Enterprise, and legacy pages. Remove its broad visual knobs; add `eyebrow`, `claimId`, and a single action slot. `HeroV2` owns the home hero. |
| `ProofArtifacts.astro` | KEEP | Its links to cookbook, CI, memory, and trust proof are appropriate for docs/architecture pages. Update only token styles and destination copy as routes consolidate. |
| `SearchModal.astro` | MODIFY | Keep the keyboard/search behavior used by docs. Simplify visual chrome, ensure focus restoration, and invoke it only from HeaderV2/DocsTopBar rather than the legacy mega-nav. |
| `SectionHeading.astro` | REPLACE | Its flexible title/highlight/layout API duplicates the new section system. Replace with `SectionContainer` plus `EyebrowLabel` and explicit page markup. |
| `StatusQuoTable.astro` | REPLACE | Its five-row status-quo comparison is too dense for the problem argument. `ColdStartTax` with three `ProblemExhibit`s tells the cold-start story in one viewport. |
| `SupportFab.astro` | REMOVE | A floating support CTA competes with the persistent install conversion action. Keep the `/support` route, but remove the global floating button. |
| `TerminalChrome.astro` | MODIFY | It already has the right slot-based terminal shell. Rename/move it to `shared/TerminalPanel.astro`, add semantic labelling and optional copy/header actions, and migrate callers. |
| `TokenComparisonBars.astro` | REPLACE | The static multi-row chart is replaced on marketing pages by `TokenMeter`, which animates only a cited, selected proof claim. Retain benchmark visualization needs in the modified scatter chart. |
| `ToolIntegrationGrid.astro` | REPLACE | A 30+ tool wall obscures the first six installation choices. Replace with `AgentSelector`/`AgentCard`; put the full compatibility matrix in docs. |
| `WhyNotX.astro` | REMOVE | The objections duplicate the redesigned FAQ and encourage competitor framing on a primary route. Fold valid questions into `faq.json`. |
| `architecture/ArchSpecs.astro` | MODIFY | Keep in technical architecture documentation. Read its tables from versioned technical data and add claim/method links where it presents measurements. |
| `architecture/ReadPathFlow.astro` | KEEP | Keep as detailed docs architecture evidence; it already maps real stages and has no marketing-home responsibility. Only inherit shared token and motion updates. |
| `architecture/SystemBlueprint.astro` | MODIFY | Keep as the platform/enterprise architecture artifact. Simplify its responsive layout and label local versus optional provider boundaries explicitly. |

### Existing non-component files that change

| File | Decision |
| --- | --- |
| `layouts/BaseLayout.astro` | MODIFY: import `HeaderV2`/`FooterV2` for marketing; retain SEO, locale, JSON-LD, font, theme, and main landmark responsibilities. Remove `SupportFab`; stop adding global ASCII art. Keep the count/reveal script only until its duties move to `ScrollReveal`/`TrustStrip`. |
| `layouts/DocsLayout.astro` | MODIFY: retain docs page shell, breadcrumb, pager, responsive rails, and `docsTree`; use the revised `DocsTopBar`, `DocsSidebar`, and `DocsFooter`, with a compact `InstallSection` at quickstart entry points. |
| `styles/global.css` | MODIFY: preserve the current palette/font/reset/theme tokens; centralize motion tokens, focus styles, section/container primitives, terminal primitives, and reduced-motion overrides. Remove obsolete global hero/ASCII/reveal selectors after callers migrate. |
| `styles/index.css` | REPLACE: split home-specific legacy styles into scoped component styles, plus `styles/pages/index.css` only for composition-level rules. Do not carry over the legacy hero terminal, compatibility grid, or broad pillar styles. |

---

## 2. New component specifications

Shared data types used below belong in `src/lib/marketing-types.ts`:

```ts
export type AgentId = 'cursor' | 'claude-code' | 'codex' | 'windsurf' | 'copilot' | 'other';

export interface Claim {
  id: string;
  value: string;
  label: string;
  basis: string;
  tokenizer?: string;
  date: string; // ISO date
  methodology: string;
  source?: string;
}

export interface Agent {
  id: AgentId;
  name: string;
  shortName: string;
  description: string;
  install: { command: string; shell: 'bash' | 'zsh' | 'powershell'; label: string }[];
  verifyCommand: string;
  docsHref: string;
  icon: string;
  supported: boolean;
}

export interface Capability {
  id: 'read' | 'remember' | 'coordinate' | 'govern';
  number: '01' | '02' | '03' | '04';
  title: string;
  thesis: string;
  description: string;
  icon: string;
  href: string;
}
```

Estimated LOC includes Astro markup, scoped CSS, and the component’s small client script; it excludes shared JSON and tests.

### Hero

#### `components/hero/HeroV2.astro` — 170 LOC

```ts
interface Props {
  eyebrow: string;
  headline: string;
  subline: string;
  installHref?: string;       // default: '#install'
  proofHref?: string;         // default: '#context-demo'
  claims: Claim[];            // hero trust/micro-proof claims
  agents: Agent[];
}
```

- Slots: `microProof` (optional, text/link below selector); no default slot.
- Data: `agents.json`, relevant `claims.json` entries, translated hero copy.
- Responsive: centered 900px text column; selector becomes full-width at `<640px`; trust facts wrap in two rows without horizontal scrolling.
- Animation: composes `HeroBackground`; orchestrated entrance sequence in the animation section below. The static first paint remains readable before scripts load.
- Accessibility: one page `<h1>`; CTA is an anchor; secondary proof link has descriptive label; selector uses a labelled radiogroup/listbox implementation; decorative background is `aria-hidden`.
- Sketch:

```text
┌─────────────────────────────────────────────────────────────┐
│  [animated, decorative background]                           │
│  THE CONTEXT LAYER FOR CODING AGENTS                         │
│  Give your agents context—not clutter.                       │
│  Supporting sentence                                         │
│  [ Install for Cursor ▾ ]  Watch it read smarter ↓           │
│  [agent picker / selected install command]                   │
│  AST-aware · modes · languages · supported agents            │
│  Local by default.  [Method →]                               │
└─────────────────────────────────────────────────────────────┘
```

#### `components/hero/AgentSelector.astro` — 185 LOC

```ts
interface Props {
  agents: Agent[];
  context: 'hero' | 'install' | 'docs' | 'inline';
  selected?: AgentId;         // SSR fallback only; Cursor by default
  showCommand?: boolean;      // true in hero/install/docs
  compact?: boolean;
  id?: string;
}
```

- Slots: `afterCommand` for micro-proof/verification; no default slot.
- Data: `agents.json`; reads/writes shared agent preference through `agentPreference.ts`.
- Responsive: in hero, selected control stays a single button with an overlay picker; install uses a six-card grid above 720px and a horizontally scrollable, keyboard-safe list below it; docs uses compact dropdown/list.
- Animation: selected label cross-fades (150ms); command content fades/raises (180ms); no automatic opening of the picker.
- Accessibility: buttons have `aria-pressed` in a labelled radio group, or a real `<select>` in compact mode; selection changes announce the updated command through a polite live region; Escape closes popover and returns focus; all options remain keyboard reachable.
- Sketch:

```text
[ Cursor ▾ ]
 ┌─────────────────────────────────────┐
 │ ● Cursor        Claude Code          │
 │   Codex         Windsurf             │
 │   Copilot       Other MCP client     │
 └─────────────────────────────────────┘
 SETUP FOR CURSOR
 [ curl … | sh                                      ]
```

#### `components/hero/HeroBackground.astro` — 90 LOC

```ts
interface Props { density?: 'low' | 'medium'; seed?: string; }
```

- Slots: none. Data: none; `seed` determines a deterministic set of CSS custom properties, so it is not random at runtime.
- Responsive: fixed absolute layer, clipped inside hero; reduces density on narrow screens.
- Animation: two low-contrast gradient/grid layers drift over 18–28s; disabled in reduced motion and never conveys meaning.
- Accessibility: `aria-hidden="true"`, `pointer-events: none`, minimum contrast behind text maintained.
- Sketch: `· · ·  ── subtle code/connection traces ──  · · ·`

### Problem and solution

#### `components/sections/ColdStartTax.astro` — 155 LOC

```ts
interface Props {
  eyebrow: string;
  title: string;
  body: string[];
  exhibits: ProblemExhibitData[];
  closing: string;
}
interface ProblemExhibitData { id: string; label: string; description: string; artifact: string; }
```

- Slots: `closing` optional override. Data: localized narrative copy; no numeric claims.
- Responsive: title/copy at 900px measure; three exhibits form three columns above 960px, one column below 640px.
- Animation: heading and body reveal first; three exhibits use 120ms staggered horizontal rise.
- Accessibility: section `aria-labelledby`; exhibits are an ordered list because their sequence is meaningful; all decorative marks hidden.
- Sketch:

```text
[ 01 ] THE COLD-START TAX
Every new agent starts by forgetting.      body copy
┌ RE-READING ─┐ ┌ RE-DISCOVERING ─┐ ┌ RE-EXPLAINING ─┐
│ mini proof  │ │ mini proof       │ │ mini proof     │
└─────────────┘ └──────────────────┘ └────────────────┘
Your agents do not need more context…
```

#### `components/sections/ProblemExhibit.astro` — 95 LOC

```ts
interface Props { item: ProblemExhibitData; index: number; }
```

- Slots: `artifact` for a compact terminal/timeline artifact; default displays `item.artifact` as mono text.
- Data: supplied only by `ColdStartTax`.
- Responsive: fill parent; fixed visual hierarchy, not fixed height.
- Animation: hover border/2px rise only for pointer devices; receives parent reveal delay.
- Accessibility: list item semantics come from parent; label is a heading, artifact is descriptive text rather than a fake interactive control.
- Sketch: `[01] RE-READING / one-sentence consequence / $ repeated read…`.

#### `components/sections/CapabilityCard.astro` — 115 LOC

```ts
interface Props { capability: Capability; headingLevel?: 'h2' | 'h3'; }
```

- Slots: none. Data: a `Capability` from `capabilities.json`.
- Responsive: four equal cards in `SolutionGrid`; content never truncates and moves to 2×2/1×4 grid.
- Animation: reveal stagger, 2px upward hover/focus-visible treatment; icon stroke has no continuous motion.
- Accessibility: semantic heading, descriptive link (`Read the shape of the code — learn more`), visible keyboard focus, icon hidden from assistive tech when title supplies meaning.
- Sketch: `[01] [icon] Read the shape of the code / Structure over file dumps. / description / Learn more →`.

#### `components/sections/SolutionGrid.astro` — 125 LOC

```ts
interface Props { eyebrow: string; title: string; intro: string; closing: string; capabilities: Capability[]; }
```

- Slots: `afterGrid` for a narrowly scoped deep link; no default slot.
- Data: `capabilities.json`, localized section copy.
- Responsive: 2×2 at ≥768px, single column below; max card count is four and not reused as a generic arbitrary grid.
- Animation: section intro reveal, then card cascade at 100ms intervals.
- Accessibility: `aria-labelledby`, `<ol>` communicates numbered capabilities.
- Sketch:

```text
[ 02 ] CONTEXT, ENGINEERED
Before an agent reads…
┌ 01 Read ─────────┐ ┌ 02 Remember ─────┐
└──────────────────┘ └──────────────────┘
┌ 03 Coordinate ───┐ ┌ 04 Govern ───────┐
└──────────────────┘ └──────────────────┘
Token savings are the receipt.
```

### Proof and shared proof primitives

#### `components/sections/ContextDemo.astro` — 260 LOC

```ts
interface DemoMode { id: 'full' | 'map' | 'signatures'; label: string; output: string; tokens: number; retained: string[]; }
interface Props {
  source: { path: string; revision: string; language: string; code: string; tokens: number };
  modes: DemoMode[];
  claim: Claim;
  methodologyHref: string;
}
```

- Slots: `cacheProof` below the meter; no default slot.
- Data: versioned demo fixture (new `src/data/demo-context.ts`), `claims.json`, translation labels. The source and outputs must be generated/verified from the same pinned revision, not invented UI copy.
- Responsive: 2-column original/output layout at ≥1024px; stacked with output immediately after tabs below 1024px. Source has an intentional fixed max height and an accessible expand control, not a scroll trap.
- Animation: changing mode replaces the output with a 160ms opacity/translate transition, then animates `TokenMeter`; never typewrites an output that appears to be live.
- Accessibility: `ModeSwitcher` provides tabs; panels use `role="tabpanel"`; tokens use text as well as bar; the “expand” action is a labelled button/link to the full artifact; every metric has a `BasisLabel`.
- Sketch:

```text
[ 03 ] SEE THE CONTEXT CHANGE     One file. Three useful views.
              [ Full | Map | Signatures ]
┌ ORIGINAL: src/auth.ts ──┐  ┌ MAP OUTPUT ───────────────┐
│ dimmed source            │  │ exports / deps / symbols  │
│ …                         │  │ [Expand full artifact]    │
└──────────────────────────┘  └──────────────────────────┘
4,200 tokens ━━━━━━━━━━━→ 180 tokens  [ 95.7% less ]
measured · method · date                                      [Method →]
```

#### `components/shared/ModeSwitcher.astro` — 105 LOC

```ts
interface Mode { id: string; label: string; description?: string; }
interface Props { modes: Mode[]; selected: string; controls: string; label?: string; }
```

- Slots: none. Data: `ContextDemo` modes.
- Responsive: equal-width segmented controls where possible; horizontally scrollable button group only when the labels cannot fit.
- Animation: active indicator slides 150ms; transitions are disabled for reduced motion.
- Accessibility: WAI-ARIA tabs pattern: `role=tablist`, roving Tab focus, Arrow/Home/End navigation, `aria-selected`, and `aria-controls`. It dispatches a `leanctx:modechange` custom event.
- Sketch: `[ Full ] [ Map ] [ Signatures ]`.

#### `components/shared/TokenMeter.astro` — 90 LOC

```ts
interface Props { before: number; after: number; claimId: string; label?: string; animate?: boolean; }
```

- Slots: `caption` for a contextual sentence. Data: selected `Claim` resolved by parent or `claims.json` loader.
- Responsive: text values wrap before the meter; visual bar takes full remaining width.
- Animation: IntersectionObserver + a mode-change event animates width and count from the previous rendered value over 550ms; initializes directly when reduced motion is requested.
- Accessibility: visible numeric before/after and percentage; meter has `role="img"` with complete `aria-label`, not an inaccessible color-only chart.
- Sketch: `4,200 tokens  ━━━━━━━━━━━━━━━━━  180 tokens   95.7% less`.

#### `components/shared/TerminalPanel.astro` — 105 LOC

```ts
interface Props {
  title: string;
  label?: string;
  language?: string;
  tone?: 'default' | 'proof' | 'success';
  scrollable?: boolean;
  copyText?: string;
}
```

- Slots: default terminal body; `headerActions`; `footer`.
- Data: supplied content; `CopyCommand` may be used in header/body.
- Responsive: shell fills container; code wraps by default, with explicit horizontal scrolling only for literal commands/source where wrapping would change meaning.
- Animation: no entrance of its own; terminal dots are static (unlike the legacy “live” pulse), avoiding a misleading live-state cue.
- Accessibility: `<section aria-label={label ?? title}>`; copy action is a button with explicit command text; code remains selectable; color does not signal success alone.
- Sketch: `● ● ●  ctx_read output [Copy] / ───────────────── / $ command / output`.

#### `components/shared/BasisLabel.astro` — 60 LOC

```ts
interface Props { claim: Claim; compact?: boolean; showMethodLink?: boolean; }
```

- Slots: none. Data: `Claim` from `claims.json`.
- Responsive: wraps inline terms cleanly; compact form is a caption under a metric.
- Animation: none.
- Accessibility: rendered as readable text, with a descriptive methodology link; `date` is a `<time datetime>`.
- Sketch: `Measured: repo · map mode · o200k · 2026-07-15  [Method →]`.

### Trust and install

#### `components/sections/TrustPillars.astro` — 150 LOC

```ts
interface TrustPillar { id: string; title: string; body: string; icon: string; href?: string; }
interface Props { eyebrow: string; title: string; pillars: TrustPillar[]; enterpriseHref: string; }
```

- Slots: `footer` for `TrustStrip`. Data: localized trust copy, `claims.json` for any metrics.
- Responsive: three columns at ≥900px, stack under 700px.
- Animation: heading then three cards at 120ms stagger; cards only have pointer hover movement.
- Accessibility: a labelled section and a three-item list; enterprise link has contextual name; local/deterministic/evidence claims use `BasisLabel` when numeric.
- Sketch: `Local-first | Deterministic | Accountable / brief evidence / [Enterprise →]`.

#### `components/sections/TrustStrip.astro` — 110 LOC

```ts
interface TrustMetric { claimId?: string; value: string; label: string; countTo?: number; suffix?: string; }
interface Props { metrics: TrustMetric[]; variant?: 'hero' | 'full'; }
```

- Slots: none. Data: `claims.json` for factual values; static capability labels may omit a claim id.
- Responsive: hero form wraps into two even rows; full form stacks values and labels rather than creating a horizontally scrolling metric rail.
- Animation: IntersectionObserver count-up only for verified numeric metrics, once per page; capability-only values fade in. Count-up is static with reduced motion.
- Accessibility: a list with textual values and labels; no reliance on the animated number alone; claims are linked/labelled with a basis where needed.
- Sketch: `AST-aware context  ·  10 read modes  ·  26 languages  ·  30+ agents`.

#### `components/sections/InstallSection.astro` — 210 LOC

```ts
interface Props {
  eyebrow: string;
  title: string;
  body: string;
  agents: Agent[];
  quickstartHref: string;
  compact?: boolean;
}
```

- Slots: `reassurance`; `doctorOutput` (inside `TerminalPanel`).
- Data: `agents.json`, translated copy, a versioned expected `lean-ctx doctor` excerpt.
- Responsive: full variant has six `AgentCard`s in 3×2/2×3/1×6 layouts, command then doctor output; compact docs variant uses `AgentSelector` with a single command card.
- Animation: section reveal; command cross-fade after selection; copy feedback only upon interaction.
- Accessibility: cards use radio semantics; selected command heading updates; copy action announces success/failure; doctor output is not auto-read by a live region.
- Sketch:

```text
[ 06 ] YOUR FIRST BETTER READ IS MINUTES AWAY
Install for the agent you already use.
[ Cursor ] [ Claude Code ] [ Codex ] [ Windsurf ] [ Copilot ] [ Other ]
SETUP FOR CURSOR
┌ curl … | sh                                      [Copy] ┐
└─────────────────────────────────────────────────────────┘
Then run `lean-ctx doctor`.
┌ ● ● ● doctor / ✓ Binary … ✓ MCP config … ──────────────┐
```

#### `components/shared/AgentCard.astro` — 85 LOC

```ts
interface Props { agent: Agent; selected: boolean; name: string; disabled?: boolean; }
```

- Slots: none. Data: one `Agent` supplied by selector/install section.
- Responsive: fixed visual weight, min-height only; grid owns placement.
- Animation: 120ms selected outline/background transition; no card lift when acting as a radio option.
- Accessibility: a real radio input visually styled through its label, or a `button role="radio"`; uses `aria-checked`; disabled agents expose unavailable state and cannot be selected.
- Sketch: `◉ [icon] Cursor / CLI redirect + MCP setup`.

#### `components/shared/CopyCommand.astro` — 85 LOC

```ts
interface Props { command: string; label?: string; language?: string; showPrompt?: boolean; }
```

- Slots: `after` (optional verification hint). Data: resolved command only.
- Responsive: command wraps at narrow widths while its Copy button remains visible; explicit copy value contains the unwrapped original command.
- Animation: “Copied” confirmation fades after 1.8s, without changing layout.
- Accessibility: button `aria-label="Copy install command"`; a polite live region reports copied/failed; do not rely on icon-only success; fallback selects command text if Clipboard API is unavailable.
- Sketch: `$ curl -fsSL … | sh                       [Copy]`.

### Layout and common composition

#### `components/layout/HeaderV2.astro` — 170 LOC

```ts
interface NavItem { label: string; href: string; current?: boolean; }
interface Props { nav: NavItem[]; githubHref: string; installHref?: string; }
```

- Slots: `utilities` for theme/search controls; no default slot.
- Data: locale-aware route helper, translated labels, optionally a build-time GitHub stars value (never an unlabelled live fetch).
- Responsive: logo + Install remains visible; navigation converts to a labelled disclosure menu below 900px, with focus trapped only while menu is open.
- Animation: header is static on load; mobile menu opacity/height transitions 160ms.
- Accessibility: `<header>`, `<nav aria-label="Primary">`, current route uses `aria-current="page"`; menu button exposes `aria-expanded`; skip link remains in BaseLayout.
- Sketch: `lean-ctx | How it works  Docs  Pricing  Enterprise | GitHub  theme  search | [Install]`.

#### `components/layout/FooterV2.astro` — 110 LOC

```ts
interface FooterLink { label: string; href: string; external?: boolean; }
interface Props { productLinks: FooterLink[]; resourceLinks: FooterLink[]; license: string; }
```

- Slots: `locale` for `LanguageSwitcher`.
- Data: localized direct routes, `Facts.license`, current year from build/layout.
- Responsive: two columns become a single two-group stack; legal line follows content.
- Animation: none.
- Accessibility: named navigation groups, external links disclose that they open externally, language control stays labelled.
- Sketch: `Product: How it works / Pricing / Enterprise / Changelog | Resources: Docs / GitHub / Discord / Security / Apache-2.0…`.

#### `components/shared/SectionContainer.astro` — 120 LOC

```ts
interface Props {
  id?: string;
  eyebrow?: string;
  ordinal?: string;
  title?: string;
  intro?: string;
  width?: 'text' | 'content' | 'wide';
  tone?: 'default' | 'raised';
  headingLevel?: 'h2' | 'h3';
}
```

- Slots: default; `headerActions`; `afterHeader`; `footer`.
- Data: passed content only. It intentionally does not read translation files.
- Responsive: `text=900px`, `content=1100px`, `wide=1200px`; standard section vertical rhythm uses global custom properties.
- Animation: wraps header/content in `ScrollReveal` only when `reveal` is requested via an optional future prop; no automatic animation by default.
- Accessibility: emits a unique heading id and connects it with `aria-labelledby`; never renders a `<section>` without a heading/label.
- Sketch: `[ 02 ] EYEBROW / H2 title / intro / [default section content]`.

#### `components/shared/EyebrowLabel.astro` — 45 LOC

```ts
interface Props { label: string; ordinal?: string; tone?: 'accent' | 'muted'; }
```

- Slots: none. Data: passed text only. Responsive: inline, wraps naturally. Animation: inherited from parent only. Accessibility: plain text; ordinal is included in accessible text. Sketch: `[ 02 ]  CONTEXT, ENGINEERED`.

#### `components/shared/ScrollReveal.astro` — 115 LOC

```ts
interface Props { as?: 'div' | 'section' | 'li'; delay?: number; once?: boolean; threshold?: number; }
```

- Slots: default. Data: none. Responsive: transparent wrapper with no layout styles except the requested element.
- Animation: toggles `data-revealed` through one shared IntersectionObserver; default `once=true`, threshold `0.15`, and no observer/hidden state in reduced motion or no-JS rendering.
- Accessibility: content is never `display:none`, `visibility:hidden`, or inaccessible before observer execution; honors `prefers-reduced-motion`.
- Sketch: `[offscreen content] → [opacity 1, translateY 0]`.

### Supporting page-specific components required by the page map

These are new because their current equivalents do not match the redesigned information architecture.

| Component | Contract and responsibility | LOC / sketch |
| --- | --- | --- |
| `sections/WorkflowLoop.astro` + `WorkflowStep.astro` | `WorkflowLoop` props: `{ eyebrow, title, steps: WorkflowStepData[], catalogHref, agents }`; `WorkflowStep` props: `{ step, index }`. Three real, pinned read/search/remember trace steps for How it works. Each step owns one `TerminalPanel`; output data is versioned fixtures. Stack vertically on mobile, 3-column timeline on desktop; stagger on reveal; ordered-list semantics and code labels. | 210 + 100. `01 Read → 02 Search & execute → 03 Remember & share` |
| `sections/PricingPlans.astro` | Props: `{ tiers: PricingTier[], selected?: string; claim?: Claim }`; slot `footer`. Extract existing inline tier markup so pricing boundary/copy and `BasisLabel` are consistent. 3 columns → stack; static unless user selects optional comparison; headings/list semantics. | 175. `Local OSS | Team controls | Enterprise governance` |
| `sections/RoiCalculator.astro` | Props: `{ defaults: RoiInputs; assumptions: Claim[]; currency?: 'USD' }`; no slots. Move existing PricingPage calculator and client math here, preserving no-JS visible assumptions/results. Responsive form/result stack; input changes animate result only when motion allowed; labelled inputs, result live region, explicit assumptions/method links. | 240. `[team] [usage] → hours/tokens/value` |
| `sections/PolicyEvidenceFlow.astro` | Props: `{ steps: { title; description; evidence; }[]; claim?: Claim }`; slot `artifact`. Enterprise’s policy → decision → signed ledger → report proof. Four-column flow → vertical sequence; reveal per node; ordered-list semantics and labelled arrows hidden from AT. | 150. `Policy → access decision → signed ledger → report` |
| `sections/DocsQuickstart.astro` | Props: `{ agents: Agent[]; firstRead: string; docsHref: string }`; slots `expectedOutput`, `nextStep`. Uses compact `AgentSelector`, `CopyCommand`, `TerminalPanel`. Inserted in Docs home/getting-started only, not every reference page; all selection/copy a11y contracts come from shared primitives. | 160. `choose agent → copy setup → doctor → first ctx_read` |

---

## 3. Page composition map

### `IndexPage.astro`

```text
BaseLayout (marketing, modified)
├── HeaderV2
├── HeroV2
│   ├── HeroBackground
│   ├── AgentSelector
│   │   └── CopyCommand
│   ├── TrustStrip (hero)
│   └── BasisLabel (micro-proof claim)
├── ColdStartTax
│   └── ProblemExhibit ×3
├── SolutionGrid
│   └── CapabilityCard ×4
├── ContextDemo
│   ├── ModeSwitcher
│   ├── TerminalPanel ×2
│   ├── TokenMeter
│   └── BasisLabel
├── TrustPillars
│   └── TrustStrip (full)
├── InstallSection
│   ├── AgentCard ×6
│   ├── AgentSelector (shared state controller)
│   ├── CopyCommand
│   └── TerminalPanel (doctor output)
├── FaqSection (modified, `faq.json`)
└── FooterV2
```

`IndexPage` should only compose components and load the four data files plus localized copy. It should not contain client DOM code, terminal markup, agent command branches, numbers, or component-specific CSS.

### `HowItWorksPage.astro`

```text
BaseLayout (marketing)
├── HeaderV2
├── PageHero (modified; one action: Install for selected agent)
├── SectionContainer (the causal premise)
├── WorkflowLoop
│   └── WorkflowStep ×3
│       └── TerminalPanel
├── ContextDemo (same pinned proof fixture as home)
│   ├── ModeSwitcher
│   ├── TerminalPanel ×2
│   ├── TokenMeter
│   └── BasisLabel
├── ArchitectureDiagram (modified; bounded-plane summary)
├── TrustStrip
├── InstallSection (compact)
├── FaqSection (modified, mechanism subset)
└── FooterV2
```

The page’s single question is “what happens between my agent and codebase?” The detailed `ReadPathFlow`, `SystemBlueprint`, and `ArchSpecs` remain at docs architecture routes rather than making this page a second reference manual.

### `PricingPage.astro`

```text
BaseLayout (marketing)
├── HeaderV2
├── PageHero (modified; local OSS boundary and one Start free action)
├── PricingPlans
│   └── BasisLabel (only where a numerical value claim is shown)
├── RoiCalculator
│   └── BasisLabel ×n (assumptions)
├── SectionContainer (what the team/enterprise layer adds)
│   └── CapabilityCard ×4 (reused, pricing copy variant sourced by page)
├── TrustStrip (evidence/reproducibility)
├── FaqSection (modified, `faq.json` pricing subset)
├── CtaSection (modified; agent-aware Start free action)
└── FooterV2
```

The existing pricing functions (`getPricingTiers`, `getPricingFaq`, ROI calculator library) remain the pricing data/math authority; page markup and the existing inline browser script migrate into `PricingPlans` and `RoiCalculator`.

### `EnterprisePage.astro`

```text
BaseLayout (marketing)
├── HeaderV2
├── PageHero (modified; request architecture review is valid primary action)
├── SectionContainer (local/hosted deployment boundary)
│   └── TerminalPanel or deployment evidence artifact
├── PolicyEvidenceFlow
│   └── TerminalPanel (signed ledger excerpt)
├── TrustPillars (enterprise copy variant)
├── ArchitectureDiagram (modified; platform evidence)
├── SectionContainer (controls, support, engagement)
├── FaqSection (modified, enterprise subset)
├── CtaSection (modified; architecture review)
└── FooterV2
```

Enterprise must not reuse `InstallSection` as a primary conversion surface. It may use a small contextual “install locally first” link, while its main action is the explicit architecture-review path.

### `DocsLayout.astro`

```text
BaseLayout (docs=true, modified)
├── DocsTopBar (modified)
│   ├── SearchModal (modified)
│   └── HeaderV2-compatible Install link
├── DocsSidebar (modified; quickstart-first tree)
├── DocsLayout content shell
│   ├── breadcrumb/header
│   ├── DocsQuickstart (Docs home + getting-started only)
│   │   ├── AgentSelector (compact)
│   │   ├── CopyCommand
│   │   └── TerminalPanel (expected doctor output)
│   ├── page `<slot />`
│   │   ├── DocsCallout (kept)
│   │   ├── TerminalPanel (migrated code/output examples)
│   │   ├── ReadPathFlow (kept in architecture docs)
│   │   ├── SystemBlueprint (modified in architecture docs)
│   │   └── ArchSpecs (modified in architecture docs)
│   ├── pager
│   └── generated on-this-page rail
└── DocsFooter (modified)
```

`DocsQuickstart` is deliberately route-controlled: show it on `/docs/` and `/docs/getting-started/`, rather than injecting repeated install UI into all 40+ reference documents.

---

## 4. Data architecture

### File ownership

```text
website/src/data/
├── claims.json         # public measurable claim registry
├── agents.json         # six first-class install targets
├── capabilities.json   # four homepage product capabilities
├── faq.json            # question/answer sets and page assignment
├── demo-context.ts     # pinned source fixture + generated mode outputs (new)
└── [existing addons.json, config-schema.json]
```

`demo-context.ts` is TypeScript rather than JSON because the fixture can import a generated hash/verifier and exports strongly typed mode data. It must include the source file path and immutable revision. A test should fail if the supplied output/hash and corresponding claims do not agree.

### `claims.json`

```json
{
  "claims": [
    {
      "id": "demo-auth-map-compression",
      "value": "95.7%",
      "label": "less context for the map view",
      "basis": "lean-ctx demo fixture, src/auth.ts at pinned revision, map mode",
      "tokenizer": "o200k",
      "date": "2026-07-15",
      "methodology": "/docs/benchmarks#methodology",
      "source": "website/src/data/demo-context.ts"
    }
  ]
}
```

Rules:

- `id` is stable and referenced by components/data, never by a display string.
- `date` is the measurement date, not build date. Do not show an uncited public percentage.
- `methodology` must resolve to a public, reproducible description. `source` is optional implementation provenance.
- Existing `lib/facts.ts` remains authoritative for generated binary facts (tool/read-mode counts). Add an adapter that resolves those facts to `Claim` objects at render time, rather than duplicating the values in JSON.

### `agents.json`

```json
{
  "default": "cursor",
  "agents": [
    {
      "id": "cursor",
      "name": "Cursor",
      "shortName": "Cursor",
      "description": "Configure the local context runtime for Cursor.",
      "icon": "cursor",
      "supported": true,
      "install": [{
        "label": "macOS / Linux",
        "shell": "bash",
        "command": "curl -fsSL https://… | sh && lean-ctx setup"
      }],
      "verifyCommand": "lean-ctx doctor",
      "docsHref": "/docs/getting-started#cursor"
    }
  ]
}
```

Populate commands only after they are validated against the current installer/CLI. The selector contains the initial six: Cursor, Claude Code, Codex, Windsurf, Copilot, and Other MCP client. The full 30+ compatibility list stays in docs/generated data.

### `capabilities.json`

```json
{
  "capabilities": [
    {
      "id": "read",
      "number": "01",
      "title": "Read the shape of the code",
      "thesis": "Structure over file dumps.",
      "description": "AST-aware reads, code graphs, intent detection, and focused search show the symbols, dependencies, and changes the task needs.",
      "icon": "nodes",
      "href": "/how-it-works#read"
    }
  ]
}
```

The other canonical IDs are `remember`, `coordinate`, and `govern`. Icon names resolve through one local SVG map; no arbitrary HTML/SVG is stored in JSON.

### `faq.json`

```json
{
  "items": [
    {
      "id": "token-compressor",
      "question": "Is lean-ctx just a token compressor?",
      "answer": "…",
      "pages": ["home", "how-it-works"],
      "order": 10
    }
  ]
}
```

`pages` enables page-specific subsets without duplicate question wording. Answers stay plain text or a constrained rich-text token format; do not pass arbitrary HTML from JSON to `set:html`. Generate FAQPage JSON-LD from the exact rendered subset.

### Data loading and validation

- Add `src/lib/marketing-data.ts` with typed loaders: `getClaims()`, `getClaim(id)`, `getAgents()`, `getAgent(id)`, `getCapabilities()`, and `getFaq(page)`.
- Validate unique IDs, the fixed agent/capability unions, valid internal `href`s, ISO dates, non-empty commands, and that each `claimId` used by a page resolves. Fail the production build on violation.
- Component props receive data objects rather than reading files themselves, except `AgentSelector` may receive the shared `Agent[]` to remain portable. This keeps rendering testable and lets page templates control locale overlays.

---

## 5. Shared agent state

### Contract

Create `src/lib/agentPreference.ts` as a small browser/SSR-safe module. It owns one state value, `AgentId`, and the event name `leanctx:agentchange`.

Selection precedence on every page load:

1. Valid `?agent=<AgentId>` URL parameter.
2. Valid `localStorage['leanctx:selected-agent']` value.
3. `agents.json.default`, initially `cursor`.

When a user changes agent:

1. Validate against `agents.json` and update all mounted selector/card instances.
2. Persist to localStorage.
3. Dispatch `CustomEvent<AgentId>('leanctx:agentchange', { detail: agentId })`.
4. Update command labels/blocks and every “Install for …” label.
5. Use `history.replaceState` to set `agent=<id>` only while the user is in an install flow or after they explicitly selected an option. Preserve unrelated query parameters and never add an invalid/default-only query parameter during SSR.

All site-generated install links should carry the selected `agent` parameter when an explicit selection exists; it is the fallback for private browsing, copied links, and pages opened before localStorage is available. A user can remove the parameter to resume stored/default behavior.

### Implementation notes

- Render Cursor commands server-side so first paint, crawlers, and no-JS users get a complete install route.
- Every stateful component subscribes after DOMContentLoaded and unsubscribes on `pagehide`; do not put multiple independent localStorage implementations inside Hero, Header, and Install components.
- If a valid URL choice overrides storage, store that choice only after page interaction or a user-visible selection acknowledgement. This avoids a shared/kiosk URL silently rewriting a visitor’s long-term preference.
- Store only the agent identifier, never commands, user/project data, or analytics identity.

---

## 6. Animation orchestration

### Global motion policy

- Centralize durations/easings in `global.css`: `--motion-fast: 150ms`, `--motion-base: 240ms`, `--motion-slow: 560ms`, `--ease-out: cubic-bezier(.22,1,.36,1)`.
- `ScrollReveal` starts when 15% of a section enters view and reveals once. It must not hide content in no-JS mode.
- `@media (prefers-reduced-motion: reduce)` removes transforms, looping backgrounds, count-up, and smooth tab transitions; content is immediately visible and meters render final numbers.
- Delay is only for narrative sequencing, never for essential controls or focusable content.

### Home sequence

```text
Hero (on load)
  0ms     HeroBackground fades in
  200ms   eyebrow slides up/fades in
  400ms   headline fades up
  600ms   subline fades up
  800ms   install CTA + AgentSelector fade up
  1000ms  TrustStrip fades in

Problem (on scroll)
  0ms     section heading fades up
  200ms   body copy fades up
  400ms   ProblemExhibit 1 slides in
  520ms   ProblemExhibit 2 slides in
  640ms   ProblemExhibit 3 slides in

Solution (on scroll)
  0ms     eyebrow/headline reveals
  180ms   intro reveals
  360ms   Capability 01 reveals
  460ms   Capability 02 reveals
  560ms   Capability 03 reveals
  660ms   Capability 04 reveals
  800ms   closing line fades in

Proof (on scroll / interaction)
  0ms     heading reveals
  220ms   terminal panels reveal together
  360ms   mode selector becomes active
  on tab  output cross-fades (160ms), then TokenMeter animates (550ms)
  520ms   BasisLabel/method link fades in

Trust (on scroll)
  0ms     heading reveals
  220ms   pillar 1 reveals
  340ms   pillar 2 reveals
  460ms   pillar 3 reveals
  620ms   TrustStrip fades in; verified count-up begins once visible

Install (on scroll / interaction)
  0ms     heading/body reveals
  200ms   AgentCard grid reveals
  420ms   selected command panel reveals
  540ms   doctor TerminalPanel reveals
  on pick  selected state (120ms), command replacement (180ms)
  on copy  confirmation appears for 1.8s; no layout movement

FAQ / Footer
  FAQ heading and rows use a brief 80ms row cascade; native details opens immediately.
  Footer has no entrance animation.
```

### Other pages

| Page/section | Motion |
| --- | --- |
| How it works `WorkflowLoop` | Headline then its three ordered steps at 140ms intervals; terminal content itself is static, not typewritten. |
| Pricing plans | Hero/plan group reveal only. Calculator result updates with a 180ms number opacity transition after valid input, never a looping count. |
| Enterprise `PolicyEvidenceFlow` | Each flow node reveals in causal order (0/160/320/480ms); arrows become visible with their destination node. |
| Docs | No ornamental reveal on reference content. DocsQuickstart may use the compact command selection transition; navigation/menu motion remains 150ms maximum. |

---

## 7. File and folder structure

```text
website/src/
├── components/
│   ├── hero/
│   │   ├── HeroV2.astro
│   │   ├── AgentSelector.astro
│   │   └── HeroBackground.astro
│   ├── sections/
│   │   ├── ColdStartTax.astro
│   │   ├── ProblemExhibit.astro
│   │   ├── CapabilityCard.astro
│   │   ├── SolutionGrid.astro
│   │   ├── ContextDemo.astro
│   │   ├── TrustPillars.astro
│   │   ├── TrustStrip.astro
│   │   ├── InstallSection.astro
│   │   ├── WorkflowLoop.astro
│   │   ├── WorkflowStep.astro
│   │   ├── PricingPlans.astro
│   │   ├── RoiCalculator.astro
│   │   ├── PolicyEvidenceFlow.astro
│   │   └── DocsQuickstart.astro
│   ├── shared/
│   │   ├── TerminalPanel.astro
│   │   ├── TokenMeter.astro
│   │   ├── CopyCommand.astro
│   │   ├── AgentCard.astro
│   │   ├── EyebrowLabel.astro
│   │   ├── ScrollReveal.astro
│   │   ├── BasisLabel.astro
│   │   ├── ModeSwitcher.astro
│   │   └── SectionContainer.astro
│   ├── layout/
│   │   ├── HeaderV2.astro
│   │   └── FooterV2.astro
│   ├── architecture/                 # retained detailed docs artifacts
│   │   ├── ArchSpecs.astro
│   │   ├── ReadPathFlow.astro
│   │   └── SystemBlueprint.astro
│   └── [kept legacy/documentation components during migration]
├── data/
│   ├── claims.json
│   ├── agents.json
│   ├── capabilities.json
│   ├── faq.json
│   ├── demo-context.ts
│   ├── addons.json                   # existing
│   └── config-schema.json            # existing
├── lib/
│   ├── agentPreference.ts
│   ├── marketing-data.ts
│   ├── marketing-types.ts
│   └── [existing facts, pricing, docs/i18n utilities]
├── layouts/
│   ├── BaseLayout.astro              # modified
│   └── DocsLayout.astro              # modified
└── styles/
    ├── global.css                    # tokens, reset, primitives, motion
    ├── components/                   # only cross-component composition styles, if needed
    └── pages/
        └── index.css                 # page composition only
```

Prefer scoped `<style>` blocks inside the component that owns the DOM. `styles/components/` is an exception for genuinely cross-cutting styles only; do not recreate a monolithic `index.css` there.

---

## 8. Migration strategy

### Phase 0 — establish safe foundations

1. Freeze current route behavior and record a visual/HTML baseline for `/`, `/how-it-works/`, `/pricing/`, `/enterprise/`, and docs getting started.
2. Add `marketing-types.ts`, `marketing-data.ts`, the JSON schema/data loaders, and validation tests. Seed only claims that have a reproducible basis.
3. Add motion/focus/container primitives to `global.css` without removing current selectors. Add the shared agent state module with browser tests for URL/storage/default precedence.

### Phase 1 — build beside the old home

1. Create the new directories and shared primitives (`TerminalPanel`, `CopyCommand`, `BasisLabel`, `ModeSwitcher`, `ScrollReveal`, `SectionContainer`, `AgentCard`).
2. Build the new hero, problem, solution, proof, trust, and install sections against fixtures/data. Do not edit legacy `IndexPage` content while the demo claim remains unverified.
3. Create `IndexPageV2.astro` or render `IndexPage` by a server-side `siteVariant` flag. The flag must be a build/runtime configuration value, not a client-side flicker or crawler-dependent experiment.
4. Use the same SEO title, canonical URL, structured FAQ data, and analytics event names on both variants. Do not run two home pages with competing canonicals.

### Phase 2 — cut over the landing page

1. QA desktop/mobile, keyboard-only flows, screen-reader labels, no-JS fallback, reduced motion, light/dark themes, and the six agent commands.
2. Verify every visible number resolves to `claims.json` or the generated facts adapter; verify each Method link and demo revision.
3. Swap the canonical `/` route to the V2 composition. Keep the old template in source for one release cycle but remove it from build/routing. Redirect/retain legacy anchors only where analytics or external links demonstrate need.
4. Remove obsolete home-only CSS and imports only after `rg` confirms no callers remain.

### Phase 3 — migrate core pages and docs chrome

1. Update `BaseLayout` to HeaderV2/FooterV2 and retire `SupportFab` globally after a route-by-route check.
2. Extract current PricingPage inline plans/calculator into the new sections without changing the underlying pricing/ROI library behavior; then apply revised copy/layout.
3. Recompose How it works and Enterprise with the shared proof/trust primitives. Preserve technical diagrams on their deep routes.
4. Update DocsTopBar/Sidebar/Footer and add `DocsQuickstart` only on docs index/getting started. Migrate `TerminalChrome` callers to `TerminalPanel` incrementally.

### Phase 4 — deprecate deliberately

1. Remove references to `Header`, `Footer`, `MegaDropdown`, `AsciiHeroBg`, `BuildDoors`, audience components, `WhyNotX`, `SupportFab`, and legacy home comparison components after their last route is migrated.
2. Keep old page routes until a route audit defines redirects; do not delete docs/architecture artifacts merely because they left the home page.
3. Run `npm run validate`, `npm run build`, route/link checks, and a visual regression pass after each page cutover.
4. Localize only after English data schemas/copy have stabilized; add locale-specific overlays, not duplicated component branches.

### Release gates

- No public quantitative claim without `Claim.id`, basis, date, and methodology link.
- No broken selected-agent flow between Header, Hero, Install, Pricing CTA, and DocsQuickstart.
- No animation that hides content without JavaScript or violates reduced-motion preferences.
- No new primary-page mega navigation, compatibility wall, support FAB, or competing CTA.
- Zero route regressions for docs, pricing library behavior, SEO metadata, and existing i18n page generation.

