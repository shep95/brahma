# release security

brahma's release workflow builds platform packages and a draft GitHub release from the `release` branch. the authoritative process is [`docs/releasing.md`](https://github.com/shep95/brahma/blob/main/docs/releasing.md).

## implemented workflow elements

- platform-specific signing/notarization steps exist for macos and windows;
- signing secrets are scoped through the GitHub `release` environment;
- release artifacts include `SHA256SUMS.txt`;
- the final release job receives `contents: write`, while build jobs use narrower access;
- artifacts remain in a draft release until a maintainer publishes them.

signing is conditional. the workflow can produce unsigned artifacts when signing material is unavailable and emits warnings. consumers and maintainers must verify the job summary and artifact signature rather than assuming every release is signed.

## future hardening

- generate a machine-readable SBOM for each release;
- publish provenance/attestations tied to the source commit and workflow;
- make signature state explicit in release metadata;
- verify checksums, signatures, notarization, and installer behavior before publication;
- review action pinning and release-tool provenance;
- document key rotation and compromise response.

building, signing, drafting, publishing, and deploying are separate states. a successful build must not be reported as a published release.
