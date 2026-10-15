# command engine

every user-visible editing action is represented by a command specification in `crates/engine/src/*_cmds.rs` and registered by `crates/engine/src/commands.rs`. a specification supplies an id, label, menu path, shortcut, parameter documentation, enabled predicate, and `run` implementation.

```text
caller
  |
  | command id + JSON parameters
  v
command registry -- enabled/parameter checks --> command run closure
                                                   |
                                                   v
                                           Session + one history step
```

the desktop ui, CLI, control protocol, and MCP tools all dispatch into this registry. pure view/window state is the exception and is handled by the ui shell's command table.

## input contract

command parameters and current document state are untrusted at the command boundary. implementations must return `Err` for missing, wrong-type, out-of-range, or contradictory parameters. values derived from input must not reach unchecked indexing, division, allocation sizing, or panic paths.

the ignored `crates/engine/tests/panic_hunt.rs` integration test exercises commands with adversarial parameters. a changed command also needs focused graceful-failure tests and should be verified with:

```sh
cargo test -p brahma-engine
cargo test -p brahma-engine --test panic_hunt -- --ignored
```

the exact command-creation checklist is maintained in the existing [contribution guide](https://github.com/shep95/brahma/blob/main/docs/contributing.md).

## security boundary note

the registry validates command behavior, but current automation transports do not provide per-command capabilities. an MCP or control client that reaches command dispatch can request any exposed command whose runtime `enabled` predicate allows it. capability enforcement is [proposed automation hardening](../security/automation-security.md), not an implemented control.
