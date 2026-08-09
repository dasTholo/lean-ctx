# lean-ctx Design System v2

**Direction:** dark editorial infrastructure. The page should feel like a precise
technical publication with a living product surface—not a generic AI dashboard.
Use one brand signal (emerald), near-black depth, real terminal evidence, and motion
that communicates state or hierarchy.

This specification replaces visual drift. Component CSS may use semantic custom
properties only; hex, RGB, and arbitrary Tailwind colour utilities are forbidden
outside the token declarations below. Indigo and purple remain data-viz-only.

## 1. Visual rules

- **Signal hierarchy:** eyebrow → statement → short evidence-led explanation →
  one proof artifact → one primary action.
- **Surfaces:** `bg` is page canvas; `surface` is a panel; `surface-2` is a
  control/header; `surface-3` is code/inset; `elevated` is floating UI only.
- **Brand colour:** emerald appears on primary actions, interactive focus, active
  state, positive/live status, and a small number of structural ticks. It is not
  decorative body copy.
- **Data colour:** `--viz-*` is exclusively charts, legends, or comparative
  before/after evidence. It must never colour a button, link, tab, nav item, or badge.
- **Evidence first:** use real command output, benchmark methodology, and source
  links. Every numeric claim includes its basis adjacent to the number.
- **Density:** a viewport contains one proposition. Prefer a large proof artifact
  over a grid of small feature cards.

## 2. Tokens: the only source of visual values

Put this block at the top of `src/styles/global.css`, after Tailwind’s import. It
supersedes the current duplicated palette/type/motion definitions. It intentionally
keeps the existing font stack and near-black/emerald character while adding explicit
interaction and atmospheric roles.

```css
@import "tailwindcss";
@source "../../src";

@theme {
  /* Fonts */
  --font-sans: var(--font-inter, "Inter"), -apple-system, BlinkMacSystemFont,
    "Segoe UI", var(--fallback-i18n), sans-serif;
  --font-display: var(--font-space-grotesk, "Space Grotesk"), var(--font-sans);
  --font-mono: var(--font-jetbrains-mono, "JetBrains Mono"), "Fira Code",
    "Cascadia Code", var(--fallback-i18n), monospace;

  /* Dark palette: source values. Theme rules below assign the active values. */
  --color-bg: #050507;
  --color-surface: #0a0a0f;
  --color-surface-2: #111118;
  --color-surface-3: #18181f;
  --color-surface-elevated: #0d0d14;
  --color-border: #1a1a24;
  --color-border-strong: #2a2a38;
  --color-text: #b0b0c4;
  --color-text-bright: #eeeef5;
  --color-text-muted: #8585a0;
  --color-accent: #34d399;
  --color-accent-hover: #6ee7b7;
  --color-accent-pressed: #10b981;
  --color-danger: #f87171;
  --color-warning: #fbbf24;
  --color-success: var(--color-accent);
  --color-viz-indigo: #818cf8;
  --color-viz-purple: #d4a0ff;

  /* Typography */
  --text-display: clamp(3.25rem, 2.05rem + 5.2vw, 6.75rem);
  --text-h1: clamp(2.5rem, 1.75rem + 3.3vw, 4.5rem);
  --text-h2: clamp(2rem, 1.45rem + 2.15vw, 3.25rem);
  --text-h3: clamp(1.375rem, 1.12rem + 0.9vw, 1.75rem);
  --text-h4: 1.125rem;
  --text-body-lg: clamp(1.0625rem, 1rem + 0.35vw, 1.25rem);
  --text-body: 1rem;
  --text-small: 0.875rem;
  --text-caption: 0.75rem;
  --text-micro: 0.6875rem;
  --weight-body: 400;
  --weight-medium: 500;
  --weight-semibold: 600;
  --weight-bold: 700;
  --leading-display: 0.98;
  --leading-heading: 1.08;
  --leading-snug: 1.25;
  --leading-body: 1.65;
  --leading-relaxed: 1.75;
  --tracking-display: -0.045em;
  --tracking-heading: -0.035em;
  --tracking-tight: -0.02em;
  --tracking-label: 0.14em;

  /* 8px spacing grid */
  --space-1: 0.25rem;  /* 4 */
  --space-2: 0.5rem;   /* 8 */
  --space-3: 0.75rem;  /* 12 */
  --space-4: 1rem;     /* 16 */
  --space-5: 1.5rem;   /* 24 */
  --space-6: 2rem;     /* 32 */
  --space-7: 3rem;     /* 48 */
  --space-8: 4rem;     /* 64 */
  --space-9: 6rem;     /* 96 */
  --space-10: 8rem;    /* 128 */
  --space-11: 10rem;   /* 160 */
  --space-12: 12rem;   /* 192 */

  /* Layout, edge and elevation */
  --container-narrow: 47.5rem; /* 760px */
  --container-medium: 60rem;   /* 960px */
  --container-wide: 75rem;     /* 1200px */
  --gutter: clamp(1.25rem, 0.8rem + 2vw, 2rem);
  --radius-sm: 0.375rem;
  --radius-md: 0.625rem;
  --radius-lg: 0.875rem;
  --radius-pill: 999px;
  --border-default: 1px solid var(--color-border);
  --border-strong: 1px solid var(--color-border-strong);

  /* Motion */
  --ease-out: cubic-bezier(0.22, 1, 0.36, 1);
  --ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);
  --duration-instant: 120ms;
  --duration-fast: 200ms;
  --duration-base: 300ms;
  --duration-reveal: 600ms;
  --duration-slow: 12000ms;
}
```

