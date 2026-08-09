# lean-ctx — New Design Language & Storyline (v2)

> Synthesized from 6 Codex CLI agents:
> - Round 1: Deep analysis of lean-ctx website, nango.dev, caveman.so
> - Round 2: Full product capability catalog, category creation, narrative architecture
>
> Goal: A clear, compelling website that communicates what lean-ctx REALLY is —
> not a token saver, but the context layer for coding agents.

---

## Executive Summary

### Problem with the current site
The current website tries to sell the entire company on the landing page: 8 major sections,
70+ page templates, a mega-dropdown with 30+ links, and a hero that simultaneously explains
compression, proxy, memory, governance, signed proof, web intake, and API embedding.
A new visitor cannot determine what lean-ctx IS within 10 seconds.

### What we learn from competitors

| Pattern | Nango | Caveman | lean-ctx (NEW) |
|---------|-------|---------|----------------|
| Hero claim | "Integrations for your products & agents" | "Cut 65% of your tokens" | "Your agents read too much. lean-ctx fixes that." |
| Narrative | 3-step pipeline | 4-chapter editorial | 4-section proof arc |
| Trust | Logos + testimonials + runtime proof | GitHub stars + explicit claim basis | Reproducible benchmarks + deterministic proof |
| CTA | START BUILDING / BOOK A DEMO | Install 4 ways / Waitlist | Install for [your agent] |
| Design | Product-UI simulations | Dark terminal + ledgers | Dark editorial + live terminal proof |

---

## Part 1: Design Principles (Revised)

### 1. One idea per scroll
Every viewport makes exactly one argument. No section contains more than one claim +
one proof artifact. Whitespace is not wasted space — it is focus.

### 2. Proof, not adjectives
Every claim ships with a reproducible artifact: a terminal output, a token count,
a benchmark trace, or a code comparison. No decorative screenshots. No stock visuals.

### 3. Show the product, not a diagram of it
Like Nango's product-UI simulations and Caveman's compression workbench: show real
tool output, real code, real terminal sessions. The website IS a product demo.

### 4. Claims carry their basis
Borrowed from Caveman: every number on the site states its measurement basis.
"62% fewer tokens (measured: Rust codebase, 847 files, map mode, o200k tokenizer)"
Not "up to 99%" without context.

### 5. One obvious next step
Every section funnels to one action: **Install for [your agent]**. The agent selector
(Cursor, Claude Code, Codex, Windsurf, ...) persists across the site. No competing
CTAs within the same viewport.

### 6. Progressive depth
The landing page answers WHY and WHAT. Docs answer HOW. Architecture answers HOW DEEP.
Nobody is forced through marketing to reach docs. Nobody is overwhelmed with enterprise
governance before they understand the core product.

---

## Part 2: The New Storyline

### Hero → Proof → How → Trust → Install

Four sections. One scroll each. No detours.

---

### Section 1: HERO — The Promise

**Eyebrow:**
> THE CONTEXT LAYER FOR CODING AGENTS

**Headline:**
> Give your agents context — not clutter.

**Subline:**
> lean-ctx helps coding agents read the right code, remember what they learn,
> and work from one shared understanding. Local by default. Open source.

**Primary CTA:** Install for [Cursor ▾] (agent selector, persists across site)
**Secondary link:** Watch it read smarter ↓

**Trust strip:**
> AST-aware context · 10 read modes · 27 languages · Cursor, Claude Code, Codex + more

**Micro-proof:**
> Your code stays local by default. Every performance claim links to its method.

**Design notes:**
- Dark background, emerald accent on CTA
- The agent selector is the HERO INTERACTION
- Small "Apache-2.0 · ★ N on GitHub" trust line below CTA
- NO Discord, no star-on-GitHub competing with Install
- The headline sells OUTCOME, not mechanism

---

### Section 2: THE PROBLEM

**Eyebrow:** `THE COLD-START TAX`

**Headline:**
> Every new agent starts by forgetting.

**Body copy:**
> It rereads files it has already seen. It scans terminal noise for the one useful line.
> It loses the decision the last agent made. Then the next handoff repeats it all.
>
> Bigger context windows do not solve this. They make it easier to send more noise
> to a model that still cannot tell what matters.

**Three exhibit labels:**
- `RE-READING` — The same repository enters the context window again.
- `RE-DISCOVERING` — Architectural decisions vanish with the session.
- `RE-EXPLAINING` — Every agent works from its own partial version of reality.

