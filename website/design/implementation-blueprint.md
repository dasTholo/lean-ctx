# lean-ctx Website Redesign — Implementation Blueprint

## Outcome and delivery rules

Rebuild the marketing experience around one clear promise: lean-ctx is the
context layer for coding agents. The homepage proves that claim with real,
inspectable product output; secondary pages add depth only after the visitor
understands the core product.

This plan is intentionally English-first. Existing translated routes remain
online until an explicit localized replacement and redirect have passed review.
“Remove” below means remove from navigation and retire the template only after
the redirect inventory has been approved; it never means silently returning a
404. Keep factual claims in data, never hard-code a metric into a component.

### Current-state constraints informing the plan

- `IndexPage.astro` is 1,061 lines and combines the entire current narrative.
- `Header.astro` is 776 lines and owns a multi-column mega-menu; `Footer.astro`
  is 377 lines with five link columns.
- `global.css` is 1,518 lines and `index.css` is 689 lines; V2 must add scoped,
  reusable primitives rather than a third unbounded page stylesheet.
- Astro already supplies Tailwind v4, sitemap generation, light/dark themes,
  locale routing, and a substantial redirect map. No new UI framework is needed.

### Dependency and rollout model

```text
E1 design foundation
  ├─ E2 homepage components and composition
  ├─ E3 navigation / redirects / sitemap
  └─ E4 secondary page shells
       └─ E5 docs information architecture

E2 claims + install data ──► every marketing page
E3 redirect inventory ─────► template retirement and sitemap release
```

Use a short-lived `site-v2` feature branch (or equivalent deployment preview),
not a permanent runtime feature flag. Merge and release English V2 only after
the quality gates below; localizations follow as a separate release train.

## Epic roadmap

| Epic | Schedule | Outcome | Exit dependency |
|---|---:|---|---|
| 1. Design Foundation | Week 1 | Tokens, motion, shared editorial/product primitives | Component visual QA |
| 2. Homepage Rebuild | Weeks 2–3 | New proof-led homepage and focused conversion path | E1 and content/data contracts |
| 3. Content Consolidation | Week 3 | Canonical IA, redirects, lean navigation/footer, sitemap | E2 canonical paths |
| 4. Secondary Pages | Week 4 | Focused How It Works, Pricing, Enterprise, About pages | E1, E2 data, E3 IA |
| 5. Documentation Restructure | Week 5 | Getting-started-first docs and agent quickstarts | E3 canonical docs paths |

---

# Epic 1: Design Foundation (Week 1)

**Goal:** establish the durable V2 vocabulary before any page migration. All
components must be semantic HTML first, progressively enhanced JavaScript
second, and visually correct in both themes.

### Ticket: [EPIC-1.1] Design Token Migration
**Type:** design  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** None

**Acceptance criteria:**
- [ ] Preserve existing emerald, surface, typography, and theme variables while
      adding named V2 tokens for editorial sections, terminal surfaces, borders,
      glows, gradients, focus treatment, and elevation.
- [ ] Add a small motion token set: entrance distance, durations, easing, and
      reusable transition utilities; no indigo/purple UI accent is introduced.
- [ ] Define `@keyframes` only for reusable motion (reveal, terminal cursor,
      copy-success, subtle ambient drift) and disable/reduce them for
      `prefers-reduced-motion`.
- [ ] Add V2 layout utilities for 900px prose and 1200px demo widths, section
      rhythm, eyebrow/caption hierarchy, and accessible focus-visible states.
- [ ] V2 tokens work under explicit dark, explicit light, and system themes.

**Technical notes:** Extend the existing `@theme` and theme-variable overrides;
do not mass-rename every legacy selector in this ticket. Keep visual-data colours
under `--viz-*`; emerald remains the only interactive accent.

**Files to create/modify:**
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-1.2] ScrollReveal System
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.1

**Acceptance criteria:**
- [ ] Provide a wrapper/component that accepts `direction` (`up`, `down`,
      `left`, `right`, `none`), `delay`, and `stagger` props.
- [ ] Reveal only after IntersectionObserver entry; initial markup remains
      readable when JavaScript is unavailable.
- [ ] Respect `prefers-reduced-motion` by showing content immediately with no
      movement; no animation blocks keyboard navigation.
- [ ] Disconnect observers once revealed and avoid one observer per child when
      a group can be observed together.

**Technical notes:** Put behavioural JS in the component where Astro can scope
it; global CSS supplies only stable class/keyframe contracts. Make initialization
idempotent for Astro client navigation or repeated component instances.

**Files to create/modify:**
- Create `website/src/components/ui/ScrollReveal.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-1.3] TerminalPanel Component
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.1

**Acceptance criteria:**
- [ ] Render reusable terminal chrome with title, optional language badge,
      labelled body slot, and optional copy action.
- [ ] Provide dark terminal surface, three non-semantic chrome dots, robust
      horizontal overflow for narrow screens, and a textual accessible label.
- [ ] Copy action delegates to `CopyCommand` when present and never duplicates
      clipboard JavaScript.

**Technical notes:** Replace or wrap existing `TerminalChrome.astro` only after
all its callers are migrated; do not break docs in this foundation ticket.

**Files to create/modify:**
- Create `website/src/components/ui/TerminalPanel.astro`
- Modify `website/src/components/TerminalChrome.astro` (compatibility adapter, if needed)
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-1.4] EyebrowLabel Component
**Type:** design  
**Priority:** P1 (critical)  
**Estimated effort:** S (2–4h)  
**Dependencies:** EPIC-1.1

**Acceptance criteria:**
- [ ] Render `[ 01 ]  LABEL` with caller-supplied number/label and correct
      tracking, small-caps/uppercase, and mono-caption styling.
- [ ] Permit a no-number variant for the hero without duplicate CSS.
- [ ] Remain legible at 320px and in light theme.

**Technical notes:** Use a semantic inline element; the number is presentational
unless the surrounding heading provides a meaningful ordered sequence.

**Files to create/modify:**
- Create `website/src/components/ui/EyebrowLabel.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-1.5] CopyCommand Component
**Type:** feature  
**Priority:** P1 (critical)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.1

