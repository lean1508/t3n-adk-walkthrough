# Terminal 3 ADK — walkthrough, notes and bug reports

A first-timer's run through the [Terminal 3 Agent Developer Kit](https://docs.terminal3.io/developers/adk/overview/what-is-adk):
claiming credentials, getting the first authenticated call working, and
building, registering, invoking and testing a TEE contract.

Submitted for the Superteam Earn bounty
[*Create Agent ID, claim free tokens, & deploy first RUST contract*](https://superteam.fun/earn/listing/ai-id).

> **Disclosure:** this walkthrough was carried out with AI assistance (Claude),
> and every step was reviewed and run by a human before publishing. Terminal 3
> [publishes a skill file](https://docs.terminal3.io/developers/adk/support/ai-coding-assistants)
> specifically for AI coding assistants, which is what pointed us to the
> recommended order of operations — so this run doubles as a test of whether
> that skill file actually holds up in practice.

## What's here

| File | What it is |
|---|---|
| [`BUGS.md`](BUGS.md) | Every bug, friction point and documentation gap found, in the order we hit them |
| [`quickstart/`](quickstart/) | Step 1 — first authenticated call |
| [`contract/`](contract/) | The TEE contract in Rust |
| [`screenshots/`](screenshots/) | Evidence for each step |
| [`USE-CASE.md`](USE-CASE.md) | A concrete use case beyond the first contract |

## Environment

Everything runs in a disposable GitHub Codespace rather than a local machine.
That is a deliberate choice, not incidental: the walkthrough installs and
executes a third-party SDK, and a throwaway container is the right place to do
that the first time.

| | |
|---|---|
| SDK | `@terminal3/t3n-sdk` |
| Network | testnet (`setEnvironment("testnet")`) |
| Runtime | Node.js, ESM (`npm pkg set type=module`) |

## Reproducing this

See [`quickstart/README.md`](quickstart/README.md). You will need your own T3N
ID and developer key from the
[claim page](https://docs.terminal3.io/developers/adk/get-started/prerequisites/request-test-tokens) —
ours are not in this repository, and neither should yours be.

## Status

Work in progress. This section will list what was completed and what was not,
honestly, including anything we could not get working.
