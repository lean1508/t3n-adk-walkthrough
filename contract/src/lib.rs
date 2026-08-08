//! z-agent-approvals v0.1.0 — a verifiable record of human approvals for
//! agent actions.
//!
//! Written as the "go beyond the first contract" part of the Terminal 3 ADK
//! walkthrough. It is not a toy: it is the missing piece of a system we
//! actually run, described in `../USE-CASE.md`.
//!
//! The shape of the problem. An agent that opens pull requests on a person's
//! GitHub account must not do so without that person's approval. Today that
//! approval lives in the agent's own SQLite database — which means the
//! component being constrained is also the one holding the evidence that it
//! was constrained. Anyone auditing the agent has to take its word for it.
//!
//! Moving the record into a TEE contract changes three things:
//!
//!   1. The agent cannot forge an approval, because it does not own the store.
//!   2. The approval is scoped. Approving "work on this issue" is not the same
//!      as approving "publish this pull request", and the contract keeps them
//!      apart rather than collapsing both into one boolean.
//!   3. A third party can audit the trail without trusting the agent.
//!
//! # Storage layout
//!
//! Everything lives in the tenant KV map `z:<tid>:approvals`, keyed by
//! `<action-id>|<scope>`. The map name is built at runtime from
//! `tenant-context.tenant-did()` rather than hardcoded, so the same artifact
//! works for any tenant.
//!
//! # Capabilities
//!
//! `wit/world.wit` imports only tenant-context, logging and kv-store. There is
//! deliberately no HTTP import: this contract has no business reaching the
//! network, and under the ADK's model the WIT imports *are* the capability set,
//! so the host enforces that absence rather than trusting this comment.

#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

wit_bindgen::generate!({
    world: "agent-approvals",
    path: "wit",
    generate_all,
});

// Los bindings del host solo existen al compilar para wasm; en nativo se
// compila unicamente la logica pura, que es la que cubren los tests.
#[cfg(target_arch = "wasm32")]
use crate::host::{
    interfaces::{kv_store, logging},
    tenant::tenant_context,
};

pub const CONTRACT_VERSION: &str = "0.1.0";

/// Name of the tenant KV map, built from the tenant DID at runtime.
///
/// The DID arrives as raw bytes (the 20-byte CompactDid shape), so it is
/// hex-encoded to form a printable map name.
#[cfg(target_arch = "wasm32")]
fn approvals_map() -> String {
    let did = tenant_context::tenant_did();
    let mut hex = String::with_capacity(did.len() * 2);
    for b in did.iter() {
        hex.push_str(&format!("{:02x}", b));
    }
    format!("z:{}:approvals", hex)
}

/// Composite key: an approval is for one action *and* one scope.
///
/// Keeping the scope in the key is what stops "approved to prepare" being
/// replayed as "approved to publish".
fn approval_key(action_id: &str, scope: &str) -> Vec<u8> {
    format!("{}|{}", action_id, scope).into_bytes()
}