**Acceptance criteria:**
- [ ] Render code/command text and an accessible copy button; copy text is
      passed as data, never scraped from decorated DOM.
- [ ] Click copies with `navigator.clipboard`, changes to a checkmark and
      “Copied!” feedback, then returns to its resting label.
- [ ] Clipboard failure exposes a short, accessible failure state without
      throwing; keyboard and touch paths work.
- [ ] Motion follows the reduced-motion contract.

**Technical notes:** One scoped, idempotent event handler can service all V2
instances via data attributes. Do not introduce a client framework.

**Files to create/modify:**
- Create `website/src/components/ui/CopyCommand.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-1.6] BasisLabel Component
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** S (2–4h)  
**Dependencies:** EPIC-1.1

**Acceptance criteria:**
- [ ] Render a compact claim basis label with measurement, method/repository,
      tokenizer, date, and optional methodology link.
- [ ] Consume a typed claim object/ID from the claims registry; no display metric
      can supply a conflicting literal basis.
- [ ] Methodology links are descriptive and keyboard visible.

**Technical notes:** Keep the component data-only. Schema validation of the
registry belongs to EPIC-2.1.

**Files to create/modify:**
- Create `website/src/components/ui/BasisLabel.astro`
- Modify `website/src/styles/global.css`

**Epic 1 exit:** component preview page or temporary fixture verifies each state
in light/dark and reduced-motion; no homepage route is replaced yet.

---

# Epic 2: Homepage Rebuild (Weeks 2–3)

**Goal:** replace feature-inventory marketing with the narrative in
`website/design/narrative-sales-concept.md`: promise → cold-start problem →
context-engineered solution → inspectable proof → trust → install → FAQ.

### Ticket: [EPIC-2.1] Claims Registry and Validator
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.6

**Acceptance criteria:**
- [ ] Create a typed English claims registry covering every homepage number,
      including value, label, basis, tokenizer, date, methodology URL, owner,
      and review date.
- [ ] Create validation that fails the site check when fields are missing,
      duplicate IDs exist, dates are invalid, or a component references an
      unknown claim.
- [ ] Demo-only values are explicitly marked `illustrative` and never rendered
      as aggregate product results.

**Technical notes:** JSON is the requested source of truth; add a small TypeScript
loader/validator rather than trusting raw `any`. Confirm real benchmark evidence
with product owners before populating production values.

**Files to create/modify:**
- Create `website/src/data/claims.json`
- Create `website/src/lib/claims.ts`
- Create `website/scripts/validate-claims.mjs`
- Modify `website/package.json`

### Ticket: [EPIC-2.2] Agent Install Data Contract
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** None

**Acceptance criteria:**
- [ ] Define Cursor, Claude Code, Codex, Windsurf, Copilot, and Other MCP
      client with display name, install command, verification command, docs URL,
      and accessibility label.
- [ ] Validate that commands are non-empty and selected IDs are stable.
- [ ] Keep commands in one data file used by hero, install section, and future
      quickstarts.

**Technical notes:** Commands require maintainer approval against current product
documentation; do not invent integration syntax. Store no shell code in locale
translation files.

**Files to create/modify:**
- Create `website/src/data/agent-installs.ts`
- Create `website/src/lib/agent-installs.ts`
- Modify `website/package.json`

### Ticket: [EPIC-2.3] Homepage V2 Content Contract
**Type:** content  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-2.1, EPIC-2.2

**Acceptance criteria:**
- [ ] Extract approved English copy for all seven homepage sections from the
      narrative sales concept; one claim/proof/action per viewport.
- [ ] Tag each metric reference with a claims ID and each install CTA with an
      agent data ID.
- [ ] Add title, meta description, OpenGraph text, and six FAQ Q&A entries;
      remove unsubstantiated social/energy claims from the V2 hero.

**Technical notes:** This is the only ticket allowed to decide wording. Components
receive text/data props, so localization can occur later without structural edits.

**Files to create/modify:**
- Create `website/src/data/homepage-v2.ts`
- Modify `website/src/i18n/translations.ts` (English keys only, if retained)
- Modify `website/src/lib/positioning.ts`

### Ticket: [EPIC-2.4] HeaderV2 Focused Navigation
**Type:** refactor  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-2.2, EPIC-2.3

**Acceptance criteria:**
- [ ] Provide at most five primary items: How it works, Docs, Pricing,
      Enterprise, and emerald Install.
- [ ] Retain GitHub, theme toggle, and search as compact utilities; desktop and
      mobile menus expose equivalent links and correct focus trapping/escape.
- [ ] Install opens/scrolls to the shared selected-agent install experience;
      obsolete mega-menu categories are absent from V2 navigation.
- [ ] Existing non-V2 pages retain functional navigation until Epic 3 redirects
      land.

**Technical notes:** Build `HeaderV2` beside the legacy component first. Avoid
copying the 776-line mega-menu script; use native disclosure/details semantics
where possible.

**Files to create/modify:**
- Create `website/src/components/HeaderV2.astro`
- Modify `website/src/layouts/BaseLayout.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.5] FooterV2 Two-Column Information Architecture
**Type:** refactor  
**Priority:** P1 (critical)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-2.3

**Acceptance criteria:**
- [ ] Render only Product (How it works, Pricing, Enterprise, Changelog) and
      Resources (Documentation, GitHub, Discord, Security) link columns.
- [ ] Retain Apache-2.0, Made in Switzerland, current-year copyright, and
      required legal links without promoting retired marketing routes.
- [ ] Stack accessibly and legibly on mobile.

**Technical notes:** Use a new component during transition. Legal links can live
in the bottom row rather than expanding the content architecture.

