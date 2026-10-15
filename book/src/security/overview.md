# security overview

brahma processes complex attacker-controlled files and exposes automation that can act with the user's filesystem permissions. security is therefore part of parser, engine, i/o, automation, release, and dependency design.

## current posture

| area | implemented protections | known limitations / future work |
|---|---|---|
| PSD/PSB | header, channel, decoded-byte, nesting, and pattern limits; malformed/property tests; fuzz targets | broader hostile corpus, aggregate work budgets, embedded-object coverage |
| raster codecs | default dimension/pixel/allocation limits; decoder-specific budgets; malformed/property tests; fuzz targets | continuous fuzzing and more format-specific semantic cases |
| `.pcraft` | manifest/blob/total-byte limits; bounded decompression; CRC/hash and version checks; fuzz target | tighter complexity/entry limits and directory-bundle path-race analysis |
| engine commands | error-return contract, focused bad-parameter tests, adversarial `panic_hunt` test | continue eliminating input-derived panic and exhaustion paths |
| control TCP | loopback binding; bearer-token authentication; rooted read/write authority; 1 mib request lines; 8 mib replies; 16 active connections; i/o timeouts | no general method capabilities, encryption, client identity, explicit JSON-depth policy, aggregate session-memory/command-duration budgets, or audit trail |
| MCP | stdio; typed tools; authenticated and bounded bridge; rooted read/write authority; 256-step batches with aggregate reply budgets; headless preview/output ceilings | no general tool authorization scopes, aggregate session-memory accounting, or command cancellation; transport reply caps do not bound desktop screenshot capture |
| dependencies | cargo lockfile and normal ci build/test/clippy | no `cargo-audit` or `cargo-deny` job/configuration confirmed in current tree |
| releases | checksums and platform signing/notarization paths when secrets are configured | signing can be absent; no SBOM generation confirmed in current tree |

the table is a snapshot of the current repository, not a guarantee about deployed artifacts. verify source and release evidence for high-risk use.

## security priorities

1. add capability-scoped automation sessions on top of the authenticated transport.
2. maintain the implemented workspace-handle boundary as file-bearing commands evolve.
3. extend the preview/reply budgets to JSON complexity, aggregate document/session memory,
   desktop capture/import/export, and command duration/cancellation.
4. expand PSD/PSB, codec, and `.pcraft` fuzzing with regression corpora.
5. add dependency advisories/license policy checks and release sboms.

start with the [threat model](threat-model.md), then use the focused pages for file handling, parsers, fuzzing, and automation.

## the standing security loop

`.github/workflows/security.yml` runs on every push and pull request and every monday:

- `cargo audit` against the rustsec advisory database, warnings as errors;
- `cargo deny` with `deny.toml`: advisories, licences, registry and git sources, wildcard versions;
- clippy with the unsafe-surface lints (`undocumented_unsafe_blocks`, `mem_forget`, pointer casts) as errors;
- gitleaks over the whole history, so a key or token never stays committed;
- a two-minute fuzz pass over every file parser (psd, codecs, affinity, the native format).

a failure there is a new pattern to patch, not noise: fix the dependency, the parser or the lint, then
the loop goes quiet again.
