# installation

brahma is currently an early-alpha project. release artifacts, when available, are published through the repository's GitHub releases page. verify the release notes and checksums before running downloaded artifacts.

for source builds, continue to [building](building.md). the desktop application, headless CLI, and web target are separate workspace applications:

- `apps/brahma`: native desktop application
- `apps/brahma-cli`: headless CLI, JSON-lines server, and MCP server
- `apps/brahma-web`: WebAssembly application built with trunk

platform packaging and signing details are maintained in the existing [release documentation](https://github.com/shep95/brahma/blob/main/docs/releasing.md). signing is conditional on the configured release secrets; the existence of an artifact alone is not proof that it was signed.

## trusting input files

treat PSD, PSB, `.pcraft`, raster images, profiles, luts, patterns, and embedded document content as potentially malicious. review [secure file handling](../security/secure-file-handling.md) before using unknown files in privileged or sensitive environments.