**Closing line:**
> Your agents do not need more context. They need a better way to earn it.

---

### Section 3: THE SOLUTION

**Eyebrow:** `CONTEXT, ENGINEERED`

**Headline:**
> Before an agent reads, lean-ctx decides what matters.

**Intro copy:**
> lean-ctx sits between your agent and the engineering system it depends on.
> It turns raw code, tools, decisions, and external work into usable working context.

**Capability 01 — Read the shape of the code**
> Structure over file dumps. AST-aware reads, code graphs, intent detection,
> and focused search show the symbols, dependencies, and changes the task needs.

**Capability 02 — Keep what the team learns**
> Memory over cold starts. Sessions, knowledge, snapshots, and provider data
> preserve relevant understanding after the chat ends.

**Capability 03 — Turn agents into a team**
> Coordination over parallel confusion. Agents share context, hand off work,
> and leave decisions where the next agent can use them.

**Capability 04 — Prove and govern the work**
> Evidence over black boxes. Policies, budgets, sensitivity controls, and a
> signed ledger make agent activity accountable.

**Closing line:**
> Token savings are the receipt. Better agent judgment is the product.

---

### Section 4: PROOF — Interactive Demo

**Eyebrow:** `SEE THE CONTEXT CHANGE`

**Headline:**
> One file. Three useful views.

**Interactive artifact:**
Real source file with mode switcher: `full` → `map` → `signatures`
Each shows actual output, token count, retained symbols, and an Expand action.

**Metric rail:**
> `4,200 tokens → 180 tokens`
> `95.7% less context for this view`
> `Basis: [repository] · [file] · map mode · [tokenizer] · [date]`

**Second proof beat:**
> Re-read the same view? The cache returns known context instead of
> charging the agent to learn it twice.

**Proof CTA:** Inspect the benchmark method →

**Design notes:**
- Split layout: original (dimmed) left, compressed output right
- Terminal chrome, monospace
- Mode selector as tabs/segmented control
- Every number states its basis

---

### Section 3: HOW — The Complete Loop

**Eyebrow:** `[ 02 ]  NOT JUST READS`

**Headline:**
> Every tool your agent touches. Compressed.

**Three-step narrative (inspired by Nango):**

**Step 1: Read**
```
ctx_read("src/auth.rs", mode="map")
→ 4,200 tokens → 180 tokens
```
"10 read modes. Tree-sitter AST for 27 languages. The agent sees structure, not noise."

**Step 2: Search & Execute**
```
ctx_search("authentication middleware")
→ 12 results, ranked, deduplicated

ctx_shell("cargo test auth")
→ 340 lines → 28 lines (95+ shell patterns)
```
"Shell output, search results, git status — all compressed with pattern-specific rules."

**Step 3: Proxy & Track**
```
lean-ctx proxy → localhost:4141
→ Prefix caching · Cost tracking · Provider routing
→ Savings ledger: Ed25519-signed, hash-chained
```
"Every API call through the proxy. Every saving verified. Every cent attributed."

**Inline CTA:** "See the full tool catalog →" (links to docs/tools)

**Design notes:**
- Vertical timeline layout with terminal chrome for each step
- Each step: icon → one-line description → real command + output
- Numbers on each step: `01`, `02`, `03`
- Inspired by: Nango's 3-step pipeline + Caveman's chapter structure

---

### Section 5: TRUST

**Eyebrow:** `BUILT FOR THE WORK YOU CANNOT HAND-WAVE`

**Headline:**
> Local-first. Deterministic. Accountable.

**Pillar 01:**
> Your code is not the product. lean-ctx runs locally by default with zero
> telemetry. Any hosted capability is explicit and configurable.

**Pillar 02:**
> Compression stays inspectable. Structure is preserved, originals can be
> expanded, and identical inputs produce deterministic outputs.

**Pillar 03:**
> Savings carry evidence. The Ed25519-signed, hash-chained ledger creates
> a tamper-evident record of what changed and what it saved.

**Trust line:**
> Apache-2.0 · local core · reproducible benchmarks · policy-backed evidence

**Enterprise link:** See how lean-ctx governs agent fleets →

---

### Section 6: INSTALL — The Conversion Moment

**Eyebrow:** `YOUR FIRST BETTER READ IS MINUTES AWAY`