/// Minimal JSON string-field reader.
///
/// The contract deliberately does not pull in serde_json: the inputs are three
/// or four flat string fields, and every dependency in a TEE contract is
/// attack surface plus artifact size. This handles exactly `"key":"value"`
/// with no escaping, which is the whole input format, and returns None for
/// anything else rather than guessing.
fn json_str(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let start = body.find(&needle)? + needle.len();
    let rest = &body[start..];
    let colon = rest.find(':')?;
    let after = &rest[colon + 1..];
    let open = after.find('"')?;
    let tail = &after[open + 1..];
    let close = tail.find('"')?;
    Some(tail[..close].to_string())
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
struct Component;

#[cfg(target_arch = "wasm32")]
impl exports::z::agent_approvals::contracts::Guest for Component {
    /// A human records approval of one specific action, for one scope.
    fn record_approval(
        req: exports::z::agent_approvals::contracts::GenericInput,
    ) -> Result<Vec<u8>, String> {
        let raw = req.input.ok_or("record-approval: missing input")?;
        let body = core::str::from_utf8(&raw).map_err(|_| "input is not valid UTF-8")?;

        let action_id = json_str(body, "action-id")
            .ok_or("record-approval: missing required field 'action-id'")?;
        let approver = json_str(body, "approver")
            .ok_or("record-approval: missing required field 'approver'")?;
        let scope =
            json_str(body, "scope").ok_or("record-approval: missing required field 'scope'")?;
        let note = json_str(body, "note").unwrap_or_default();

        // An empty action-id would create an approval matching nothing, which
        // is worse than an error because it looks like it worked.
        if action_id.is_empty() || scope.is_empty() {
            return Err("record-approval: 'action-id' and 'scope' must be non-empty".to_string());
        }

        let value = format!(
            "{{\"approver\":\"{}\",\"scope\":\"{}\",\"note\":\"{}\"}}",
            json_escape(&approver),
            json_escape(&scope),
            json_escape(&note)
        );

        kv_store::put(
            &approvals_map(),
            &approval_key(&action_id, &scope),
            value.as_bytes(),
        )
        .map_err(|e| format!("record-approval: kv put failed: {}", e))?;

        let _ = logging::info(&format!(
            "approval recorded action={} scope={} by={}",
            action_id, scope, approver
        ));

        Ok(format!(
            "{{\"recorded\":true,\"action-id\":\"{}\",\"approver\":\"{}\",\"scope\":\"{}\"}}",
            json_escape(&action_id),
            json_escape(&approver),
            json_escape(&scope)
        )
        .into_bytes())
    }

    /// The agent asks whether it may proceed. It cannot answer this itself.
    fn check_approval(
        req: exports::z::agent_approvals::contracts::GenericInput,
    ) -> Result<Vec<u8>, String> {
        let raw = req.input.ok_or("check-approval: missing input")?;
        let body = core::str::from_utf8(&raw).map_err(|_| "input is not valid UTF-8")?;

        let action_id = json_str(body, "action-id")
            .ok_or("check-approval: missing required field 'action-id'")?;
        let scope =
            json_str(body, "scope").ok_or("check-approval: missing required field 'scope'")?;

        let found = kv_store::get(&approvals_map(), &approval_key(&action_id, &scope))
            .map_err(|e| format!("check-approval: kv get failed: {}", e))?;

        // A missing approval is a normal, expected answer — not an error.
        // Returning Err here would make "not approved" indistinguishable from
        // "the store is broken", and the agent must treat those differently.
        let approved = found.is_some();
        let _ = logging::info(&format!(
            "approval checked action={} scope={} approved={}",
            action_id, scope, approved
        ));

        let detail = match found {
            Some(bytes) => {
                let stored = String::from_utf8(bytes).unwrap_or_default();
                let approver = json_str(&stored, "approver").unwrap_or_default();
                let note = json_str(&stored, "note").unwrap_or_default();
                format!(
                    ",\"approver\":\"{}\",\"note\":\"{}\"",
                    json_escape(&approver),
                    json_escape(&note)
                )
            }
            None => String::new(),
        };

        Ok(format!(
            "{{\"approved\":{},\"action-id\":\"{}\",\"scope\":\"{}\"{}}}",
            approved,
            json_escape(&action_id),
            json_escape(&scope),
            detail
        )
        .into_bytes())
    }

    /// The audit trail.
    fn list_approvals(
        req: exports::z::agent_approvals::contracts::GenericInput,
    ) -> Result<Vec<u8>, String> {
        // `limit` is optional; the host rejects a scan limit of 0, so the
        // default has to be a real number rather than "unlimited".
        let limit: u32 = req
            .input
            .as_ref()
            .and_then(|raw| core::str::from_utf8(raw).ok())
            .and_then(|body| json_str(body, "limit"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let pairs = kv_store::scan(&approvals_map(), &[], &[0xff], limit)
            .map_err(|e| format!("list-approvals: kv scan failed: {}", e))?;

        let mut items: Vec<String> = Vec::with_capacity(pairs.len());
        for (key, value) in pairs.iter() {
            let key_s = String::from_utf8(key.clone()).unwrap_or_default();
            let val_s = String::from_utf8(value.clone()).unwrap_or_default();
            let (action_id, scope) = key_s.split_once('|').unwrap_or((key_s.as_str(), ""));
            items.push(format!(
                "{{\"action-id\":\"{}\",\"scope\":\"{}\",\"record\":{}}}",
                json_escape(action_id),
                json_escape(scope),
                if val_s.is_empty() { "null" } else { &val_s }
            ));
        }

        let _ = logging::info(&format!("approvals listed count={}", items.len()));
        Ok(format!(
            "{{\"approvals\":[{}],\"count\":{}}}",
            items.join(","),
            items.len()
        )
        .into_bytes())
    }
}

#[cfg(target_arch = "wasm32")]
export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        let parts: Vec<&str> = CONTRACT_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);
        for p in parts {
            assert!(p.parse::<u32>().is_ok());
        }
    }

    #[test]
    fn key_binds_action_to_scope() {
        // The whole security property: approving one scope must not produce
        // the key that another scope looks up.
        let prepare = approval_key("https://github.com/o/r/issues/1", "prepare");
        let publish = approval_key("https://github.com/o/r/issues/1", "publish");
        assert_ne!(prepare, publish);
    }

    #[test]
    fn reads_flat_json_fields() {
        let body = r#"{"action-id":"issue-1","approver":"leandro","scope":"publish"}"#;
        assert_eq!(json_str(body, "action-id").unwrap(), "issue-1");
        assert_eq!(json_str(body, "approver").unwrap(), "leandro");
        assert_eq!(json_str(body, "scope").unwrap(), "publish");
        assert!(json_str(body, "missing").is_none());
    }

    #[test]
    fn escaping_cannot_break_out_of_a_string() {
        let evil = r#"a" ,"approved":true,"x":"b"#;
        let escaped = json_escape(evil);
        assert!(!escaped.contains(r#"","#));
        assert!(escaped.contains(r#"\""#));
    }
}
