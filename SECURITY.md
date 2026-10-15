# security policy

brahma processes complex document and image formats and exposes CLI, JSON control, and MCP automation. treat files, paths, metadata, automation requests, and command parameters as potentially malicious.

## reporting a vulnerability

please give the repository maintainers a reasonable opportunity to investigate and coordinate a fix before publishing exploitable details, proof-of-concept code, or weaponized files.

this repository does not currently document a dedicated private security email or private reporting channel. contact the repository maintainers through the repository owner/maintainer channels already listed in the project, provide only a minimal non-exploitable summary in public, and ask for an appropriate private transfer method before sharing sensitive details. do not open a public issue containing a working exploit, secret, sensitive crash dump, or malicious attachment.

if no private channel can be established, withhold weaponized material and report the smallest safe description that allows maintainers to make contact.

## what to include

a useful report includes:

- affected brahma version, commit, or release artifact;
- operating system, architecture, build profile, and relevant feature flags;
- the affected component or path;
- security impact and required attacker access;
- exact reproduction steps;
- expected behavior and actual behavior;
- crash logs, backtraces, sanitizer output, or resource measurements when applicable;
- whether the problem reproduces from a clean checkout;
- any known workaround or containment.

remove credentials, personal paths, private document content, and unrelated system information from logs.

## malicious test files

do not attach a potentially harmful or non-redistributable file to a public issue. initially provide its format, size, cryptographic hash, observed effect, provenance category (for example, synthetic or fuzz-generated), and the command needed to reproduce the problem. coordinate a private transfer method with maintainers before sending the sample.

test files accepted into the repository must be minimized, free of personal or proprietary data, legally redistributable, and documented with the expected failure behavior. a fixed parser defect should receive a permanent regression test whenever safe and practical.

## scope

security reports may cover:

- PSD/PSB, raster, `.pcraft`, ICC, LUT, pattern, and embedded-object parsing;
- decompression bombs, oversized dimensions, integer overflow, memory exhaustion, recursion, or hangs;
- import/export and filesystem path handling, including traversal, symlinks, and unsafe overwrite behavior;
- engine command validation and serialization/deserialization;
- the desktop control server, headless JSON server, MCP server, and MCP-to-GUI bridge;
- authentication, authorization/capabilities, request limits, batch limits, and connection exhaustion;
- GPU resource validation or driver-facing input handling;
- dependencies, ci, packaging, signing, checksums, and release artifact integrity.

general bugs without a security impact should use the project's normal issue and contribution process.

## current limitations

TCP control connections require a 256-bit bearer token before method dispatch and enforce request-line, active-connection, and batch-step limits. authentication does not provide per-client or per-tool capabilities, so an authenticated client still receives the exposed control surface. remote filesystem methods use separately configured read and write workspace roots, and engine commands that still use ambient filesystem paths are denied at the automation boundary. automation can change individual safe preferences, but whole-preferences updates and filesystem-bearing preference sections are refused; applying the preferences dialog through automation is also refused. JSON-depth, render, document-memory, command-duration, client-identity, encryption, and audit controls are not yet implemented. see the [security documentation](book/src/security/overview.md) for the source-backed status and hardening roadmap.