### Theme assignments and atmosphere

These variables are runtime semantic tokens. Every component must reference these
roles—not literals and not the dark source values in `@theme`.

```css
:root,
:root[data-theme="dark"] {
  color-scheme: dark;
  --color-bg: #050507;
  --color-surface: #0a0a0f;
  --color-surface-2: #111118;
  --color-surface-3: #18181f;
  --color-surface-elevated: #0d0d14;
  --color-border: #1a1a24;
  --color-border-strong: #2a2a38;
  --color-text: #b0b0c4;
  --color-text-bright: #eeeef5;
  --color-text-muted: #8585a0;
  --color-accent: #34d399;
  --color-accent-hover: #6ee7b7;
  --color-accent-pressed: #10b981;
  --color-danger: #f87171;
  --color-warning: #fbbf24;
  --color-viz-indigo: #818cf8;
  --color-viz-purple: #d4a0ff;
  --overlay-subtle: rgb(255 255 255 / 0.02);
  --overlay-hover: rgb(255 255 255 / 0.05);
  --overlay-pressed: rgb(255 255 255 / 0.08);
  --overlay-glass: rgb(10 10 15 / 0.72);
  --glow-hero: rgb(52 211 153 / 0.16);
  --glow-section: rgb(52 211 153 / 0.08);
  --glow-control: rgb(52 211 153 / 0.22);
  --grid-line: rgb(255 255 255 / 0.035);
  --shadow-card: 0 10px 30px rgb(0 0 0 / 0.26);
  --shadow-elevated: 0 22px 60px rgb(0 0 0 / 0.42);
  --shadow-glow: 0 0 32px var(--glow-control);
}

:root[data-theme="light"] {
  color-scheme: light;
  --color-bg: #f7f8fb;
  --color-surface: #ffffff;
  --color-surface-2: #f0f1f5;
  --color-surface-3: #e8e9ef;
  --color-surface-elevated: #ffffff;
  --color-border: #d5d8e0;
  --color-border-strong: #bcc4d0;
  --color-text: #3a3d4e;
  --color-text-bright: #111827;
  --color-text-muted: #5f6775;
  --color-accent: #047857;
  --color-accent-hover: #059669;
  --color-accent-pressed: #065f46;
  --color-danger: #dc2626;
  --color-warning: #b45309;
  --color-viz-indigo: #4f46e5;
  --color-viz-purple: #7c3aed;
  --overlay-subtle: rgb(17 24 39 / 0.025);
  --overlay-hover: rgb(17 24 39 / 0.05);
  --overlay-pressed: rgb(17 24 39 / 0.09);
  --overlay-glass: rgb(255 255 255 / 0.76);
  --glow-hero: rgb(4 120 87 / 0.12);
  --glow-section: rgb(4 120 87 / 0.065);
  --glow-control: rgb(4 120 87 / 0.2);
  --grid-line: rgb(17 24 39 / 0.055);
  --shadow-card: 0 10px 30px rgb(17 24 39 / 0.08);
  --shadow-elevated: 0 22px 60px rgb(17 24 39 / 0.13);
  --shadow-glow: 0 0 28px var(--glow-control);
}

```

`[data-theme]` is the source of truth. Bootstrap it before first paint, so the CSS
always has an explicit theme and never flashes the incorrect palette:

```html
<script is:inline>
  const storedTheme = localStorage.getItem('lean-ctx-theme');
  const systemTheme = matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  document.documentElement.dataset.theme = storedTheme || systemTheme;
</script>
```