**Files to create/modify:**
- Create `website/src/components/FooterV2.astro`
- Modify `website/src/layouts/BaseLayout.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.6] HeroV2 — Promise, Agent Selector, and Live Terminal
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-1.2, EPIC-1.3, EPIC-1.4, EPIC-1.5, EPIC-2.1, EPIC-2.2, EPIC-2.3

**Acceptance criteria:**
- [ ] Deliver a full-viewport hero whose copy is “Give your agents context —
      not clutter.” and whose sole button is Install for selected agent.
- [ ] Agent selection persists in localStorage with a safe default and updates
      all V2 selector instances without a page reload.
- [ ] Use an emerald, restrained dark-editorial background and real/approved
      terminal output; no decorative stock art, network fetch, or blocking JS.
- [ ] Include the trust strip and a basis-linked micro-proof without competing
      Discord/GitHub CTAs.
- [ ] Meet keyboard, reduced-motion, 320px, light-theme, and no-JS content
      requirements.

**Technical notes:** Separate the selector/state helper from Hero markup so the
Install section can consume it. The terminal sequence must be deterministic,
must not auto-replay indefinitely, and should appear fully when motion is reduced.

**Files to create/modify:**
- Create `website/src/components/home/HeroV2.astro`
- Create `website/src/components/home/AgentSelector.astro`
- Create `website/src/lib/agent-selection.ts`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.7] Cold-Start Problem Exhibits
**Type:** feature  
**Priority:** P1 (critical)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.2, EPIC-1.4, EPIC-2.3

**Acceptance criteria:**
- [ ] Render the “Every new agent starts by forgetting” section with exactly
      three responsive exhibits: Re-reading, Re-discovering, Re-explaining.
- [ ] Each exhibit communicates one failure mode with inspectable textual or
      terminal-style evidence, not generic icons.
- [ ] Use ScrollReveal without hiding meaningful content before JS runs.

**Technical notes:** Keep each exhibit data-driven from `homepage-v2.ts`; avoid
reintroducing the old broad audience/product grid.

**Files to create/modify:**
- Create `website/src/components/home/ProblemSection.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.8] Context-Engineered Solution Section
**Type:** feature  
**Priority:** P1 (critical)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-1.2, EPIC-1.3, EPIC-1.4, EPIC-2.3

**Acceptance criteria:**
- [ ] Explain four product capabilities: structure-aware reads, persistent
      knowledge, coordinated agents, and proof/governance.
- [ ] Render at most three cards per row; use editorial numbered treatments and
      a tangible terminal/output artifact where relevant.
- [ ] Close with “Token savings are the receipt. Better agent judgment is the
      product.” and no competing CTA.

**Technical notes:** Capability wording must map to the deep feature catalog but
avoid a 13-category inventory. Link deeper concepts only from secondary/docs pages.

**Files to create/modify:**
- Create `website/src/components/home/SolutionSection.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.9] Proof Demo Data and Claim Wiring
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.6, EPIC-2.1, EPIC-2.3

**Acceptance criteria:**
- [ ] Define one legally reviewable source file and three views (`full`, `map`,
      `signatures`) with source text, displayed output, token totals, retained
      symbols, and an expand target.
- [ ] Associate every displayed statistic with a claim ID and render its basis.
- [ ] Mark any simulated source/output as a representative fixture and provide a
      link to reproducible benchmark methodology.

**Technical notes:** Do not use a live API or runtime compression in marketing.
Keep fixture data in a typed module so token math can be unit-tested.

**Files to create/modify:**
- Create `website/src/data/proof-demo.ts`
- Create `website/src/lib/proof-demo.ts`
- Modify `website/src/data/claims.json`

### Ticket: [EPIC-2.10] ProofSection Mode Switcher
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-1.3, EPIC-1.5, EPIC-1.6, EPIC-2.9

**Acceptance criteria:**
- [ ] Build split original/compressed terminal panels with accessible tabs for
      full, map, and signatures; selected state follows keyboard tab behaviour.
- [ ] Update output, token rail, retained-symbol list, basis label, and Expand
      action atomically when a mode changes.
- [ ] Expand reveals approved fuller context inline without losing selection or
      causing layout shift; copy works for each shown command/output.
- [ ] No active tab content is inaccessible to screen readers; all functionality
      remains usable with JavaScript disabled by showing a default view.

**Technical notes:** Use a small scoped script, native buttons, `aria-selected`,
and `aria-controls`; avoid client-side hydration libraries. Budget CSS to avoid
large repainting gradients behind the terminals.

**Files to create/modify:**
- Create `website/src/components/home/ProofSection.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.11] Trust Section and Evidence Links
**Type:** feature  
**Priority:** P1 (critical)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.4, EPIC-1.6, EPIC-2.3

**Acceptance criteria:**
- [ ] Present three pillars: local-first/zero telemetry, inspectable deterministic
      compression, and signed hash-chained evidence.
- [ ] Every deeper assertion links to an existing security, benchmark, or docs
      source; present-tense claims only.
- [ ] Include one subordinate Enterprise link and the approved trust line.

**Technical notes:** Treat hosted features precisely: explicit/configurable,
never imply every capability is local.

**Files to create/modify:**
- Create `website/src/components/home/TrustSection.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.12] InstallSection Shared Conversion Flow
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-1.3, EPIC-1.5, EPIC-2.2, EPIC-2.6

**Acceptance criteria:**
- [ ] Render all supported agents, synchronize selection with HeroV2, persist it,
      and make the active option visible and keyboard operable.
- [ ] Display the approved per-agent command, copy button, `lean-ctx doctor`
      verification step, and relevant five-minute quickstart link.
- [ ] Anchor navigation to install preserves focus and supports direct `#install`
      URLs.
- [ ] LocalStorage failures/private mode degrade to the default agent.

**Technical notes:** One implementation must power both hero and install sections;
event names and storage key are internal constants with migration-safe versioning.

