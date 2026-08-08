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

### S-2 · The "API key" is actually a private signing key
**Type:** product / security naming · **Where:** claim page, SDK

The credential handed out at the claim page is called an *API key* and the SDK
parameter is named `T3N_API_KEY`. It is not an API key. It is a **secp256k1
private key** (`0x` + 64 hex): the SDK derives an Ethereum address from it with
`eth_get_address(T3N_API_KEY)` and signs with it via `metamask_sign(...)`.

That naming matters because developers treat the two categories very
differently. API keys routinely end up in client-side env vars, CI logs,
`.env.example` files, screenshots and support chats. A private signing key
should never go anywhere near those places. Calling it an API key invites
exactly the handling it must not get.

**Suggestion:** call it a *developer signing key* in the UI, the docs and the
variable name (`T3N_SIGNING_KEY`), and say on the claim page what it can do if
leaked.

### B-0 · A stray character in the copied key produces an unrecoverable state
**Type:** product / onboarding · **Severity:** high in combination with S-1

Our copied key came out **65 hex characters instead of 64**, with one extra
character appended. Combined with S-1 (the key is shown once and cannot be
retrieved), a single bad copy leaves a developer with a credential that fails
and no way to get a good one short of re-registering.

The SDK's error is good — it validates and redacts:

```
Error: Invalid Ethereum private key (INVALID_ARGUMENT): 0x…… [redacted]
```

Credit where due: redacting the key in the error message is the right call and
not everyone does it.

**Suggestions:** validate length and charset on the claim page before showing
the key as copyable; offer a "copy" button rather than manual selection; and
allow one re-reveal during the test phase.

## Step 2 — Quickstart: first authenticated call

### B-1 · ⚠️ The official Quickstart code crashes — it omits a required field
**Type:** documentation bug with security consequences · **Severity:** high
**Where:** https://docs.terminal3.io/developers/adk/get-started/quickstart
**SDK:** `@terminal3/t3n-sdk@4.30.0`, Node v24.14.0, Linux x86_64

Copying the Quickstart's `T3nClient` snippet verbatim and running it fails:

```
TypeError: Cannot read properties of undefined (reading 'unsafe_trust_server')
    at isUnsafeTrustServer (...)
    at assertNodeTrusted (...)
    at async T3nClient.handshake (...)
```

The cause is that `T3nClientConfig` requires a `trustAnchor` field which **the
Quickstart snippet does not include**. The SDK's own type definitions are
emphatic that it is mandatory:

> **Required.** Client-pinned trust anchor the node's DKG attestation is
> verified against before the handshake trusts its ML-KEM key (SP-003) […]
> Without this, a network attacker with their own TDX VM can hand the SDK a
> forged-but-valid attestation for a key it controls and read every session.
> It is a required field precisely so no caller can omit it by accident —
> bypassing verification must be a visible, grep-able choice.

**Why this is worse than an ordinary docs gap.** The design intent is sound:
make the unsafe path a deliberate, greppable choice. But because the Quickstart
omits the field, every newcomer meets `trustAnchor` for the first time as *"the
thing I must add to stop the crash"*. The quickest way out — the one visible in
the type docs and the one we used — is `{ unsafe_trust_server: true }`, which
disables exactly the MITM protection the field exists to enforce. The omission
converts a considered security decision into a copy-paste fix. It would not
surprise us if a meaningful share of testnet integrations now carry that flag
into production.

**Fix that unblocks the Quickstart (what we used, testnet only):**

```typescript
const t3n = new T3nClient({
  wasmComponent,
  handlers: { EthSign: metamask_sign(address, undefined, T3N_API_KEY) },
  trustAnchor: { unsafe_trust_server: true },   // ← missing from the docs
});
```

**Suggestions**, in order of value:

1. Add `trustAnchor` to the Quickstart snippet, with one line saying what the
   unsafe opt-out gives up and a pointer to the real `TrustAnchor` for
   production.
2. Throw a purposeful error. `Cannot read properties of undefined` gives the
   developer nothing; the field is documented as required, so the SDK should
   say `T3nClientConfig.trustAnchor is required — see <url>`. A required field
   deserves a required-field error.
3. Consider naming the opt-out so it stays uncomfortable in a code review, e.g.
   `unsafe_trust_server_TESTNET_ONLY`.

### B-2 · An SDK error dumps 1.5 MB of obfuscated bundle to the console
**Type:** product / DX · **Severity:** medium

Any uncaught error inside the SDK prints the offending line of
`dist/index.esm.js` — which is minified and **name-obfuscated** into a single
line — so the terminal fills with ~1.5 MB of `_0x1e6e(0x63d)`-style code. The
actual message and stack are buried at the end, and on a normal terminal
scrollback they are simply lost.

We had to filter the output (`grep -vE '_0x[0-9a-f]{4}'`) to read our own
errors, which is not something a first-time user will think to do.

Two separate observations here:

- **Ship a source map**, or at least don't emit a single-line bundle; either
  one makes stack traces readable.
- **Obfuscation in a security SDK is worth reconsidering.** This library
  handles a private signing key and attestation verification. Identifier
  mangling does not protect anything that runs on the client, and it does stop
  users auditing what the library does with their key. For a product whose
  pitch is verifiable trust, shipping unreadable code works against the pitch.

### B-3 · Four vulnerabilities in the dependency tree, one critical
**Type:** supply chain · **Severity:** critical (transitive)

A clean `npm install @terminal3/t3n-sdk tsx` reports:

```
4 vulnerabilities (3 moderate, 1 critical)
```

The critical one is **Zip Slip / arbitrary file write during archive
extraction** in `decompress`, reached through:

```
@terminal3/t3n-sdk@4.30.0
└─ @bytecodealliance/jco@1.27.0
   └─ @bytecodealliance/componentize-js@0.22.0
      └─ @bytecodealliance/weval@0.4.1
         └─ decompress@4.2.1     ← GHSA-mp2f-45pm-3cg9 (critical)
                                   GHSA-h39j-r5qq-r9mm (moderate)
```

`npm audit` reports `fixAvailable: true` for all four, so this looks like a
dependency bump on the `jco` line rather than anything structural.

Worth prioritising for two reasons beyond the CVE itself: it is the **first
thing a new developer sees** after installing (a critical warning on step one
is not the first impression you want), and the vulnerability class — writing
files outside the extraction directory — is a poor fit for a product built on
trusted execution.

## Step 3 — Walkthrough: write, build, register, invoke, test the contract

_To be filled in during the run._

---

## Summary

_Counts and the short list of what would most improve the onboarding, written
once the run is complete._
