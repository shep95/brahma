# contributing

read [`AGENTS.md`](https://github.com/shep95/brahma/blob/main/AGENTS.md) and the existing [contribution guide](https://github.com/shep95/brahma/blob/main/docs/contributing.md) before editing source. those files define the clean-room policy, crate layering, command checklist, testing requirements, naming, and visual verification rules.

important repository invariants include:

- user-visible behavior is implemented as an engine command unless it is pure shell/view state;
- public pixel paths cannot assume 8-bit data or srgb;
- command execution must return errors for adversarial input rather than panic;
- format work includes round-trip, malformed-input, and limit tests;
- l0-l6 changes must continue to compile for WebAssembly;
- third-party assets need a permissive license, a neighboring license file, and an `ATTRIBUTION.md` entry.

## security-sensitive contributions

before changing parsers, file i/o, serialization, automation, MCP, the control server, or command dispatch, read the [threat model](../security/threat-model.md) and [trust boundaries](../security/trust-boundaries.md). state whether a change adds an implemented control, documents a limitation, or proposes later hardening. add a regression test for every fixed security defect when a safe test is possible.

do not include live exploit payloads, secrets, personal files, or unlicensed samples in issues, commits, fixtures, or fuzz corpora. follow [vulnerability reporting](../security/vulnerability-reporting.md) for undisclosed vulnerabilities.
