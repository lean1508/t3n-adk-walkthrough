# Use case: who vouches for the agent?

The bounty offers a bonus for going beyond the first contract and proposing a
use case. Ours is not hypothetical — it is a gap in a system that is running
today, and working through the ADK is what made the gap obvious.

## The system

We run an agent that hunts open-source bounties. It scans GitHub for issues
with a cash reward, filters out the ones that cannot be collected, and when its
operator approves one, it clones the repository, implements a fix, runs the
project's tests and prepares a pull request. The operator reviews the diff and
confirms; only then does the pull request go out under his GitHub account.

The design rule is that nothing irreversible happens without a human. Publishing
a pull request to a stranger's repository is irreversible in the way that
matters: it is public, it carries his name, and a bad one damages a reputation
that took years to build.

## The gap

That approval lives in the agent's own SQLite database.

The component being constrained is also the one holding the evidence that it
was constrained. That is fine while everything works, and worthless the moment
anyone asks a harder question:

- The agent has a bug and opens a pull request nobody approved. Afterwards, its
  own log says it was approved. Which is true?
- A maintainer asks whether a human actually reviewed the change. The only
  answer available is "our system says so".
- Six months later, someone audits which actions were authorised. They must
  trust the audited party's records.

None of these need malice. An ordinary bug produces the same situation, and the
system cannot tell the difference — which is the real problem.

## What the contract changes

`z-agent-approvals` moves the record out of the agent and into a TEE contract:

**The agent cannot forge an approval**, because it does not own the store. It
can write approvals only through a contract whose code is fixed and auditable.

**Approvals are scoped, not boolean.** Our pipeline has two distinct human
decisions — *work on this* (spend compute, clone the repo) and *publish this*
(irreversible, public, his name on it). Collapsing them into one flag is how
"I approved looking into it" becomes "I approved shipping it". The contract
keys approvals by `<action-id>|<scope>`, so one cannot stand in for the other.
That is the single unit test we would keep if we could keep only one.

**A third party can audit without trusting us.** `list-approvals` produces a
trail that a maintainer, or anyone else, can read independently.

## Why this belongs in a TEE rather than a database

A hosted database run by the same operator moves the problem without solving
it: whoever runs the agent still runs the store. What makes the difference is
that the contract's capability set is declared in `world.wit` and enforced by
the host. Ours imports `tenant-context`, `logging` and `kv-store` — and
notably **not** HTTP. That is not a promise in a comment; the host refuses to
load a contract importing an interface its world does not provide. Someone
evaluating the ledger can read forty lines of WIT instead of auditing the
agent.

That property — a capability boundary a reader can check in a minute — is what
we could not get any other way, and it is what the ADK is actually selling.

## What we would build next

1. **Wire it into the real pipeline.** Replace the SQLite approval check with
   `check-approval` before the publish step, and fail closed when the answer is
   `false` *or* unreachable.
2. **Approvals that expire.** An approval from three weeks ago should not
   authorise a publish today; the issue may have been closed or claimed. A
   `valid-until` field turns a standing permission into a decision with a
   shelf life.
3. **Bind the approval to the artifact.** Today `action-id` is the issue URL.
   Making it the hash of the diff the human actually read would mean approval
   covers *that* change and not whatever the agent produces afterwards.
   Approving a plan and approving an artifact are different acts, and only the
   second one is checkable.

Step 3 is the one we find genuinely interesting, and it is the reason this
belongs on a platform built around attestation rather than in a table with a
boolean column.
