# secure file handling

brahma reads and writes documents in the desktop app, CLI, headless RPC server, and MCP server.
remote automation paths are untrusted relative names resolved beneath explicitly granted roots.

## implemented

- format loaders return structured errors for i/o and malformed data.
- `.pcraft` ZIP loading validates archive offsets, entry sizes, crcs, content hashes, and configured decompressed-size budgets.
- raster and PSD loaders apply parser/decode limits before major pixel allocation.
- native `.pcraft` saves use temporary-file/rename patterns in relevant store paths.
- `AuthorizedWorkspace` holds independent read and write directory capabilities for remote
  automation.
- absolute paths, parent traversal, alternate separators, drive/device/stream prefixes, reserved
  windows device names, empty or dot components, and link escapes are rejected before file
  effects.
- non-existent output files are supported when their parent already exists beneath the write
  root. automation does not create caller-selected directory trees.
- filesystem-bearing engine commands that have not migrated to capability i/o fail closed for
  automation. trusted local CLI subcommands and user-driven desktop file pickers keep their
  intended ambient authority.

## known limitations

- directory `.pcraft` bundles have not been documented as resistant to link replacement or time-of-check/time-of-use races.
- root grants are process launch configuration, not per-document grants.
- general method capabilities and structured filesystem audit events are not implemented.
- existing hard links inside a granted root are trusted root contents; the path policy cannot
  infer how they were created.

## capability-based design

prefer an opened directory capability or workspace handle over repeated string sanitization:

```text
AuthorizedWorkspace
  - open_document(relative_path)
  - save_document(relative_path)
  - export_render(relative_path)
```

the handle is created from an explicit launch-time grant, preserves separate read/write
permissions, rejects absolute and escaping paths, and performs operations relative to the held
directory capability. new files require an existing in-root parent because a non-existent target
cannot be canonicalized safely.

## design requirements

- canonicalization alone is insufficient: links can change after validation.
- policy should operate on handles and relative components where platform apis allow it.
- reads and writes should have distinct capabilities.
- replace, create-new, truncate, and directory creation semantics must be explicit.
- logs should avoid file contents and should minimize disclosure of full private paths.
- tests should cover `..`, absolute paths, mixed separators, case behavior, symlinks/junctions, non-existent targets, and replacement races on supported platforms.

os-account, vm, container, or sandbox isolation remains appropriate defense in depth for
untrusted automation and for controls not yet covered by general method capabilities.