Persist an explicit user choice in `localStorage`. The toggle is a 44px icon button
with `aria-pressed`, an accessible label, and no spinning/sunburst animation. Add
`.theme-transition` to `html` for 200ms only after a user-initiated toggle, then
remove it on `transitionend`; do not animate a first paint.

## 3. Typography and layout

### Type hierarchy

| Role | Token / family | Treatment | Use |
|---|---|---|---|
| Eyebrow | mono, `--text-caption` | uppercase, `--tracking-label`, muted | section label, `[ 01 ]`, figure metadata |
| Display | display, `--text-display` | 600, `--leading-display`, `--tracking-display` | hero only |
| H1 | display, `--text-h1` | 600, `--leading-heading`, `--tracking-display` | page title |
| H2 | display, `--text-h2` | 600, `--leading-heading`, `--tracking-heading` | major section |
| H3/H4 | display/sans | 600, 1.15–1.25 line height | feature/card heading |
| Lead | sans, `--text-body-lg` | 400, 1.65 | one concise argument |
| Body | sans, `--text-body` | 400, 1.65 | explanatory copy |
| Caption | mono, `--text-caption` | 500, 0.08em tracking | methodology, figure caption |
| Micro | mono, `--text-micro` | 500, 0.1em tracking | terminal/meta/status only |

Headlines use `text-wrap: balance` and max 12 words in the hero / 16 words in a
section. Only emphasise a maximum of two words per headline; prefer the full bright
colour, not a multicolour gradient. Body copy maxes at 62ch; metric values use
`font-variant-numeric: tabular-nums`.

### Spacing, containers, and grids

The 8px grid is strict. Use the scale above; one-off values require a component
reason (e.g. 44px touch target).

| Context | Block padding | Container | Grid |
|---|---:|---|---|
| Hero | `clamp(7rem, 14vw, 12rem)` top, `clamp(5rem, 10vw, 9rem)` bottom | wide | 2 columns only ≥1024px |
| Major narrative section | `clamp(5rem, 10vw, 10rem)` each side | medium | 1-col story or 2-col proof split |
| Proof/demo section | `clamp(4.5rem, 8vw, 8rem)` each side | wide | full-width terminal/demo |
| Supporting section | `clamp(4rem, 7vw, 6rem)` each side | medium | 2 or 3 cards |
| Footer | `clamp(3.5rem, 6vw, 6rem)` top, 32px bottom | wide | 2 columns ≥640px |

- `narrow` (760px): prose, FAQs, method notes.
- `medium` (960px): section copy plus a compact proof, 2-up feature grids.
- `wide` (1200px): hero, terminals, demonstrations, trust strips.
- Two columns: argument + dense artifact, or two comparable concepts. Never split
  a paragraph across columns.
- Three columns: equal, short cards only. Use them for the three cold-start costs,
  three trust pillars, or three metrics; not for long feature descriptions.
- Full width: terminals, source/file viewers, benchmarks, tabs, agent installer.

```css
html { scroll-behavior: smooth; -webkit-text-size-adjust: 100%; }
body {
  margin: 0; overflow-x: clip; background: var(--color-bg); color: var(--color-text);
  font-family: var(--font-sans); font-size: var(--text-body); line-height: var(--leading-body);
  -webkit-font-smoothing: antialiased; text-rendering: optimizeLegibility;
}
* { box-sizing: border-box; }
:where(a, button, input, select, summary) { -webkit-tap-highlight-color: transparent; }
:where(button, [role="button"], [role="tab"], summary, select) { min-height: 44px; }
:where(a, button, [role="button"], [role="tab"], summary, select):focus-visible {
  outline: 2px solid var(--color-accent); outline-offset: 3px;
}
.ds-container { width: min(100% - (var(--gutter) * 2), var(--container-medium)); margin-inline: auto; }
.ds-container--narrow { max-width: var(--container-narrow); }
.ds-container--wide { max-width: var(--container-wide); }
.ds-section { position: relative; padding-block: clamp(5rem, 10vw, 10rem); }
.ds-section--supporting { padding-block: clamp(4rem, 7vw, 6rem); }
.ds-section--proof { padding-block: clamp(4.5rem, 8vw, 8rem); }
.ds-split { display: grid; gap: clamp(2rem, 5vw, 5rem); align-items: center; }
.ds-grid-3 { display: grid; gap: var(--space-4); }
@media (min-width: 640px) { .ds-grid-3 { grid-template-columns: repeat(2, 1fr); } }
@media (min-width: 1024px) {
  .ds-split { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .ds-grid-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--space-5); }
}
```

## 4. Component language

