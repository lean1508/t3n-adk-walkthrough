# `z-agent-approvals` — a TEE contract for human approval of agent actions

Our own contract, written after working through the reference implementation
([`z-tenant-flight`](https://github.com/Terminal-3/z-tenant-flight)). It is the
"go beyond the first contract" part of the bounty, and it is not a toy — it is
the missing piece of a system we actually run. The reasoning is in
[`../USE-CASE.md`](../USE-CASE.md).

## What it does

Three functions:

| Function | Who calls it | Why |
|---|---|---|
| `record-approval` | the human | Approve one specific action, for one scope |
| `check-approval` | the agent | Ask whether it may proceed — it cannot answer this itself |
| `list-approvals` | an auditor | The trail, after the fact |

Approvals are keyed by `<action-id>|<scope>`. Keeping the scope inside the key
is the point: approving *"work on this issue"* must not be replayable as
approving *"publish this pull request"*. There is a unit test for exactly that.

## Design decisions worth explaining

**No HTTP import.** `wit/world.wit` imports only `tenant-context`, `logging`
and `kv-store`. Under the ADK's model the WIT imports *are* the capability set
— the host refuses to load a contract that imports an interface its world does
not provide — so a contract that cannot reach the network is enforced rather
than promised. An approval ledger has no business making outbound calls, and
this way that claim is checkable by someone reading `world.wit` rather than
auditing the Rust.

**No serde.** The reference pulls in `serde` and `serde_json`; we parse three
flat string fields by hand instead. In a TEE contract each dependency is both
attack surface and artifact size, and the payoff is visible: **83 KB against
the reference's 198 KB**, for a contract with a comparable number of exports.

**A missing approval is not an error.** `check-approval` returns
`{"approved": false}`, not `Err`. If absence were an error the agent could not
distinguish *"you are not allowed"* from *"the store is unreachable"* — and
those must lead to different behaviour. Failing closed is only safe if you can
tell which failure you are in.

## Build

```bash
rustup target add wasm32-wasip2
cargo test --release --target x86_64-unknown-linux-gnu   # native: pure logic
cargo build --release --target wasm32-wasip2             # the component
```

The explicit `--target` on the test command is needed because `.cargo/config.toml`
pins `wasm32-wasip2` as the default; without it `cargo test` builds a `.wasm`
and then tries to execute it. See D-6 in [`../BUGS.md`](../BUGS.md).

## Result

```
test tests::key_binds_action_to_scope ... ok
test tests::escaping_cannot_break_out_of_a_string ... ok
test tests::reads_flat_json_fields ... ok
test tests::version_is_semver ... ok
test result: ok. 4 passed; 0 failed

   Compiling z-agent-approvals v0.1.0
    Finished `release` profile [optimized] target(s)

target/wasm32-wasip2/release/z_agent_approvals.wasm   83,295 bytes
```

| | Reference (`z-tenant-flight`) | This contract |
|---|---|---|
| Artifact | 197,968 bytes | **83,295 bytes** |
| Dependencies | wit-bindgen, serde, serde_json, hex | wit-bindgen only |
| Build (cold) | 32 s | 29 s |
| Host imports | 5 | 3 |

## Status

Complete. Registered on testnet as **contract id 508**
(`z:c0e83772d154449476e743f694de71bf8cb94e5b:agent-approvals`), invoked and
tested — see [`../quickstart/salida-invoke.txt`](../quickstart/salida-invoke.txt)
for the full session output.

Getting from *registered* to *running* needed one step the walkthrough never
mentions: a map created with an ACL naming the contract id. Written up as D-8
in [`../BUGS.md`](../BUGS.md).
