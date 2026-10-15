# brahma documentation

brahma is a native image editor written in rust. this book is the maintained entry point for installation, architecture, formats, automation, development, and security documentation.

the repository's existing `docs/` directory remains authoritative for detailed design notes, parity reports, ui conventions, release procedures, and the complete control-protocol method reference. the book links to those documents instead of copying them.

## start here

- users and evaluators: [installation](getting-started/installation.md)
- contributors: [building](getting-started/building.md) and [contributing](getting-started/contributing.md)
- architecture work: [architecture overview](architecture/overview.md)
- agents and automation clients: [automation overview](automation/overview.md)
- security-sensitive work: [security overview](security/overview.md) and [threat model](security/threat-model.md)

## documentation for coding agents

coding agents should read the following in order:

1. [`AGENTS.md`](https://github.com/shep95/brahma/blob/main/AGENTS.md) for repository rules and verification gates.
2. [`README.md`](https://github.com/shep95/brahma/blob/main/README.md) for product scope and current entry points.
3. [`book/src/architecture/`](architecture/overview.md) for crate boundaries and command flow.
4. [`book/src/automation/`](automation/overview.md) before driving the CLI, control protocol, or MCP server.
5. [`book/src/security/`](security/overview.md) before changing a parser, file i/o, automation, MCP, the control server, command execution, or serialization/deserialization.

agents must treat document bytes, paths, command parameters, and automation messages as untrusted input. a page that describes a proposed control does not mean that control is implemented.

## status language

security pages use four labels consistently:

- **implemented:** behavior confirmed in current source or tests.
- **known limitation:** an exposed risk or missing control in current source.
- **proposed:** a design direction that is not enforced by the current build.
- **future hardening:** useful follow-up work without an accepted implementation yet.
