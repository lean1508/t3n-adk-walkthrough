# Bugs, friction and documentation gaps

Logged in the order we hit them, while onboarding onto the T3N ADK as
first-time users. Each entry says what we expected, what happened, and where.
Documentation issues are marked separately from product bugs, because they are
fixed by different people.

Nothing here is a complaint. A first run is the only time you get to see the
onboarding with fresh eyes, so we wrote everything down as it happened —
including the things that turned out to be our own mistake, since those tend to
point at a docs gap anyway.

---

## Pre-flight: before writing any code

### D-1 · The "About Terminal 3" page never mentions the ADK
**Type:** documentation · **Where:** https://docs.terminal3.io/intro/about-t3

Arriving at the docs root redirects to `/intro/about-t3`, which describes
Terminal 3 as a "data freedom company" and covers identity verification,
reusable KYC and national digital IDs. It does not mention the Agent Developer
Kit, TEE contracts or the SDK anywhere.

A developer who lands on the docs from the ADK bounty (or from a search) has
no path from that page to the thing they came for. The `llms.txt` index is far
more useful as an entry point than the human-facing landing page, which is an
odd inversion.

**Suggestion:** link the ADK overview from `about-t3`, or redirect developer
traffic to `/developers/adk/overview/what-is-adk`.

### D-2 · The bounty says "Rust contract"; the docs say "TEE contract"
**Type:** documentation / expectation mismatch

The bounty listing is titled *"deploy first RUST contract on the network"*,
which reads like a Solana program — especially on a Solana-ecosystem board.
What the ADK actually builds is a **TEE contract**: Rust compiled to a WASM
component, running in a trusted execution environment on the T3N network. The
dependency on `@bytecodealliance/jco` is the giveaway.

That is a more interesting product than the title suggests, and the mismatch
probably costs you developers who skip the bounty thinking it's another
"deploy a hello-world program" task.

### D-3 · The Quickstart doesn't deploy a contract, but the bounty asks for one
**Type:** documentation · **Where:** `/developers/adk/get-started/quickstart`

The bounty asks to "complete Quickstart and Walkthrough". The Quickstart ends
at a working authenticated call and explicitly states the contract is not
deployed at that stage — the contract work is the five-step Walkthrough
(write → build → register → invoke → test).

This is correct behaviour and the docs are clear about it; the friction is
that the two documents are named as if the Quickstart were the whole path.
Worth making the relationship explicit at the top of the Quickstart: *"this
gets you authenticated; the Walkthrough builds the contract."*

### D-4 · The API reference flags part of its own surface as unverified
**Type:** documentation · **Where:** `/developers/adk/reference`

The reference contains a section headed *"Observed in community code only —
not confirmed against official docs"*, warning that some method names were
reported by hackathon participants and should be treated as leads rather than
guarantees.

Being upfront about this is genuinely good practice and we would rather have
it than silent inaccuracy. Flagging it because it means a newcomer cannot rely
on the reference alone and has to check against the installed SDK's type
definitions — worth saying explicitly in the Quickstart's prerequisites.

### D-5 · The claim link redirects to a product page, not the claim form
**Type:** documentation / link · **Where:** https://go.terminal3.io/adk-community

The bounty points at `go.terminal3.io/adk-community` to "claim free tokens and
ID". It resolves to `terminal3.io/products/agent-developer-kit`, a product
page, rather than landing directly on the claim form. Minor, but it is the
very first step of the funnel.

### S-1 · The developer key is shown once and cannot be recovered
**Type:** product / onboarding UX · **Where:** claim page

The docs state: *"During the test phase, the key is shown only once and can't
be retrieved after you leave the page."*

Understandable for a test phase, and we handled it. Flagging it because it is
the single highest-risk moment in the funnel: a developer who closes the tab
has to re-register, and there is no obvious in-page warning before the fact
(the warning lives in the docs, which is not necessarily where they are).

**Suggestion:** a "copy key" confirmation step, or a one-time re-reveal.

---

## Step 1 — Claiming credentials

_To be filled in during the run._

## Step 2 — Quickstart: first authenticated call

_To be filled in during the run._

## Step 3 — Walkthrough: write, build, register, invoke, test the contract

_To be filled in during the run._

---

## Summary

_Counts and the short list of what would most improve the onboarding, written
once the run is complete._