**Files to create/modify:**
- Create `website/src/components/home/InstallSection.astro`
- Modify `website/src/components/home/AgentSelector.astro`
- Modify `website/src/lib/agent-selection.ts`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-2.13] FAQ V2 and Structured Data
**Type:** content  
**Priority:** P1 (critical)  
**Estimated effort:** S (2–4h)  
**Dependencies:** EPIC-2.3

**Acceptance criteria:**
- [ ] Replace current FAQ copy with six approved answers covering scope, code
      changes, lossiness, agent support, data boundary, and proof of savings.
- [ ] Use native `<details>`/`<summary>` with visible focus and no accordion JS.
- [ ] Generate FAQPage JSON-LD solely from the rendered FAQ data; title and
      answer text do not drift.

**Technical notes:** Retire the old `FaqSection.astro` only after all callers are
known; an adapter is acceptable during rollout.

**Files to create/modify:**
- Create `website/src/components/home/FaqSectionV2.astro`
- Modify `website/src/data/homepage-v2.ts`

### Ticket: [EPIC-2.14] Homepage Composition and Route Cutover
**Type:** refactor  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-2.4 through EPIC-2.13

**Acceptance criteria:**
- [ ] Reduce `IndexPage.astro` to metadata, structured data, imports, and V2
      section composition in narrative order.
- [ ] Use HeaderV2/FooterV2 only for V2 route; remove homepage-only remote stats
      fetches and duplicate inline animation scripts.
- [ ] Preserve canonical URL `/`, explicit title/description, and valid landmark
      hierarchy; V2 has one H1.
- [ ] Legacy homepage components become unreferenced before deletion work begins.

**Technical notes:** Do not delete legacy components in this ticket. Use a clean
small temporary composition page if rollout needs preview comparison.

**Files to create/modify:**
- Modify `website/src/page-templates/IndexPage.astro`
- Modify `website/src/pages/index.astro`
- Modify `website/src/layouts/BaseLayout.astro`

### Ticket: [EPIC-2.15] Homepage CSS Retirement and Performance Pass
**Type:** refactor  
**Priority:** P1 (critical)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-2.14

**Acceptance criteria:**
- [ ] Move V2 shared styles to global primitives/component-scoped styles and
      remove unreferenced homepage selectors from `index.css`.
- [ ] Eliminate continuous, decorative animation and costly filters on mobile;
      terminal panels remain readable with horizontal scroll where necessary.
- [ ] Verify no V2 selector uses an undefined token and CSS bundle does not grow
      without a documented reason.

**Technical notes:** A CSS LOC target is not a quality measure; delete only
verified-unused styles and retain legacy rules until their pages are retired.

**Files to create/modify:**
- Modify `website/src/styles/index.css`
- Modify `website/src/styles/global.css`
- Modify V2 component styles as needed

### Ticket: [EPIC-2.16] Homepage SEO, Analytics, and Link QA
**Type:** refactor  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-2.14

**Acceptance criteria:**
- [ ] Validate canonical, OpenGraph/Twitter metadata, FAQ JSON-LD, and semantic
      headings in built HTML.
- [ ] Verify every V2 internal link resolves with trailing-slash policy and every
      external CTA has safe target/rel handling.
- [ ] Add privacy-preserving, optional conversion measurement only if an existing
      approved analytics mechanism exists; otherwise document manual event hooks.

**Technical notes:** Do not add third-party tracking in scope of a design rebuild.
Treat Pagefind indexing and sitemap output as build artefacts to test.

**Files to create/modify:**
- Modify `website/src/layouts/BaseLayout.astro`
- Modify `website/src/page-templates/IndexPage.astro`
- Create/modify `website/scripts/check-links.mjs`

### Ticket: [EPIC-2.17] Homepage Cross-Browser Release Review
**Type:** design  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-2.15, EPIC-2.16

**Acceptance criteria:**
- [ ] Capture desktop and 320/375/768px review artefacts in both themes.
- [ ] Test Chrome and Mobile Safari interactions: selector, copy, tab switcher,
      FAQ, menu, theme persistence, and reduced motion.
- [ ] Record Lighthouse and Core Web Vitals measurements against the release
      quality gate; failures generate follow-up issues before release.

**Technical notes:** This ticket signs off the homepage; it does not waive a
failed SEO/accessibility/links check.

**Files to create/modify:**
- Create `website/design/qa/homepage-v2-release.md`
- Create test artefacts under `website/design/qa/artifacts/` (git-ignore screenshots if policy requires)

**Epic 2 exit:** `/` delivers the new narrative, one coherent install flow, and
claims with evidence; no SEO or accessibility regression is accepted.

---

# Epic 3: Content Consolidation (Week 3)

**Goal:** make the simplified funnel true in the route graph, not merely in the
homepage navigation. Retire redundant pages safely and preserve search equity.

### Ticket: [EPIC-3.1] Canonical Route and Content Disposition Inventory
**Type:** content  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-2.14

**Acceptance criteria:**
- [ ] Inventory every current English marketing route and locale family as keep,
      rebuild, consolidate, docs move, redirect, or retain-for-legal/account.
- [ ] Define a single destination and rationale for `/what-is-leanctx`,
      `/what-is-context-engineering`, `/architecture`, `/compatibility`,
      `/compare`, `/use-cases/*`, `/addons`, `/builders`, `/cgb`, `/tools`,
      `/services`, and `/metrics`.
- [ ] Record title/meta/canonical/traffic-owner decisions and locale handling for
      each removed/moved path.

**Technical notes:** Redirect destinations must be semantically closest, not a
blanket homepage redirect. Ask SEO owner to approve before changing config.

**Files to create/modify:**
- Create `website/design/route-migration-inventory.csv`
- Modify `website/astro.config.mjs` (only after approval)

### Ticket: [EPIC-3.2] 301 Redirect Implementation and Test Matrix
**Type:** refactor  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-3.1