**Headline:**
> Install for the agent you already use.

**Body copy:**
> Pick your agent. Copy one command. Run your first structured read.
> No account required for the local core.

**Selector:** Cursor · Claude Code · Codex · Windsurf · Copilot · Other MCP client

**Command panel:** `SETUP FOR [SELECTED AGENT]`

**Primary CTA:** Copy install command
**Verification:** Then run `lean-ctx doctor` to verify your context runtime is ready.
**Secondary link:** Read the 5-minute quickstart →

**Reassurance line:**
> Start local. Keep your workflow. Add shared memory and governance when your team is ready.

---

### Section 7: FAQ

**Is lean-ctx just a token compressor?**
No. It reduces wasteful context, but its job is larger: structured code reads,
memory across sessions, multi-agent coordination, code understanding, and
governance with evidence.

**Will lean-ctx change my code?**
No. Its context layer helps agents read, search, and interpret engineering
information. Your normal edit and review workflow remains in control.

**Is compression lossy?**
Different tasks need different views. lean-ctx preserves structure, makes output
inspectable, and lets the agent expand to fuller context when needed.

**Which agents work with lean-ctx?**
Cursor, Claude Code, Codex, Windsurf, Copilot, and any MCP-compatible agent.

**Does my code leave my machine?**
Not by default. lean-ctx is local-first and zero-telemetry. Hosted features
state their data boundary before activation.

**How do I prove the savings?**
Run the benchmark against your own repository. The ledger and reproducible
benchmark artifacts make performance claims verifiable, not promotional.

---

### Section 7: FOOTER (simplified)

**Two columns only:**

| Product | Resources |
|---------|-----------|
| How it works | Documentation |
| Pricing | GitHub |
| Enterprise | Discord |
| Changelog | Security |

Plus: "Apache-2.0 · Made in Switzerland · © 2026 lean-ctx"

NO: 30 links, 5 columns, every subpage listed.

---

## Part 3: Navigation (Simplified)

### Primary nav (5 items max):
1. **How it works** — the mechanism
2. **Docs** — getting started, tools, concepts
3. **Pricing** — free / pro / team / enterprise
4. **Enterprise** — governance, security, compliance
5. **Install** (emerald button, primary CTA)

### Header utilities:
- GitHub star count (small, linked)
- Theme toggle (dark/light)
- Search (icon, opens modal)

### REMOVED from primary nav:
- Use cases (→ move to docs or landing page sections)
- Compatibility (→ merge into Install section)
- Architecture (→ move to docs)
- Compare (→ move to pricing or docs)
- Addons (→ move to docs)
- Language selector (→ move to footer)

---

## Part 4: Design System Updates

### Color (keep current palette)
- Primary: Emerald (`#34d399` dark / `#047857` light)
- Surfaces: Current near-black ladder
- Text: Current white-lilac system
- **ENFORCE:** Indigo/purple ONLY in data-viz. Never as UI accents.

### Typography (keep current stack)
- Display: Space Grotesk
- Body: Inter
- Code/Data: JetBrains Mono
- **NEW:** Strict 4-level hierarchy: eyebrow → headline → body → caption

### Layout patterns
- **Content width:** 900px for text, 1200px for demos
- **Section rhythm:** `section-y` with generous padding
- **Card limit:** Max 3 cards per row. Never more.
- **Terminal chrome:** Used for ALL code/output displays

### Interactive elements
- **Agent selector:** Persistent component, reusable across pages
- **Mode switcher:** For compression demo (tabs, not dropdown)
- **Copy button:** On every code block and install command
- **Collapsible FAQ:** Native `<details>`, not JS accordion

### Claims registry (NEW)
Create `website/src/data/claims.json`:
```json
{
  "claims": [
    {
      "id": "avg-compression",
      "value": "62%",
      "label": "average token savings",
      "basis": "measured across 847 files, Rust+TS+Python, map mode",
      "tokenizer": "o200k",
      "date": "2026-07-15",
      "methodology": "/docs/benchmarks#methodology"
    }
  ]
}
```
Every number on the site references this registry. One source of truth.

---

## Part 5: Pages to Keep, Remove, Consolidate

### KEEP (core funnel):
- `/` — Landing page (rebuilt per above storyline)
- `/how-it-works` — Mechanism deep-dive
- `/pricing` — Plans + ROI calculator
- `/enterprise` — Governance, security, compliance
- `/docs/*` — Documentation (restructured for getting-started-first)