The following base classes are designed for component-scoped styles or dedicated
files. `data-state`, `aria-selected`, and native state attributes carry state; do not
create modifier classes for each state.

### Hero, background, and section transition

**Recommendation: Option A — CSS grid/dot field with a single emerald aura.** It
fits a context engine better than generic gradient orbs, supports real terminals,
and remains nearly invisible. It uses two static pseudo-elements plus one 18-second
opacity/position drift—no canvas, particle JS, or per-frame DOM work. Do not use code
rain (too decorative) or low-poly mesh (too SaaS-generic). Option D is reserved for
campaign pages only.

The hero has an editorial full-viewport minimum, headline left and real terminal
right on desktop. The aura must sit behind content at low alpha. Following sections
fade through an emerald-tinted 1px horizon rather than use hard dividers.

```css
.hero {
  isolation: isolate; position: relative; overflow: clip;
  min-block-size: min(900px, 100svh);
  padding-block: clamp(7rem, 14vw, 12rem) clamp(5rem, 10vw, 9rem);
}
.hero::before {
  content: ""; position: absolute; inset: 0; z-index: -2; pointer-events: none;
  background-image: radial-gradient(var(--grid-line) 1px, transparent 1px);
  background-size: 24px 24px; mask-image: linear-gradient(to bottom, black, transparent 86%);
}
.hero::after {
  content: ""; position: absolute; z-index: -1; pointer-events: none;
  inline-size: min(70vw, 58rem); block-size: min(70vw, 58rem); inset: -28rem auto auto 42%;
  border-radius: 50%; filter: blur(18px); opacity: .9;
  background: radial-gradient(circle, var(--glow-hero), transparent 68%);
  animation: hero-aura var(--duration-slow) var(--ease-in-out) infinite alternate;
}
.hero__inner { display: grid; align-items: center; gap: clamp(2.5rem, 6vw, 6rem); }
.hero__copy { max-inline-size: 43rem; }
.hero__title { margin: var(--space-4) 0 0; color: var(--color-text-bright); font: var(--weight-semibold) var(--text-display)/var(--leading-display) var(--font-display); letter-spacing: var(--tracking-display); text-wrap: balance; }
.hero__lead { max-inline-size: 40rem; margin: var(--space-5) 0 0; font-size: var(--text-body-lg); }
.section-transition::before { content: ""; position: absolute; inset: 0 0 auto; block-size: 1px; background: linear-gradient(90deg, transparent, var(--glow-section), transparent); }
@media (min-width: 1024px) { .hero__inner { grid-template-columns: minmax(0, 1fr) minmax(25rem, .9fr); } }
@media (max-width: 639px) { .hero { min-block-size: auto; } .hero::after { inset-inline-start: 18%; } }
```

### Terminal chrome

The terminal is the principal product artifact: a hairline shell, quiet dots, a
mono title, and genuine code/output. It is never styled as a fake IDE.

```css
.terminal {
  overflow: hidden; border: var(--border-default); border-radius: var(--radius-lg);
  background: var(--color-surface); box-shadow: var(--shadow-card);
}
.terminal__bar { display: flex; align-items: center; gap: var(--space-2); min-block-size: 44px; padding-inline: var(--space-4); background: var(--color-surface-2); border-block-end: var(--border-default); }
.terminal__dot { inline-size: 8px; block-size: 8px; border-radius: 50%; background: var(--color-text-muted); opacity: .45; }
.terminal__dot[data-live] { opacity: 1; background: var(--color-accent); box-shadow: 0 0 10px var(--glow-control); }
.terminal__title { margin-inline: var(--space-2) auto; color: var(--color-text-muted); font: var(--weight-medium) var(--text-micro)/1 var(--font-mono); letter-spacing: .08em; text-transform: uppercase; }
.terminal__body { overflow-x: auto; padding: clamp(1rem, 2vw, 1.5rem); background: var(--color-surface); color: var(--color-text); font: var(--weight-medium) var(--text-small)/1.7 var(--font-mono); }
.terminal__body [data-accent] { color: var(--color-accent); }
@media (max-width: 639px) { .terminal { border-radius: var(--radius-md); } .terminal__body { margin-inline: 0; font-size: .8125rem; } }
```

### Cards

Cards are proof containers, not default page framing. Feature, trust pillar, and
agent-selector cards share a calm resting state; only interactive cards lift.

