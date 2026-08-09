//! Public `model=auto` proxy surface.
//!
//! The wire contracts live in `lean_ctx_protocol` so the private
//! `lean-ctx-enterprise` control plane can interoperate without depending on
//! proxy internals. The OSS proxy intentionally contains no model/provider
//! selection intelligence; production decision intelligence is Class D and
//! remains in `lean-ctx-enterprise`.

pub use lean_ctx_protocol::auto_routing::{
    AutoRoutingConfig, AutoRoutingRequest, RoutingDecision, RoutingMode, RoutingReceipt,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_reexports_protocol_wire_types() {
        let request = AutoRoutingRequest {
            task_id: "task-1".to_owned(),
            requested_model: Some("model".to_owned()),
            routing_mode: RoutingMode::Shadow,
        };
        let json = serde_json::to_string(&request).expect("request serializes");
        let decoded: AutoRoutingRequest = serde_json::from_str(&json).expect("request parses");
        assert_eq!(decoded, request);
    }
}