### CONSOLIDATE:
- `/what-is-leanctx` + `/what-is-context-engineering` → single `/about` or merge into landing
- `/architecture` → move into docs as `/docs/architecture`
- `/compatibility` → merge into Install section on landing page
- `/compare` → move into pricing as a tab/section
- `/use-cases/*` → move into docs or remove (the product demo IS the use case)

### REMOVE:
- `/addons` → premature for landing; move to docs
- `/builders` → merge into docs/contributing
- `/cgb` → move to blog/changelog
- `/tools` (marketing page) → redirect to docs/tools
- `/services` → premature; remove or hide
- `/metrics` → move to docs or dashboard
- All i18n pages initially (rebuild English first, then localize)

### NEW:
- `/about` — Team, mission, open-core model explained clearly

---

## Part 6: Category & Competitive Positioning

### The Category: Agent Context Control Plane

lean-ctx is NOT a token compressor. It is the **Agent Context Control Plane** — the layer
that selects, remembers, shares, governs, and proves the context behind every agent action.

**Three-level naming architecture:**
```
Category:        Agent Context Control Plane
Product:         lean-ctx, the Context OS for agents
Technical thesis: Governed pre-prompt runtime / Cognition Interface
```

**The simple analogy:**
> Git is the system of record for code.
> lean-ctx is the control plane for agent context.

### Five Jobs of the Control Plane

1. **Select** — deliver the most relevant code/context in the right form
2. **Remember** — preserve decisions, findings, and knowledge across sessions
3. **Coordinate** — let agents hand off work and share governed context
4. **Govern** — apply budgets, policies, and sensitivity rules before the model
5. **Prove** — keep an auditable account of context, quality, and savings

### The Anti-Caveman Narrative

**Core distinction:** Token compressors optimize a request. lean-ctx operates the
whole context lifecycle.

| Token-optimizer frame | lean-ctx control-plane frame |
|---|---|
| Fewer tokens enter the model | The RIGHT, permitted, reusable context reaches the agent |
| Optimizes a request | Operates the whole context lifecycle |
| Success: reduction percentage | Success: task quality, reuse, policy compliance, verified impact |
| Output is shorter | Context is selected, retained, shared, and proven |

**Competitive one-liners (use on compare pages, not homepage):**
1. "Token compressors shrink what agents say. lean-ctx transforms what agents know."
2. "Shorter prompts are useful. Better context changes the work itself."
3. "Compression removes noise from a message. lean-ctx decides what should enter it."
4. "One tool trims a request. lean-ctx gives the next agent the last agent's understanding."
5. "Saving tokens is an event. Building agent memory is an advantage."
6. "A proxy can reduce traffic. A context layer can read, reason, remember, coordinate, and prove."

**The honest qualifier:**
> If all you need is smaller payloads, a compressor may be enough.
> If you need agents to improve over time, work together, and operate under
> control, lean-ctx is built for that job.

### The 13 Capability Categories (from deep product analysis)

1. **Context Control & Density Runtime** — 10 read modes, AST, shell patterns, delta reads
2. **Adaptive Context Intelligence** — intent detection, pressure-aware optimization, bounce accounting
3. **Governed Context Kernel** — budget planning, receipts, policy enforcement, delivery traces
4. **Persistent Project Memory** — knowledge graph, lifecycle decay, supersession, embedding recall
5. **Graph-Native Code Understanding** — call graphs, impact analysis, architectural clusters, hotspots
6. **Hybrid Retrieval** — BM25 + embeddings + graph proximity + external providers (GitHub/GitLab/Jira)
7. **Session Continuity & Time Machine** — snapshots, restore, publish, replay against repo history
8. **Multi-Agent Coordination** — A2A protocol, handoffs, agent bus, diary, relay chains, cost attribution
9. **Context OS** — shared runtime, event bus, filtered subscriptions, multi-client operations
10. **Governance & Proof** — policy packs (SOC2, EU AI Act, ISO 42001), signed ledger, compliance reports
11. **OCLA Contract** — 16 open traits, stable OSS/enterprise boundary, provider-neutral envelopes
12. **Engineering Workflow** — task management, plans, reviews, refactoring, benchmarks, FinOps
13. **Cross-Repo & Ecosystem** — multi-root search, context packages, dynamic tool loading, 30+ agents

