# roadmap

the live product roadmap, milestone definitions, parity measurements, and current focus are maintained in [`docs/roadmap.md`](https://github.com/shep95/brahma/blob/main/docs/roadmap.md). menu coverage is generated into [`docs/parity-checklist.md`](https://github.com/shep95/brahma/blob/main/docs/parity-checklist.md) by `cargo xtask parity`.

security hardening is cross-cutting rather than a replacement for those product milestones. the current security sequence is:

1. threat model and reporting policy;
2. authenticated control sessions;
3. capability-scoped MCP/control access;
4. workspace-rooted filesystem handles;
5. request, batch, render, and connection budgets;
6. broader parser fuzzing and malicious-input regression corpora;
7. dependency policy and release SBOM/provenance.

only the documentation foundation in item 1 is established by this book. later controls remain proposed until implemented and tested.