```css
.card {
  display: flex; flex-direction: column; min-block-size: 100%; padding: var(--space-6);
  border: var(--border-default); border-radius: var(--radius-lg); background: var(--color-surface);
  box-shadow: 0 1px 0 var(--overlay-subtle) inset;
  transition: transform var(--duration-fast) var(--ease-out), border-color var(--duration-fast), box-shadow var(--duration-fast), background-color var(--duration-fast);
}
a.card:hover, button.card:hover, .card[data-interactive]:hover { transform: translateY(-2px); border-color: color-mix(in srgb, var(--color-accent) 42%, var(--color-border)); box-shadow: var(--shadow-card), 0 0 24px var(--glow-section); }
.card__eyebrow { margin-block-end: var(--space-5); }
.card__title { margin: 0; color: var(--color-text-bright); font: var(--weight-semibold) var(--text-h4)/1.2 var(--font-display); letter-spacing: var(--tracking-tight); }
.card__body { margin: var(--space-3) 0 0; color: var(--color-text); font-size: var(--text-small); }
```

### Buttons, text links, and the agent selector

Primary is an emerald install action. Secondary is a bordered neutral action. A
text link is for tertiary navigation and always retains visible text/underline.
No button is smaller than 44px tall. The selector is one composite primary action,
not a CTA plus unrelated dropdown.

```css
.button { display: inline-flex; align-items: center; justify-content: center; gap: var(--space-2); min-block-size: 44px; padding: 0 var(--space-5); border: 1px solid transparent; border-radius: var(--radius-md); font: var(--weight-semibold) var(--text-small)/1 var(--font-sans); text-decoration: none; cursor: pointer; transition: transform var(--duration-fast) var(--ease-out), background-color var(--duration-fast), border-color var(--duration-fast), box-shadow var(--duration-fast), color var(--duration-fast); }
.button--primary { background: var(--color-accent); color: var(--color-bg); box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-accent) 30%, transparent); }
.button--primary:hover { background: var(--color-accent-hover); transform: scale(1.02); box-shadow: var(--shadow-glow); }
.button--primary:active { background: var(--color-accent-pressed); transform: scale(.98); }
.button--secondary { border-color: var(--color-border-strong); background: transparent; color: var(--color-text-bright); }
.button--secondary:hover { border-color: var(--color-text-muted); background: var(--overlay-hover); transform: translateY(-1px); }
.button--secondary:active { background: var(--overlay-pressed); transform: translateY(0); }
.text-link { color: var(--color-text-bright); font-weight: var(--weight-medium); text-decoration: none; background: linear-gradient(var(--color-accent), var(--color-accent)) 0 100% / 0 1px no-repeat; transition: background-size var(--duration-fast) var(--ease-out), color var(--duration-fast); }
.text-link:hover { color: var(--color-accent-hover); background-size: 100% 1px; }
.agent-selector { display: inline-grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; min-block-size: 52px; border: 1px solid var(--color-accent); border-radius: var(--radius-md); overflow: clip; background: var(--color-accent); color: var(--color-bg); box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-accent) 25%, transparent); }
.agent-selector:hover { background: var(--color-accent-hover); box-shadow: var(--shadow-glow); }
.agent-selector__label, .agent-selector__current { padding-inline: var(--space-4); font-weight: var(--weight-semibold); }
.agent-selector__current { min-inline-size: 8rem; border-inline-start: 1px solid rgb(0 0 0 / .14); }
.agent-selector__chevron { padding-inline: var(--space-3); }
.agent-selector [role="listbox"] { border: var(--border-default); border-radius: var(--radius-md); background: var(--color-surface-elevated); color: var(--color-text-bright); box-shadow: var(--shadow-elevated); }
@media (max-width: 639px) { .agent-selector { display: grid; inline-size: 100%; grid-template-columns: auto 1fr auto; } .agent-selector__current { min-inline-size: 0; } }
```

### Tabs / segmented control and copy button

Use tabs only to switch a single proof’s mutually exclusive views (`full`, `map`,
`signatures`). The active indicator moves by transform; content panels do not slide.

