# ADR-014: Open Runtime Invariant

**Status:** Accepted
**Date:** 2026-08-09
**Authors:** Architecture Team

## Context

The repository already has a stable and frozen `docs/contracts/local-free-invariant-v1.md`.
That contract protects the released single-developer experience and records the
historical promise that local capabilities are free and ungated. ADR-010,
`docs/contracts/oss-plane-separation-v1.md`, and the existing Runtime code were
written under that model.

The product now needs a forward-looking distinction between an independently
useful open Runtime and commercial decision intelligence. Enterprise customers
may require adaptive scheduling, cross-task learning, fleet control, or
governed organizational knowledge to run in a customer VPC, on-premises, or a
sovereign environment. Deployment location cannot be the license classifier.

The architecture therefore needs a replacement invariant for future decisions
without weakening the local functionality that has already been published.

## Decision

### Forward-only replacement

For decisions made on or after 2026-08-09, this ADR replaces the Local-Free
Invariant as the governing architectural test. The replacement is forward-only
and not retroactive:

> **The OSS Runtime remains independently useful for local coding tasks without
> requiring Control Plane, Cloud, or Enterprise services.**

This rule evaluates whether a new design preserves a capable local Runtime; it
does not require every future capability that can execute on a local machine to
be Apache-2.0 or free of an enterprise license. New capabilities are
classified under `docs/internal/OPEN_CORE_BOUNDARY.md`.

### What the invariant protects

The following remain meaningful, usable local Runtime capabilities without a
Control Plane, Cloud account, or Enterprise service:

- **Local reads and context construction**, including the read modes and local
  path safety implemented around `rust/src/tools/ctx_read`,
  `rust/src/core/structured_read.rs`, and `rust/src/core/pathjail.rs`.
- **Local shell execution and output handling**, including
  `rust/src/tools/ctx_shell.rs`, `rust/src/shell/`, and the shell allowlist
  and safety modules under `rust/src/core/shell_allowlist/`.
- **Local tools and MCP execution**, including `rust/src/tools/`,
  `rust/src/server/`, `rust/src/tool_defs/`, `rust/src/mcp_stdio.rs`, and the
  CLI paths under `rust/src/cli/`.
- **The local BYOK proxy and provider edge**, including `rust/src/proxy/`,
  `rust/src/proxy_setup/`, and local configuration paths under
  `rust/src/core/config/`. A developer can use credentials they control
  without a hosted gateway.
- **The local savings ledger and transparent evidence**, including
  `rust/src/core/savings_ledger/`, `rust/src/core/ocla/unified_ledger.rs`,
  and the local signing and verification primitives already in the Runtime.

These capabilities may use open contracts and local storage, but their basic
operation must not depend on a network call to Cloud or Enterprise. A local
failure to reach a commercial service may reduce commercial enrichment or
coordination; it must not make the protected local coding path unusable.

### What the invariant does not protect

The invariant does not require the following future capabilities to be open,
free, or independently complete in the OSS Runtime:

- adaptive routing that learns from multiple tasks, workloads, or tenants;
- cross-task or organization-wide learning and performance models;
- fleet management, centralized scheduling, quotas, or agent allocation;
- governed organizational knowledge, authority ranking, reconciliation,
  retention, or access policy;
- enterprise control-plane governance, managed credentials, billing,
  settlement, or commercial provider economics.

Local deterministic or reference implementations of those concepts may remain
useful and public. The production decision intelligence and compounding data
behind them are classified separately; an enterprise implementation may run in
the same physical environment as the OSS Runtime and still require an
enterprise license.

### Grandfathering and licensing

All features and code already released under Apache-2.0 remain Apache-2.0.
Their public availability, local behavior, and existing compatibility promises
are not withdrawn, reduced, or relicensed by this ADR. Public adaptive,
routing, memory, ledger, verification, and reference implementations are
grandfathered according to their current licenses and are maintained as
Runtime or reference capabilities.

The open-core boundary is drawn **forward only**. New production adaptive
intelligence, cross-task learning, fleet control, and organization governance
are developed in the private Enterprise/Control Plane surface even when an
on-premises deployment will execute them locally. Existing files such as
`rust/src/proxy/routing.rs` or `rust/src/core/adaptive.rs` are not retroactively
reclassified; future additions to those areas must be reviewed under the
classification rules.

`docs/contracts/local-free-invariant-v1.md` is deprecated for new
architectural decisions. It remains a historical, frozen contract for the
released behavior it documents. This ADR is the governing reference whenever
the old document and a new design question appear to conflict.

## Consequences

The OSS Runtime keeps a clear local value proposition: reads, shell, tools,
proxy BYOK, and local savings remain useful without hosted dependencies. The
business can offer proprietary scheduler and intelligence features in SaaS,
VPC, on-premises, and sovereign deployments without pretending that physical
placement changes intellectual-property ownership.

The boundary requires explicit A-E classification for new features and careful
separation between a public execution/reference primitive and a private
decision system. Maintainers must avoid accidentally expanding grandfathered
routers or learning modules into a new production intelligence surface.

Some users will see a commercial feature run beside or inside a local Runtime,
which can make the license distinction less obvious. Product and architecture
documentation must therefore name the capability class and license
independently from its deployment mode. A local Runtime also remains useful
when the commercial service is unavailable, but it is not required to recreate
organization-scale intelligence offline.

The old Local-Free CI and contract tests remain evidence for grandfathered
behavior until a separately approved migration changes them. This ADR does not
authorize removing those protections from existing features.

## Alternatives Considered

Keeping the Local-Free Invariant as the rule for all future work was rejected
because it would force every commercial capability that can run on-premises to
be free, confusing deployment with licensing and preventing a viable private
Control Plane.

Retroactively moving released local features behind an account, license, or
service was rejected because it would break the Apache-2.0 promise, damage
trust, and contradict the repository's existing contracts.

Making everything proprietary was rejected because the Runtime's inspectable
local reads, shell, tools, proxy, and ledger are the adoption and trust anchor.
They must remain independently useful for local coding.

Using deployment location as the boundary was rejected because the same
commercial scheduler may be delivered as SaaS or run in a customer's
air-gapped environment. The capability's responsibility and compounding
intelligence, not its process location, determine its class.
