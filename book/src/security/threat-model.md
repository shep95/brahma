# threat model

this threat model covers the source repository's desktop, CLI, WebAssembly, format, and automation surfaces. it assumes files, paths, JSON, MCP messages, agent decisions, and embedded metadata may be malicious.

## assets

- confidentiality of user documents, metadata, credentials, and unrelated local files;
- integrity of documents, preferences, filesystem contents, and application state;
- availability of brahma and the host system;
- integrity and provenance of release artifacts and dependencies;
- predictable command execution and document fidelity.

## threat actors

- an author of a malicious image, PSD/PSB, `.pcraft`, profile, LUT, pattern, or embedded object;
- a malicious or compromised local process connecting to a loopback server;
- a malicious, compromised, or over-privileged MCP client or automation agent;
- a project/action file that induces unsafe paths or expensive commands;
- a compromised dependency or build/release environment.

## entry points

```text
untrusted files and bytes
  PSD / PSB / raster / .pcraft / ICC / LUT / pattern / embedded content

untrusted control data
  CLI arguments / filesystem paths / JSON-lines / MCP requests / agent commands

untrusted build inputs
  dependencies / CI actions / release secrets / packaging tools
```

## primary threat scenarios

### hostile document parsing

a compact file declares oversized dimensions, excessive channels, nested descriptors, corrupted offsets, or highly expanding compressed data. desired outcome: reject it before unreasonable allocation or work, return an error, and retain a minimized regression test.

### filesystem authority abuse

an automation client supplies an absolute path, traversal sequence, symlink, junction, or replaced path to read or overwrite data outside the intended project. remote filesystem methods use separately configured directory capabilities, and engine commands that still require ambient paths fail closed at the automation boundary. this is not yet a general per-command capability model, and trusted desktop operations can still use ambient paths.

### local control takeover

another process on the same host connects to the loopback control server, discovers methods, drives ui input, executes commands, or opens/saves files. a 256-bit bearer token is required before method dispatch, but authentication does not grant per-client or per-tool capabilities.

### resource exhaustion

a client opens many TCP connections, sends an extremely long JSON line, creates a huge batch, requests full-size rendering, or repeatedly invokes expensive commands. request-line, connection, batch, and preview limits mitigate some cases; explicit JSON-depth, document-memory, and command-duration budgets remain absent.

### supply-chain compromise

a vulnerable or malicious crate, action, installer tool, or signing environment changes build behavior or artifacts. lockfiles and release checksums help reproducibility and verification, but advisory/license policy checks and SBOM output are future work.

## security objectives

- reject malformed and oversized data before allocation where possible;
- use checked arithmetic and bounded recursion at parser/command boundaries;
- make automation authority explicit, narrow, and revocable;
- keep file access inside authorized roots despite traversal and link tricks;
- bound work per request and per session;
- preserve sufficient non-secret audit data to investigate automation actions;
- turn every fixed security bug into a permanent regression test.

## non-goals and limits

brahma is not currently a sandbox for hostile automation or hostile native code. loopback transport, rust memory safety, typed schemas, and file-format checks do not replace authentication, authorization, os isolation, or operational least privilege. this document does not claim those missing controls are present.