```css
.tabs { position: relative; display: inline-flex; max-inline-size: 100%; overflow-x: auto; border: var(--border-default); border-radius: var(--radius-md); background: var(--color-surface-2); }
.tabs__tab { position: relative; z-index: 1; flex: 0 0 auto; min-block-size: 40px; padding-inline: var(--space-4); border: 0; background: transparent; color: var(--color-text-muted); font: var(--weight-medium) var(--text-small)/1 var(--font-mono); cursor: pointer; }
.tabs__tab:hover { color: var(--color-text-bright); }
.tabs__tab[aria-selected="true"] { color: var(--color-accent); }
.tabs__indicator { position: absolute; inset: auto 0 0; block-size: 2px; background: var(--color-accent); transform: translateX(var(--tab-offset)) scaleX(var(--tab-scale)); transform-origin: left; transition: transform var(--duration-fast) var(--ease-out); }
.copy-button { display: inline-grid; place-items: center; inline-size: 36px; block-size: 36px; border: var(--border-default); border-radius: var(--radius-sm); background: transparent; color: var(--color-text-muted); cursor: pointer; transition: color var(--duration-instant), border-color var(--duration-instant), background-color var(--duration-instant); }
.copy-button:hover { color: var(--color-text-bright); border-color: var(--color-border-strong); background: var(--overlay-hover); }
.copy-button[data-copied="true"] { color: var(--color-accent); border-color: var(--color-accent); }
.copy-button svg[data-check] { opacity: 0; transform: scale(.65); transition: opacity var(--duration-fast), transform var(--duration-fast) var(--ease-out); }
.copy-button[data-copied="true"] svg[data-check] { opacity: 1; transform: scale(1); animation: check-in var(--duration-fast) var(--ease-out) both; }
```

### Eyebrow, FAQ, trust strip, navigation, footer

```css
.eyebrow { display: inline-flex; align-items: center; gap: var(--space-2); margin: 0; color: var(--color-text-muted); font: var(--weight-medium) var(--text-caption)/1 var(--font-mono); letter-spacing: var(--tracking-label); text-transform: uppercase; }
.eyebrow::before { content: ""; inline-size: 18px; block-size: 1px; background: var(--color-accent); }
.eyebrow[data-number]::before { content: attr(data-number); inline-size: auto; background: none; color: var(--color-accent); }
.faq { border-block-start: var(--border-default); }
.faq__item { border-block-end: var(--border-default); }
.faq__question { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); inline-size: 100%; min-block-size: 64px; padding: var(--space-4) 0; border: 0; background: none; color: var(--color-text-bright); font: var(--weight-medium) var(--text-body)/1.35 var(--font-sans); text-align: start; cursor: pointer; }
.faq__icon { color: var(--color-accent); transition: transform var(--duration-fast) var(--ease-out); }
.faq__question[aria-expanded="true"] .faq__icon { transform: rotate(45deg); }
.faq__answer { display: grid; grid-template-rows: 0fr; transition: grid-template-rows var(--duration-base) var(--ease-out); }
.faq__answer[data-open="true"] { grid-template-rows: 1fr; }
.faq__answer > div { overflow: hidden; padding-inline-end: var(--space-8); }
.faq__answer p { margin: 0; padding-block: 0 var(--space-5); }
.trust-strip { display: grid; gap: 1px; border: var(--border-default); border-radius: var(--radius-lg); overflow: clip; background: var(--color-border); }
.trust-strip__item { min-block-size: 104px; padding: var(--space-5); background: var(--color-surface); }
.trust-strip__value { display: block; color: var(--color-text-bright); font: var(--weight-bold) var(--text-h3)/1 var(--font-display); letter-spacing: var(--tracking-heading); font-variant-numeric: tabular-nums; }
.trust-strip__label { display: block; margin-top: var(--space-2); color: var(--color-text-muted); font: var(--weight-medium) var(--text-caption)/1.4 var(--font-mono); letter-spacing: .06em; }
.site-header { position: sticky; inset-block-start: 0; z-index: 20; border-block-end: 1px solid transparent; background: color-mix(in srgb, var(--overlay-glass) 88%, transparent); backdrop-filter: blur(16px); transition: border-color var(--duration-fast), background-color var(--duration-fast); }
.site-header[data-scrolled="true"] { border-color: var(--color-border); }
.site-nav { display: flex; align-items: center; justify-content: space-between; min-block-size: 68px; gap: var(--space-5); }
.site-nav__links { display: flex; align-items: center; gap: var(--space-5); }
.site-nav__link { min-block-size: 44px; display: inline-flex; align-items: center; color: var(--color-text); font: var(--weight-medium) var(--text-small)/1 var(--font-sans); text-decoration: none; background: linear-gradient(var(--color-accent), var(--color-accent)) 0 calc(100% - 9px) / 0 1px no-repeat; transition: color var(--duration-fast), background-size var(--duration-fast) var(--ease-out); }
.site-nav__link:hover, .site-nav__link[aria-current="page"] { color: var(--color-text-bright); background-size: 100% 1px; }
.site-footer { border-block-start: var(--border-default); background: linear-gradient(180deg, color-mix(in srgb, var(--color-surface) 65%, var(--color-bg)), var(--color-bg)); }
.site-footer__grid { display: grid; gap: var(--space-8); }
.site-footer__links { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-7) var(--space-5); }
.site-footer__heading { margin: 0 0 var(--space-3); color: var(--color-text-bright); font: var(--weight-semibold) var(--text-small)/1.2 var(--font-sans); }
.site-footer__link { color: var(--color-text-muted); font-size: var(--text-small); text-decoration: none; }
.site-footer__link:hover { color: var(--color-accent-hover); }
@media (min-width: 640px) { .trust-strip { grid-template-columns: repeat(4, 1fr); } .site-footer__grid { grid-template-columns: minmax(0, .8fr) minmax(0, 1.2fr); } }
@media (max-width: 767px) { .site-nav__links { display: none; } .site-nav__menu { display: grid; place-items: center; inline-size: 44px; block-size: 44px; } }
```