**Acceptance criteria:**
- [ ] Add approved redirects for every retired English route and appropriate
      localized families, preserving Astro trailing slashes.
- [ ] Verify each source resolves once (no redirect chain/loop) to a 200
      canonical destination and preserves relevant query strings where platform
      support allows.
- [ ] Test existing redirects remain valid and no source maps to a soon-retired
      route.

**Technical notes:** Prefer Astro `redirects` for static redirects; record any
hosting-layer requirement separately. Do not physically delete page files until
this matrix passes in a preview build.

**Files to create/modify:**
- Modify `website/astro.config.mjs`
- Create `website/scripts/verify-redirects.mjs`
- Create `website/design/redirect-matrix.csv`

### Ticket: [EPIC-3.3] Simplified Navigation Data and Legacy Component Retirement
**Type:** refactor  
**Priority:** P1 (critical)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-2.4, EPIC-2.5, EPIC-3.2

**Acceptance criteria:**
- [ ] Promote HeaderV2/FooterV2 to the global default once all retained routes
      have valid destinations.
- [ ] Remove mega-dropdown/use-case/compatibility/addons links from shared
      navigation and V2 footer.
- [ ] Delete or archive legacy header/footer and `MegaDropdown.astro` only when
      `rg` confirms no live imports.

**Technical notes:** Do not remove search, theme toggling, locale access (footer
may own it), or legal/account paths simply because they are not primary nav.

**Files to create/modify:**
- Modify `website/src/layouts/BaseLayout.astro`
- Modify `website/src/components/Header.astro` or remove after migration
- Modify `website/src/components/Footer.astro` or remove after migration
- Remove `website/src/components/MegaDropdown.astro` (conditional)

### Ticket: [EPIC-3.4] Deprecated Template Retirement
**Type:** refactor  
**Priority:** P1 (critical)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-3.2, EPIC-3.3

**Acceptance criteria:**
- [ ] Retire approved redundant marketing templates/routes only after route tests
      and import checks pass.
- [ ] Move useful architecture, compatibility, addon, comparison, metrics, and
      use-case content into designated docs/secondary-page destinations before
      deleting source material.
- [ ] Preserve legal, account, status, and security routes unless explicitly
      included in the approved inventory.

**Technical notes:** Split into one subtask per route family in GitLab if review
size is high. Generated locale pages must be regenerated or deliberately removed
through their generator, not hand-deleted en masse.

**Files to create/modify:**
- Modify/remove approved files in `website/src/page-templates/`
- Modify/remove corresponding files in `website/src/pages/`
- Modify docs destination pages

### Ticket: [EPIC-3.5] English-First i18n Cutover and Key Cleanup
**Type:** refactor  
**Priority:** P1 (critical)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-3.1, EPIC-3.4

**Acceptance criteria:**
- [ ] Establish English V2 as the only redesigned marketing source; translated
      routes either retain stable legacy pages or redirect according to the
      approved inventory.
- [ ] Remove unused i18n keys only after generator/usage validation proves they
      have no active pages; retain a migration map for keys needed in later V2
      localization.
- [ ] Build locale pages and validate canonical/hreflang behaviour.

**Technical notes:** Do not remove locales from `astro.config.mjs` without a
separate business decision. It is safer to stage local content migration than
to silently serve English at localized URLs.

**Files to create/modify:**
- Modify `website/src/i18n/`
- Modify `website/scripts/generate-locale-pages.mjs`
- Modify `website/astro.config.mjs`

### Ticket: [EPIC-3.6] Sitemap, Internal Links, and Search Rebuild
**Type:** refactor  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-3.2 through EPIC-3.5

**Acceptance criteria:**
- [ ] Build sitemap and Pagefind index after redirects/template retirement;
      neither includes retired canonical URLs.
- [ ] Crawl internal links in rendered output and return zero 404s, loops, or
      invalid locale/canonical links.
- [ ] Verify robots/noindex rules continue to exclude account/auth and shared
      wrapped pages.

**Technical notes:** Run `npm run build` rather than trusting dev-server routing.

**Files to create/modify:**
- Modify `website/astro.config.mjs`
- Modify/create `website/scripts/check-links.mjs`
- Modify `website/package.json`

**Epic 3 exit:** navigation, footer, sitemap, redirects, and rendered links all
describe the same canonical information architecture.

---

# Epic 4: Secondary Pages (Week 4)

**Goal:** provide decision-specific depth while preserving the homepage’s focused
story. All copy is present tense and explains the product as an Agent Context
Control Plane / context layer—not a generic token-saving utility.

### Ticket: [EPIC-4.1] Secondary-Page Shell and Content Standards
**Type:** design  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-1.1, EPIC-1.3, EPIC-1.4, EPIC-1.6, EPIC-2.1, EPIC-2.4, EPIC-2.5

**Acceptance criteria:**
- [ ] Establish reusable secondary-page hero, proof block, CTA, and metadata
      conventions using V2 primitives.
- [ ] Each page has one H1, evidence-bearing claims, and one dominant action;
      no page reintroduces the homepage’s old feature inventory.
- [ ] Shared shell has visual and accessibility QA in both themes.

**Technical notes:** Adapt `PageHero.astro` only if it can meet V2 semantics;
otherwise add a small V2 component and migrate callers deliberately.

**Files to create/modify:**
- Create `website/src/components/secondary/SecondaryPageHero.astro`
- Create `website/src/components/secondary/ProofCallout.astro`
- Modify `website/src/styles/global.css`

### Ticket: [EPIC-4.2] How It Works — Three-Step Mechanism Page
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-4.1, EPIC-2.9, EPIC-3.1

**Acceptance criteria:**
- [ ] Rebuild `/how-it-works/` around Read → Search & Execute → Proxy & Track,
      each with real terminal input/output and direct docs links.
- [ ] Explain the product boundary: context selection, reuse, coordination, and
      evidence—not edits to customer code.
- [ ] Use claims/basis labels for all performance and savings statements.