### The Real Moat (compounding system)

```
More agent work
  → richer graph, knowledge, session, and evidence
  → better context decisions and safer handoffs
  → more workflows standardized through policy
  → higher switching cost in operating practice, not captive data
```

---

## Part 7: Implementation Priority

### Phase 1: Landing page rebuild (1-2 weeks)
1. New hero with agent selector
2. Compression demo (interactive read mode comparison)
3. Three-step how-it-works
4. Trust pillars
5. Install section with agent-specific commands
6. Simplified nav + footer

### Phase 2: Content consolidation (1 week)
1. Remove/redirect deprecated pages
2. Consolidate overlapping content
3. Create claims registry
4. Fix metric/terminology consistency

### Phase 3: Docs restructure (1 week)
1. Getting-started-first architecture
2. Agent-specific quickstarts
3. Tool reference cleanup

### Phase 4: Enterprise & pricing (1 week)
1. Clean enterprise page (present-tense only)
2. Pricing with ROI calculator
3. Compare section

### Phase 5: Localization (after English is solid)
1. German
2. Other languages based on traffic data

---

## Part 8: Sales Layers by Buyer Persona

### The IC Developer
**Headline they see:** "Stop making your agent reread the repository."
**Message:** Install locally. Your agent gets a smaller, structured view of code,
cleaner command output, and cached re-reads — without changing your files or
sending code anywhere.
**Conversion proof:** Real terminal comparison + agent-specific install command.
**Aha moment:** "My agent can see the relevant call path AND remember the decision
from yesterday instead of repeatedly rediscovering the repo."

### The Tech Lead
**Section that convinces:** "Make the second agent better than the first."
**Message:** A team's progress should not disappear with a chat window. Shared
knowledge, session continuity, snapshots, and handoffs preserve architectural
decisions and project context across tasks and people.
**Conversion proof:** Before/after timeline: Agent A investigates, records the
decision → Agent B resumes with files, rationale, and tests — not a blank slate.
**Aha moment:** "A handoff includes the decisions, affected files, evidence, and
next step — not an unreliable chat recap."

### The Platform Engineer
**Architecture that sells:** "One local context runtime. Every agent interface."
**Message:** lean-ctx provides a common boundary for MCP clients, shell output,
code intelligence, provider data, caching, and context packages. The platform
team gets consistent behavior while developers keep their preferred agent.
**Conversion proof:** Architecture diagram with bounded planes: agent clients →
local context runtime → code/tools/knowledge → optional providers.
**Aha moment:** "We can give Cursor, Claude Code, Codex, and the next agent the
same governed context interface rather than rebuild controls for each."

### The CISO / VP Engineering
**Governance story:** "Let agents move fast without making their behavior unaccountable."
**Message:** Policy defines what an agent may access; signed evidence shows what it
did access. Budgets and leases constrain activity. Sensitivity controls and
local-first defaults protect code.
**Conversion proof:** Policy → decision → signed ledger → compliance report walkthrough.
**Aha moment:** "We can enforce what an agent may see and produce an audit trail of
context access without default cloud telemetry."

---

## Part 9: The Positioning Statement

lean-ctx is the context layer for AI coding agents. It sits between agents and
the code, tools, knowledge, and policies they depend on — so they read the right
information instead of the whole repository, remember what they learn across
sessions, coordinate as one team, and leave evidence for every decision. Token
savings are the result. The product is a more capable, governable engineering
intelligence.

**The shortest version:**
> lean-ctx gives coding agents the right context, a lasting memory, and a shared
> operating model — so they improve with every task instead of starting from zero.

---

## Part 10: Supporting Documents (from Agent Research)

All detailed analyses are preserved in `website/design/`:

| Document | Agent | Content |
|---|---|---|
| `analysis-leanctx-current.md` | Round 1, Agent 01 | Current website deep analysis |
| `analysis-nango.md` | Round 1, Agent 02 | nango.dev design/storytelling benchmark |
| `analysis-caveman.md` | Round 1, Agent 03 | caveman.so competitor analysis |
| `deep-feature-catalog.md` | Round 2, Agent 01 | Complete 13-category product capability map |
| `category-positioning.md` | Round 2, Agent 02 | Category creation, moat analysis, buyer personas |
| `narrative-sales-concept.md` | Round 2, Agent 03 | Full 5-act narrative, hero options, section scripts |