Navigation has only **Product**, **Docs**, **GitHub**, theme toggle, and the install
selector/CTA. Avoid the current mega-menu. On mobile, the menu opens a simple dialog
with the same four destinations and the full-width selector. Footer is two columns:
brand/proof/contact + product/docs/legal links. It does not repeat the old five-column
information architecture.

## 5. Motion system

### Motion contract

| Pattern | Initial → final | Duration / easing | When |
|---|---|---|---|
| `fade-up` | opacity 0, Y 24px → opacity 1, Y 0 | 600ms `--ease-out` | default scroll reveal |
| `fade-in` | opacity 0 → 1 | 400ms `--ease-out` | subtle labels, dividers |
| `slide-in-left` | opacity 0, X -24px → 0 | 600ms `--ease-out` | desktop split copy |
| `slide-in-right` | opacity 0, X 24px → 0 | 600ms `--ease-out` | desktop terminal/proof |
| child stagger | same reveal | 80ms desktop / 100ms mobile per child, max 480ms | hero and compact groups |
| button hover | scale 1 → 1.02 | 200ms | primary only |
| card hover | Y 0 → -2px | 200ms | interactive cards only |
| tab indicator | transform position | 200ms | selection only |
| nav underline | background size 0 → 100% | 200ms | hover/current link |
| counter | integer 0 → final | 800ms, ease-out | once at 15% visibility |
| route | old/new opacity crossfade | 180ms | only Astro view transitions, if enabled |

Never animate geometry that causes layout/repaint during scroll. Animatable properties
are opacity, transform, and the small control colour/border/shadow changes above. Add
`will-change: opacity, transform` only to elements while `data-motion="pending"` or
`data-motion="visible"`; remove it when the transition ends. No animation library.

```css
[data-reveal] { opacity: 0; will-change: opacity, transform; }
[data-reveal="fade-up"] { transform: translateY(24px); }
[data-reveal="fade-in"] { transform: none; }
[data-reveal="slide-in-left"] { transform: translateX(-24px); }
[data-reveal="slide-in-right"] { transform: translateX(24px); }
[data-reveal].is-visible { opacity: 1; transform: translate3d(0, 0, 0); transition: opacity var(--duration-reveal) var(--ease-out), transform var(--duration-reveal) var(--ease-out); transition-delay: var(--reveal-delay, 0ms); }
[data-reveal].is-settled { will-change: auto; }
[data-counter] { font-variant-numeric: tabular-nums; }
@keyframes hero-aura { from { transform: translate3d(-2%, -1%, 0) scale(.96); opacity: .72; } to { transform: translate3d(3%, 2%, 0) scale(1.04); opacity: 1; } }
@keyframes check-in { from { opacity: 0; transform: scale(.65) rotate(-10deg); } to { opacity: 1; transform: scale(1) rotate(0); } }
@media (prefers-reduced-motion: reduce) {
  html { scroll-behavior: auto; }
  *, *::before, *::after { animation-duration: .01ms !important; animation-iteration-count: 1 !important; scroll-behavior: auto !important; transition-duration: .01ms !important; }
  [data-reveal] { opacity: 1; transform: none; will-change: auto; }
}
```

### One observer module

Load this once from `BaseLayout.astro` as a deferred module. It observes at the
requested `threshold: 0.15`, reveals each element once, calculates bounded stagger,
and counts only numbers expressly marked `data-counter`. The counter value must be in
`data-counter-to`; formatting must not be inferred from visible text.