**Technical notes:** Architecture depth belongs in docs `/docs/architecture/`;
this page remains an evaluator-friendly mechanism explainer.

**Files to create/modify:**
- Modify `website/src/page-templates/HowItWorksPage.astro`
- Modify `website/src/pages/how-it-works.astro`
- Create/modify `website/src/data/how-it-works.ts`

### Ticket: [EPIC-4.3] Pricing — Clear Tiers, Boundaries, and ROI Calculator
**Type:** feature  
**Priority:** P1 (critical)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-4.1, EPIC-2.1, EPIC-3.1

**Acceptance criteria:**
- [ ] Present maintainer-approved Free/Pro/Team/Enterprise tiers, feature
      boundaries, and contact/purchase paths with no invented pricing.
- [ ] Include a client-side ROI calculator with labelled assumptions, local-only
      calculation, reset action, and an explanation that it is illustrative.
- [ ] Calculator formula, default values, and currency/period semantics are
      documented and keyboard/screen-reader usable.

**Technical notes:** The calculator must not make savings claims beyond the
registry. If pricing is undecided, ship a contact-oriented comparison rather
than placeholder dollar amounts.

**Files to create/modify:**
- Modify `website/src/page-templates/PricingPage.astro`
- Modify `website/src/pages/pricing.astro`
- Create `website/src/components/pricing/RoiCalculator.astro`
- Create `website/src/data/pricing.ts`

### Ticket: [EPIC-4.4] Enterprise — Governance Story
**Type:** content  
**Priority:** P1 (critical)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-4.1, EPIC-2.11, EPIC-3.1

**Acceptance criteria:**
- [ ] Rebuild `/enterprise/` around policy, budgets, sensitivity controls,
      auditability, and signed evidence with present-tense, substantiated copy.
- [ ] Link clearly to security, compliance, and contact routes without claiming
      unshipped certifications or hosted features.
- [ ] Include data-boundary and deployment language reviewed by security/product.

**Technical notes:** Treat policy packs and compliance support precisely; legal
or CISO approval is required for any certification/compliance assertion.

**Files to create/modify:**
- Modify `website/src/page-templates/EnterprisePage.astro`
- Modify `website/src/pages/enterprise/index.astro`
- Modify `website/src/data/claims.json`

### Ticket: [EPIC-4.5] About — Mission, Team, and Open-Core Model
**Type:** content  
**Priority:** P2 (important)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-4.1, EPIC-3.1

**Acceptance criteria:**
- [ ] Create `/about/` explaining team/mission, local core, Apache-2.0, and the
      open-core boundary in plain language.
- [ ] Consolidate approved useful material from What Is lean-ctx and What Is
      Context Engineering without duplicate canonical narratives.
- [ ] Include owner-approved people/company facts, community links, and metadata.

**Technical notes:** Avoid invented bios, funding, or roadmap commitments. The
technical category explanation should link to docs, not duplicate a whitepaper.

**Files to create/modify:**
- Create `website/src/page-templates/AboutPage.astro`
- Create `website/src/pages/about.astro`
- Modify `website/src/components/HeaderV2.astro`
- Modify `website/astro.config.mjs`

### Ticket: [EPIC-4.6] Secondary-Page Release QA
**Type:** design  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-4.2 through EPIC-4.5

**Acceptance criteria:**
- [ ] Run page-level accessibility, responsive, theme, link, claim, and metadata
      checks for all four secondary pages.
- [ ] Verify redirect destinations from Epic 3 match the final page content.
- [ ] Resolve or ticket every failed quality gate before merging.

**Technical notes:** Include ROI-calculator interaction tests in this release gate.

**Files to create/modify:**
- Create `website/design/qa/secondary-pages-v2-release.md`

---

# Epic 5: Documentation Restructure (Week 5)

**Goal:** turn docs into the fastest path from chosen agent to first structured
read, then expose the complete tool reference without overwhelming a new user.

### Ticket: [EPIC-5.1] Docs Home — Getting-Started-First IA
**Type:** content  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-2.2, EPIC-3.1, EPIC-4.1

**Acceptance criteria:**
- [ ] Rebuild `/docs/` with a clear first-run path: choose agent → install →
      verify → first `ctx_read` → next concepts/tools.
- [ ] Provide visible paths for evaluator, daily developer, platform owner, and
      reference reader without a mega-grid.
- [ ] Retain searchable access to existing concepts/tool content and canonical
      breadcrumbs.

**Technical notes:** Preserve docs layout/sidebar functionality; change content
hierarchy before changing every individual reference page.

**Files to create/modify:**
- Modify `website/src/pages/docs/index.astro`
- Create/modify `website/src/page-templates/DocsHomePage.astro`
- Modify `website/src/components/DocsSidebar.astro`

### Ticket: [EPIC-5.2] Agent-Specific Quickstart Framework
**Type:** feature  
**Priority:** P0 (blocker)  
**Estimated effort:** L (1–2 days)  
**Dependencies:** EPIC-2.2, EPIC-5.1

**Acceptance criteria:**
- [ ] Build a reusable quickstart template driven by the same agent install data
      as the homepage.
- [ ] Every quickstart includes prerequisites, exact approved install command,
      verification, first structured read, expected outcome, troubleshooting,
      and links back to tool reference.
- [ ] Add canonical route pattern and navigation; no hand-copied command strings.

**Technical notes:** Static Astro dynamic routing is acceptable if `getStaticPaths`
is exhaustive and unknown IDs yield 404. Use `CopyCommand` consistently.

**Files to create/modify:**
- Create `website/src/page-templates/DocsAgentQuickstartPage.astro`
- Create `website/src/pages/docs/quickstarts/[agent].astro`
- Modify `website/src/components/DocsSidebar.astro`

### Ticket: [EPIC-5.3] Publish Cursor, Claude Code, Codex, Windsurf, Copilot, and MCP Quickstarts
**Type:** content  
**Priority:** P1 (critical)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-5.2

