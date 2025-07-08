/// Trait for Policy Decision Point (PDP).
/// Determines whether a given action is authorized under some context.
pub trait PolicyEngine: Send + Sync {
    /// Evaluate action + context. Returns `true` if allowed.
    fn allow(&self, action: &str, context_json: &serde_json::Value) -> bool;
}

/// Allow-all policy (default).
#[derive(Debug, Clone)]
pub struct AllowAllPolicy;

impl PolicyEngine for AllowAllPolicy {
    fn allow(&self, _action: &str, _context_json: &serde_json::Value) -> bool {
        true
    }
}

#[cfg(feature = "opa_policy")]
mod opa_policy {
    use super::PolicyEngine;
    use opa_wasm::OpaPolicy;

    pub struct RegoPolicy {
        engine: OpaPolicy,
    }
    impl super::PolicyEngine for RegoPolicy {
        fn allow(&self, action: &str, ctx: &serde_json::Value) -> bool {
            let input = serde_json::json!({ "action": action, "ctx": ctx });
            match self.engine.evaluate(&input) {
                Ok(v) => v.as_bool().unwrap_or(false),
                Err(_) => false,
            }
        }
    }
} 