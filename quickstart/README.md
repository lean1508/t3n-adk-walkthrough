# Step 1 — First authenticated call

Goal of this step, per the official Quickstart: *one working, authenticated
call — nothing else.* No Rust, no WASM, no contract yet.

## What we ran

```bash
mkdir quickstart && cd quickstart
npm init -y
npm pkg set type=module          # the sample uses top-level await
npm install @terminal3/t3n-sdk tsx
export T3N_API_KEY="$(cat /path/to/your/key)"
npx tsx quickstart.ts
```

The code is in [`quickstart.ts`](quickstart.ts). It is the official snippet
**plus one field the docs omit** (`trustAnchor`) — without it the call crashes.
See [B-1](../BUGS.md) for why that matters more than a normal typo in a doc.

## Result

```
loadWasmComponent(): 108 ms
direccion derivada de la key: 0x344fa5a8680ca5253f576cf0390d071aa880d278
handshake(): 637 ms
authenticate(): 1310 ms
Connected as: did:t3n:c0e83772d154449476e743f694de71bf8cb94e5b
```

Full output: [`salida-quickstart.txt`](salida-quickstart.txt).

The tenant DID matches the one shown on the claim page, which is the check the
docs ask for — they warn never to derive or hardcode the DID, but to read it
back from the authenticated session.

## Timings

Measured on a 2-core Codespace (`basicLinux32gb`), testnet, single run:

| Step | Time |
|---|---|
| `npm install` (cold) | 21 s |
| `loadWasmComponent()` | 108–222 ms |
| `handshake()` | 637 ms |
| `authenticate()` | 1310 ms |

Nothing here is slow enough to be a problem. Noting it because the Quickstart
promises "under 10 minutes" and, once you know about `trustAnchor`, the code
really does run in about two seconds — the ten minutes are all install and
reading.

## What we hit on the way

Three things cost real time; all are written up in [`BUGS.md`](../BUGS.md):

1. **B-0** — the copied key had one extra character, and the key is shown only
   once. Validate before you close that tab.
2. **B-1** — the documented snippet is missing a required field and crashes
   with an unhelpful `TypeError`.
3. **B-2** — SDK errors bury the message under ~1.5 MB of obfuscated bundle.

## Note on the environment

This ran inside a disposable GitHub Codespace, not on a personal machine. When
the first step of a walkthrough is `npm install` of an SDK you have not audited,
a throwaway container is the right place to do it. It also makes the run
reproducible: same image, same Node version, no local state.