**Acceptance criteria:**
- [ ] Publish a reviewed quickstart for each supported agent/data entry.
- [ ] Validate each command against a clean environment or documented fixture;
      failures block publication.
- [ ] Use agent-specific UI concepts only where needed and keep shared context
      concepts consistent across pages.

**Technical notes:** A missing verified command blocks that agent page, not the
entire docs redesign; show only supported integrations.

**Files to create/modify:**
- Modify `website/src/data/agent-installs.ts`
- Create content/data entries for each quickstart
- Modify `website/src/pages/docs/quickstarts/[agent].astro`

### Ticket: [EPIC-5.4] Tool Reference Cleanup and Progressive Depth
**Type:** refactor  
**Priority:** P1 (critical)  
**Estimated effort:** XL (2–5 days)  
**Dependencies:** EPIC-3.4, EPIC-5.1

**Acceptance criteria:**
- [ ] Make `/docs/tools/` a concise task-oriented index: Read, Search & Execute,
      Memory, Intelligence, Workflow, and Reference.
- [ ] Consolidate retired marketing `/tools/` content into canonical docs tools
      URLs and retain redirects.
- [ ] Audit each tool page for current names, examples, navigation, and links;
      remove duplicate or stale generated summaries.

**Technical notes:** Keep generated tool docs workflow intact (`generate:tools`,
validation scripts); improve routing/source data rather than manually fighting
the generator output.

**Files to create/modify:**
- Modify `website/src/page-templates/DocsToolsPage.astro`
- Modify `website/src/page-templates/DocsTools*.astro`
- Modify `website/src/pages/docs/tools*.astro`
- Modify `website/scripts/generate-tool-docs.mjs`

### Ticket: [EPIC-5.5] Docs Discoverability and Final Validation
**Type:** refactor  
**Priority:** P0 (blocker)  
**Estimated effort:** M (4–8h)  
**Dependencies:** EPIC-5.2 through EPIC-5.4

**Acceptance criteria:**
- [ ] Search/Pagefind indexes quickstarts and canonical reference pages; no
      obsolete path is promoted.
- [ ] Run all existing documentation, manifest, positioning, facts, journey,
      i18n, and tool validations plus link crawl/build.
- [ ] Manually complete first-run docs flows for each published agent route.

**Technical notes:** `npm run validate` and `npm run build` are minimum checks;
record any intentionally deferred localization work as a follow-up issue.

**Files to create/modify:**
- Modify `website/package.json` (only if a missing validation command is added)
- Create `website/design/qa/docs-v2-release.md`

---

# Risk analysis

| Risk | Impact | Mitigation |
|---|---|---|
| Breaking existing SEO rankings | High | 301 redirects for every removed/moved page; approved route inventory, rendered crawl, and post-release Search Console monitoring. |
| Losing i18n coverage | Medium | Rebuild English first, retain stable localized routes, then localize from data contracts and validate hreflang/canonical output. |
| Animation performance on mobile | Medium | CSS-first, bounded IntersectionObserver use, no perpetual decorative motion, `prefers-reduced-motion`, and Mobile Safari testing. |
| Design inconsistency during migration | Low | Isolate V2 primitives/components, use a short-lived preview/feature branch, and retire legacy selectors only after imports are gone. |
| Number/claim drift | High | Claims registry, typed loaders, validator, BasisLabel-only rendering, named owners/review dates, and release claim audit. |
| Unsupported or incorrect install commands | High | One reviewed agent-install data source, clean-environment checks, and quickstart publication blocked until command evidence exists. |
| Redirect loops or locale regressions | High | Matrix tests cover English and locale families, single-hop assertions, canonical checks, and sitemap crawl after build. |
| Scope creep from 70+ legacy templates | Medium | Route disposition inventory gates retirement; split each route family into a separate subtask and preserve account/legal/status pages by default. |

# Quality gates

Before each epic ships:

- [ ] Lighthouse: Performance ≥ 95, Accessibility ≥ 95, SEO ≥ 95
- [ ] Core Web Vitals: LCP < 2.5s, CLS < 0.1, INP < 200ms
- [ ] Mobile Safari + Chrome tested
- [ ] Dark + Light theme tested
- [ ] `prefers-reduced-motion` tested
- [ ] All links verified (no 404s)
- [ ] Claims registry consistent (no number drift)

Additionally require `npm run validate` and `npm run build` for every route,
navigation, docs, or content consolidation change. Quality-gate tickets attach
measurements and screenshots to the GitLab issue; a passing local impression is
not an acceptance substitute.

---

# Agent-executable goal files

The following are ready to save verbatim as individual Codex goals when their
dependencies are complete. They deliberately name exact files and verification
commands so agents can work independently without changing adjacent scope.

## `.agents/goals/epic-1-2-scroll-reveal.md`

# Agent Goal: Build ScrollReveal

## Task
Create a reusable Astro scroll-reveal primitive for V2 sections.

## Dependencies
- EPIC-1.1 complete.

## Files to create
- `website/src/components/ui/ScrollReveal.astro`

## Files to modify
- `website/src/styles/global.css`

## Exact requirements
- Props: `direction?: 'up' | 'down' | 'left' | 'right' | 'none'`,
  `delay?: number`, `stagger?: number`, and `as?: string`.
- Render content visible by default. Add the hidden/pre-reveal state only after
  JS confirms motion is allowed and IntersectionObserver is available.
- Observe the wrapper or children once, reveal when intersecting, then disconnect.
- `stagger` applies deterministic CSS custom-property delays to direct children.
- Under `prefers-reduced-motion: reduce`, reveal immediately with no transform,
  opacity transition, or delayed content.
- Do not add React, a hydration framework, or a global interval.

## Acceptance checks
- `npm run build` from `website/` succeeds.
- A fixture with directions/delays has no hidden text with JS disabled.
- Keyboard focus can reach a reveal child before it is scrolled into view.
- Light/dark and reduced-motion styles use only defined global tokens.

