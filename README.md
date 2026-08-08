# Terminal 3 ADK — walkthrough, notes and bug reports

A first-timer's run through the [Terminal 3 Agent Developer Kit](https://docs.terminal3.io/developers/adk/overview/what-is-adk):
claiming credentials, getting the first authenticated call working, then
writing, building, registering, invoking and testing our own TEE contract.

Submitted for the Superteam Earn bounty
[*Create Agent ID, claim free tokens, & deploy first RUST contract*](https://superteam.fun/earn/listing/ai-id).

> **Disclosure:** this walkthrough was carried out with AI assistance (Claude),
> and every step was reviewed and run by a human before publishing. Terminal 3
> [publishes a skill file](https://docs.terminal3.io/developers/adk/support/ai-coding-assistants)
> specifically for AI coding assistants, which is what pointed us to the
> recommended order of operations — so this run doubles as a test of whether
> that skill file holds up in practice. It does: *"get a basic authenticated
> connection working FIRST"* is the right instruction, and the one place we
> deviated from it cost us time.

## Result

Completed end to end.

| Step | Outcome |
|---|---|
| Claim credentials | ✅ `did:t3n:c0e83772d154449476e743f694de71bf8cb94e5b` |
| Quickstart — first authenticated call | ✅ handshake 637 ms, authenticate 1310 ms |
| Write a TEE contract | ✅ `z-agent-approvals`, our own, not the sample |
| Build to a WASM component | ✅ 83,295 bytes, 4 unit tests passing |
| Register | ✅ `z:c0e8…:agent-approvals` → **contract id 508** (892 ms) |
| Invoke + test on testnet | ✅ all six scenarios, including the failure cases |

**15 findings** logged in [`BUGS.md`](BUGS.md): 9 documentation issues, 4
product/SDK bugs, 2 things worth crediting.

## What's here

| File | What it is |
|---|---|
| [`BUGS.md`](BUGS.md) | Every bug, friction point and documentation gap, in the order we hit them |
| [`quickstart/`](quickstart/) | Authenticate, register and invoke — with real outputs |
| [`contract/`](contract/) | `z-agent-approvals`, the TEE contract in Rust |
| [`USE-CASE.md`](USE-CASE.md) | Why we wrote *this* contract and what we'd build next |

## The contract

We wrote our own rather than deploying the flight-booking sample, because we
had a real gap to fill. `z-agent-approvals` is a verifiable record of human
approvals for agent actions: a human records approval for one action *and one
scope*, and the agent must ask the contract — not its own database — whether it
may proceed.

The property that matters, demonstrated live on testnet:

```
=== the human approves ONLY 'prepare' ===
  record-approval -> {"recorded":true, ..., "scope":"prepare"}

=== 'prepare' is approved, 'publish' still is not ===
  check-approval prepare -> {"approved":true,  ...}
  check-approval publish -> {"approved":false, ...}
```

Approving *"work on this"* cannot be replayed as approving *"publish this"*.
That is the whole point, it is the one unit test we would keep if we could keep
only one, and [`USE-CASE.md`](USE-CASE.md) explains why it is not hypothetical.

Two design choices, explained in [`contract/README.md`](contract/README.md):
**no HTTP import**, so "this ledger cannot reach the network" is enforced by
the host rather than promised in a comment; and **no serde**, which more than
halves the artifact against the reference implementation (83 KB vs 198 KB).

## Reproducing this

```bash
# 1. authenticate
cd quickstart && npm install
export T3N_API_KEY="<your key from the claim page>"
npx tsx quickstart.ts

# 2. build the contract
cd ../contract
rustup target add wasm32-wasip2
cargo test --release --target x86_64-unknown-linux-gnu
cargo build --release --target wasm32-wasip2

# 3. register, then create the map and invoke
cd ../quickstart
npx tsx register.ts          # prints your contract id
CONTRACT_ID=<id> npx tsx invoke.ts
```

You will need your own T3N ID and developer key from the
[claim page](https://docs.terminal3.io/developers/adk/get-started/prerequisites/request-test-tokens) —
ours are not in this repository, and neither should yours be.

Note that `quickstart.ts` includes a `trustAnchor` field the official snippet
omits, and `invoke.ts` creates a map ACL the walkthrough never mentions. Both
are required; see [B-1](BUGS.md) and [D-8](BUGS.md).

## Environment

Everything ran in a disposable GitHub Codespace rather than a local machine.
That is deliberate: the walkthrough's first instruction is to `npm install` an
SDK you have not audited, and a throwaway container is the right place to do
that. It also makes the run reproducible — same image, same Node version, no
local state.

| | |
|---|---|
| SDK | `@terminal3/t3n-sdk@4.30.0` |
| Node | v24.14.0 |
| Rust | 1.97.1, target `wasm32-wasip2` |
| Network | testnet — `https://cn-api.sg.testnet.t3n.terminal3.io` |
| Machine | 2-core Linux x86_64 (`basicLinux32gb`) |
