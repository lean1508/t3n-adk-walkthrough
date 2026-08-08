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

## Step 3 — Walkthrough: writing and building a contract

### D-6 · `cargo test` fails out of the box because the target is pinned
**Type:** documentation / template · **Severity:** low, but it hits everyone

The reference repo ships a `.cargo/config.toml` pinning `wasm32-wasip2` as the
default target. That is right for `cargo build` — it makes the documented build
command work unmodified — but it also makes `cargo test` compile the tests to
WASM and then try to run the `.wasm` as a binary:

```
error: test failed, to rerun pass `--lib`
Caused by: could not execute process (...)/z_agent_approvals-*.wasm (never executed)
Caused by: Permission denied (os error 13)
```

The fix is to name the host target explicitly:

```bash
cargo test --release --target x86_64-unknown-linux-gnu
```

Worth one line in the walkthrough, because the reference `Cargo.toml` keeps
`crate-type = ["cdylib", "lib"]` precisely so that "the business logic stays
unit-testable natively". The template invites you to write native tests and
then the default configuration stops you from running them.

### O-1 · Credit where due: the capability model is the best part
**Type:** observation, not a bug

Having a contract's capability set *be* its WIT imports, enforced by the host
at load time, is a genuinely good design. It let us make a claim a reader can
check — *this ledger cannot reach the network* — by reading forty lines of WIT
instead of auditing our Rust. Most platforms would have made that a paragraph
in a README.

Flagging it because the docs undersell it: the capability pages are filed under
**tips**, which is not where a first-time reader goes looking for the security
model. This deserves to be in the ADK overview, above the fold.

## Step 4 — Register, invoke and test on the network

### D-7 · `tenant.me()` is in the docs but not in the SDK
**Type:** documentation bug · **Severity:** medium
**Where:** `/developers/adk/get-started/prerequisites/set-up-dev-env`

The Set Up Dev Env page ends with a liveness check:

```typescript
await tenant.me(); // throws if something's wrong; confirms the client actually works
```

`me()` does not exist on `TenantClient` in `@terminal3/t3n-sdk@4.30.0`:

```
TypeError: tenant.me is not a function
```

The class exposes `tenant`, `maps`, `contracts`, `token`, plus `admitForOrg`,
`canonicalName`, `executeControl` and friends — no `me`. This is the last line
of the page every developer runs before moving on to contracts, so it fails at
the worst possible moment: right after a long setup, with nothing built yet to
suggest whether the problem is you or the SDK.

**Workaround** — any real read works as a liveness check:

```typescript
const contracts = await tenant.contracts.list();   // [] on a fresh tenant
```

This looks like the same drift the reference page warns about in its
"Observed in community code only" section (D-4). Worth a docs test that
type-checks the published snippets against the shipped `.d.ts`.

### B-4 · `maps.create` warns about a footgun and then crashes on it
**Type:** SDK bug · **Severity:** medium

Calling `maps.create({ tail })` without `readers` produces a good warning
immediately followed by a useless crash:

```
[t3n-sdk] maps.create("approvals"): no `readers` specified — the map will be
created with a deny-all read policy, so no one (including you) can read it.
Pass `readers` explicitly (e.g. "all" or { only: [...] }), or { only: [] } to
deliberately make it write-only.

TypeError: Cannot read properties of undefined (reading 'toLowerCase')
```

The SDK clearly knows the field is missing — it says so, in detail. Then
something downstream calls `.toLowerCase()` on an absent value and throws a
`TypeError` that names neither the field nor the call.

Two things worth separating here. The **warning text is excellent**: it
explains the consequence, not just the omission, and it gives three concrete
options. That is better than most SDKs manage. The problem is only that the
code path behind it does not survive the case it warns about. The same is true
of `visibility`, which is also required but typed as bare `string` in the
`.d.ts` — we found `"private"` works by trying it.

**Suggestion:** validate `MapCreateInput` up front and throw one error naming
the missing fields, or apply the deny-all default the warning already
describes. Either is fine; crashing after correctly diagnosing the problem is
not.

### D-8 · Nothing tells you the contract needs a map ACL before it can run
**Type:** documentation · **Severity:** medium — this is the step that blocks you

A registered contract that writes to a tenant KV map does not work until a map
exists **with an ACL naming the contract's numeric id**. Until then every call
fails at the host boundary:

```
access denied: TenantContract(did:t3n:c0e8…/508) cannot write map
"z:c0e8…:approvals"
```

The register page mentions in passing that the id is "required in the next
setup step when you create map ACLs", but the walkthrough never shows that
step, and the ACL shape (`writers: { only: [contractId] }`) is only discoverable
from the `.d.ts`. This is the single place where a working contract looks
broken, and the error — while accurate and pleasantly specific about *which*
principal was denied on *which* map — does not say "create the map".

**What worked:**

```typescript
await tenant.maps.create({
  tail: "approvals",
  visibility: "private",
  writers: { only: [contractId] },   // the id from contracts.register()
  readers: { only: [contractId] },
});
```

**Suggestion:** a short "4. Create the map your contract writes to" between
register and invoke would remove the only genuine wall in the walkthrough.

### O-2 · Credit: the errors from inside the contract come back intact
**Type:** observation

Once past the ACL, the developer experience is good. Errors raised inside the
Rust contract arrive at the caller with the message preserved and a correlation
id attached:

```
RPC Error: contract error: check-approval: missing required field 'scope'
[9415c963-d3ac-4069-b8de-ae7d5966e7c3]
```

That is our own string, from our own `Err`, across a WASM boundary, a TEE and
an RPC hop — and the id makes a support conversation possible. Given how much
of this report is about error handling, it is worth saying that this part is
right.

Latencies were unremarkable in the good sense: registration 892 ms for an
83 KB component, and contract calls between 238 and 937 ms.

---

## Summary

Fifteen entries: **9 documentation issues**, **4 product/SDK bugs**, and **2
things worth crediting**. The walkthrough was completed end to end — first
authenticated call, own contract written, compiled, registered as contract id
508, invoked and tested on testnet.

### The three that cost the most time

| | What | Why it hurts |
|---|---|---|
| **B-1** | The Quickstart snippet omits the required `trustAnchor` and crashes | It is the first code anyone runs, and the easiest fix disables MITM protection |
| **D-8** | Nothing says a contract needs a map ACL naming its contract id | The only real wall; a working contract looks broken |
| **D-7** | `tenant.me()` is documented but does not exist | Fails at the end of setup, before you have built anything to blame |

### If only three things get fixed

1. **Add `trustAnchor` to the Quickstart snippet** with one line about what the
   unsafe opt-out gives up. Cheapest fix here, and the only one with a security
   consequence.
2. **Add a "create the map your contract writes to" step** between register and
   invoke, showing `writers: { only: [contractId] }`. Removes the wall.
3. **Type-check the published snippets against the shipped `.d.ts`.** B-1, D-7
   and the `visibility: string` type are all the same class of drift, and a CI
   check would have caught all three.

### What is genuinely good, and we mean it

The **capability model** (O-1) — a contract's permissions *being* its WIT
imports, enforced at load time — let us make a claim a reader can verify in
forty lines of WIT rather than by auditing our Rust. The **error propagation**
(O-2) brings contract errors back intact with a correlation id. And the
**warning text** in `maps.create` explains consequences rather than just naming
a missing field, which is rarer than it should be.

The gap between how carefully the SDK's type documentation is written and how
stale the published snippets are is the striking thing about this onboarding.
The knowledge is clearly there — it just is not in the pages a newcomer reads
first.