## Out of scope
- Migrating homepage sections or changing legacy `animate-entrance` classes.

## `.agents/goals/epic-2-6-hero-v2.md`

# Agent Goal: Build HeroV2

## Task
Create the proof-led V2 hero and shared agent-selection interaction.

## Dependencies
- EPIC-1.2 through EPIC-1.5, EPIC-2.1 through EPIC-2.3 complete.

## Files to create
- `website/src/components/home/HeroV2.astro`
- `website/src/components/home/AgentSelector.astro`
- `website/src/lib/agent-selection.ts`

## Files to modify
- `website/src/styles/global.css`

## Exact requirements
- Full viewport hero: eyebrow “THE CONTEXT LAYER FOR CODING AGENTS”; H1 “Give
  your agents context — not clutter.”; approved narrative subline.
- Use a restrained dark-editorial, emerald-only background made from CSS; avoid
  remote images/canvas/continuous animation.
- Selector lists data-backed agents, defaults safely, persists an ID under a
  versioned localStorage key, and emits an event so all selector instances sync.
- CTA reads `Install for [selected agent]`, links to `#install`, and does not
  compete with Discord or GitHub buttons.
- Render real approved terminal fixture output in `TerminalPanel`; terminal is
  fully visible under reduced motion and does not repeat on a loop.
- Include trust strip and one `BasisLabel` micro-proof.
- Ensure 320px layout, no-JS readable fallback, and focus-visible selector/CTA.

## Acceptance checks
- `npm run build` succeeds.
- Changing selection then reloading preserves it; blocked localStorage falls
  back to the default without an exception.
- Browser keyboard operates selector and reaches CTA.
- No text metric is hard-coded when a claims ID is available.

## Out of scope
- Updating `IndexPage.astro` composition and implementing the lower install
section; those are EPIC-2.12 and EPIC-2.14.

## `.agents/goals/epic-2-10-proof-section.md`

# Agent Goal: Build ProofSection Mode Switcher

## Task
Implement the V2 interactive proof section: one source file, three useful views.

## Dependencies
- EPIC-1.3, EPIC-1.5, EPIC-1.6, and EPIC-2.9 complete.

## Files to create
- `website/src/components/home/ProofSection.astro`

## Files to modify
- `website/src/styles/global.css`

## Exact requirements
- Consume only `proof-demo.ts` typed data; do not define content or token totals
  inside markup.
- Show source on the left and selected useful output on the right inside
  `TerminalPanel` instances.
- Implement `full`, `map`, and `signatures` as ARIA tabs with arrow-key/Home/End
  support, visible selected state, and matching tab panels.
- Switching mode updates token count, retained symbols, basis label, and Expand
  action together.
- Expand shows the approved fuller payload inline and is a real button with
  `aria-expanded`; it does not fetch data.
- Default panel remains useful without JavaScript; copy buttons use `CopyCommand`.

## Acceptance checks
- `npm run build` succeeds.
- Keyboard tabs pass a manual ARIA tabs walkthrough.
- All rendered metrics have a registry-backed BasisLabel.
- 320px screen keeps terminals usable via intended horizontal overflow and no
  unexpected page-wide horizontal scroll.

## Out of scope
- Live compression or new benchmark collection.

## `.agents/goals/epic-2-12-install-section.md`

# Agent Goal: Build InstallSection

## Task
Build the shared conversion section for an agent-specific first install.

## Dependencies
- EPIC-1.3, EPIC-1.5, EPIC-2.2, and EPIC-2.6 complete.

## Files to create
- `website/src/components/home/InstallSection.astro`

## Files to modify
- `website/src/components/home/AgentSelector.astro`
- `website/src/lib/agent-selection.ts`
- `website/src/styles/global.css`

## Exact requirements
- Root element is `id="install"` and has a focusable heading target for hash
  navigation.
- Use the canonical agent-install data: selector, install command, copy action,
  `lean-ctx doctor` verification, and per-agent quickstart URL.
- Synchronize selection with HeroV2 through the shared state helper, including
  storage persistence and safe fallback when storage is unavailable.
- Use `TerminalPanel` and `CopyCommand`; no copied clipboard logic.
- State plainly that no account is required for the local core and retain the
  approved reassurance line.
- Support mouse, touch, keyboard, both themes, and reduced motion.

## Acceptance checks
- `npm run build` succeeds.
- Selecting an agent in HeroV2 reflects in InstallSection and vice versa.
- Copy feedback is announced and returns to resting state.
- Every quickstart URL resolves in the current build or is intentionally withheld
  until Epic 5, with a documented temporary canonical docs link.

## Out of scope
- Writing quickstart content or adding new agent integrations.

## `.agents/goals/epic-3-2-redirects.md`

# Agent Goal: Implement and Verify Redirect Matrix

## Task
Turn the approved route-migration inventory into tested, single-hop redirects.

## Dependencies
- EPIC-3.1 approved by content/SEO owner.

## Files to create
- `website/scripts/verify-redirects.mjs`
- `website/design/redirect-matrix.csv`

## Files to modify
- `website/astro.config.mjs`

## Exact requirements
- Add only redirects explicitly approved in the migration inventory.
- Preserve `trailingSlash: 'always'` destinations and prevent any route from
  redirecting through another retiring route.
- Cover English sources and the locale policy identified by the inventory.
- Matrix columns: source, destination, status, rationale, locale policy, test
  result, owner.
- Verification script asserts no duplicate source, no self-loop, no chained
  approved destination, and successful target build route.
- Do not delete source page/template files in this task.

## Acceptance checks
- `npm run build` succeeds.
- Redirect verifier passes against the production build/preview configuration.
- Every approved source reaches one canonical 200 destination; no 404/loop.

## Out of scope
- Navigation changes, i18n key cleanup, template deletion, sitemap changes.