```html
<script>
  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const revealItems = document.querySelectorAll('[data-reveal]');

  function reveal(element) {
    element.classList.add('is-visible');
    element.addEventListener('transitionend', () => element.classList.add('is-settled'), { once: true });
  }

  function count(element) {
    const end = Number(element.dataset.counterTo);
    if (!Number.isFinite(end)) return;
    if (reduced) { element.textContent = new Intl.NumberFormat().format(end); return; }
    const start = performance.now();
    const duration = 800;
    const tick = (now) => {
      const progress = Math.min((now - start) / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      element.textContent = new Intl.NumberFormat().format(Math.round(end * eased));
      if (progress < 1) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  if (reduced) {
    revealItems.forEach((item) => item.classList.add('is-visible', 'is-settled'));
    document.querySelectorAll('[data-counter]').forEach(count);
  } else {
    document.querySelectorAll('[data-reveal-group]').forEach((group) => {
      group.querySelectorAll(':scope > [data-reveal]').forEach((item, index) => {
        item.style.setProperty('--reveal-delay', `${Math.min(index * 80, 480)}ms`);
      });
    });
    const observer = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        reveal(entry.target);
        entry.target.querySelectorAll('[data-counter]').forEach(count);
        observer.unobserve(entry.target);
      });
    }, { threshold: 0.15 });
    revealItems.forEach((item) => observer.observe(item));
    document.querySelectorAll('[data-counter]').forEach((item) => observer.observe(item));
  }
</script>
```

For counters located inside a revealed parent, mark the parent as the reveal target
and omit `data-counter` from the observer query to prevent the second `observe()`.
Alternatively, use a dedicated `counterObserver`; never count twice. The implementation
must retain a `WeakSet` of counted elements if an element can be both parent and target.

## 6. Responsive strategy

| Range | Rules |
|---|---|
| Mobile `<640px` | one column; 20px gutter; hero uses `--text-display` clamp floor; terminal comes after copy; selector full width; controls horizontally scroll rather than shrink; menu is dialog; no card grid below 44px targets. |
| Tablet `640–1023px` | 24–32px gutter; trust strip can be 2×2; two-card grids allowed; hero/proof stays stacked unless terminal is compact. |
| Desktop `≥1024px` | wide 1200px hero; split copy and terminal; 3-card grid; header nav visible; terminal may overhang its copy by max 32px only. |

- Hero mobile order: eyebrow → headline → lead → full-width agent selector → proof
  line → terminal. Do not place terminal ahead of the install path.
- Terminal code stays `0.8125rem` minimum; preserve horizontal scroll and a visible
  fade/scroll affordance. Do not scale code to fit.
- Agent selector dropdown is a full-width, native-like popover on mobile; each agent
  option is 48px minimum and shows its install command after selection.
- Tabs scroll inline; never wrap labels to two lines or turn them into a dropdown.
- Hover styles are additive only; all important state is available through focus,
  active, and selected styles on touch.

## 7. CSS architecture and adoption

```text
src/styles/
  global.css                 # import, @theme, theme assignments, reset, generic a11y
  components/
    hero.css                 # hero + background + transition
    terminal.css             # terminal and code controls
    controls.css             # buttons, selector, tabs, copy button
    editorial.css            # eyebrow, cards, trust, FAQ
    navigation.css           # header, mobile dialog, footer
    motion.css               # reveal/keyframes/reduced-motion
  pages/
    index.css                # composition only; no new global tokens
```

- Use semantic component classes: `.hero__title`, `.terminal__bar`, `.card__title`.
  State lives in `data-*`, `aria-*`, and native attributes: `[data-live]`,
  `[aria-selected="true"]`, `[data-copied="true"]`.
- Retain only low-level layout utilities (`.ds-container`, `.ds-split`, `.ds-grid-3`,
  `.text-balance`). Tailwind may provide structural utilities in markup, but never
  becomes the token vocabulary and never carries arbitrary colour/spacing values.
- Components import or scope their matching stylesheet; pages compose components.
  `index.css` must not restyle generic cards/buttons or redefine tokens.
- Remove duplicate legacy helpers (`gradient-text`, high-opacity global ASCII,
  coloured UI accents, mega-menu styles) as their v2 components replace them.
- Test dark/light and reduced motion in every component story/page. Keyboard focus,
  44px hit areas, and 4.5:1 body/interactive contrast are release criteria.

## 8. Implementation checklist

1. Replace token blocks and add `data-theme` bootstrap/toggle before styling pages.
2. Split global component CSS into the files above; ship hero, controls, and terminal
   first so the first viewport is coherent.
3. Implement one reusable reveal/count module and use it only for meaningful entry
   states and trust metrics.
4. Rebuild `Header.astro` around four destinations and `Footer.astro` as two columns.
5. Recompose the home page around Hero → Problem → Solution → Proof → Trust → Install;
   every percentage includes a methodology link and every demo uses real output.
