//! Event Ring module and registration ports.

use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_types::{
    EventDispatchResult, EventDraft, EventEnvelope, EventModuleDeclaration, EventModuleOutput,
    EventModuleRegistryEntry, EventModuleRegistryPolicy, Result,
};

/// A source-bound handle that submits drafts to the authoritative Event Ring.
///
/// The handle owns the registered source identity and weight. Callers supply only event content
/// and turn/stream correlation, so a module cannot impersonate another registered source.
///
/// # Examples
///
/// ```
/// use oclive_kernel_contracts::EventEmitter;
///
/// fn accepts_emitter(_emitter: &dyn EventEmitter) {}
/// ```
#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// Submits one root event draft from the bound module.
    ///
    /// # Errors
    ///
    /// Returns an error when the draft violates the module emission declaration or Event Ring
    /// bounds, or when a subscribed module fails dispatch.
    async fn emit(
        &self,
        stream_key: &str,
        correlation_id: Option<&str>,
        draft: EventDraft,
    ) -> Result<EventDispatchResult>;
}

/// A module that declaratively joins the Event Ring.
///
/// Implementations receive events matching their declaration. They cannot mutate envelope
/// identity fields directly; instead they return a payload replacement, metadata additions, and
/// optional child emissions through [`EventModuleOutput`].
///
/// # Examples
///
/// ```
/// use async_trait::async_trait;
/// use oclive_kernel_contracts::EventModule;
/// use oclive_kernel_types::{EventEnvelope, EventModuleDeclaration, EventModuleOutput};
///
/// struct Observer;
///
/// #[async_trait]
/// impl EventModule for Observer {
///     fn declaration(&self) -> EventModuleDeclaration {
///         EventModuleDeclaration {
///             module_id: "example.observer".into(),
///             subscriptions: vec!["kernel.chat.*".into()],
///             priority: 100,
///             ..Default::default()
///         }
///     }
///
///     async fn handle(
///         &self,
///         _event: &EventEnvelope,
///     ) -> oclive_kernel_types::Result<EventModuleOutput> {
///         Ok(EventModuleOutput::default())
///     }
/// }
/// ```
#[async_trait]
pub trait EventModule: Send + Sync {
    fn declaration(&self) -> EventModuleDeclaration;

    /// # Errors
    ///
    /// Returns an error when the module cannot safely process the event. Dispatch is fail-fast so
    /// a partially transformed primary event is never mistaken for a complete result.
    async fn handle(&self, event: &EventEnvelope) -> Result<EventModuleOutput>;
}

/// Registration port for modules that join a kernel-owned Event Ring.
///
/// # Examples
///
/// ```
/// use oclive_kernel_contracts::EventModuleRegistrar;
///
/// fn registered_ids(registrar: &dyn EventModuleRegistrar) -> Vec<String> {
///     registrar
///         .event_module_registry()
///         .into_iter()
///         .map(|entry| entry.declaration.module_id)
///         .collect()
/// }
/// ```
pub trait EventModuleRegistrar: Send + Sync {
    /// Registers a module with the registry's default policy, returning its source-bound emitter.
    ///
    /// # Errors
    ///
    /// Returns a stable validation message for malformed declarations or conflicting module IDs.
    fn register_event_module(
        &self,
        module: Arc<dyn EventModule>,
    ) -> std::result::Result<Arc<dyn EventEmitter>, String>;

    /// Registers a module with a trusted registry policy.
    ///
    /// # Errors
    ///
    /// Returns a stable validation message for malformed declarations/policies or conflicting
    /// module IDs. The supplied policy is stored independently from the module declaration.
    fn register_event_module_with_policy(
        &self,
        module: Arc<dyn EventModule>,
        policy: EventModuleRegistryPolicy,
    ) -> std::result::Result<Arc<dyn EventEmitter>, String> {
        if policy != EventModuleRegistryPolicy::default() {
            return Err("event registrar does not support explicit registry policy".into());
        }
        self.register_event_module(module)
    }

    /// Returns a deterministic read-only snapshot of admitted declarations and registry policy.
    fn event_module_registry(&self) -> Vec<EventModuleRegistryEntry> {
        self.event_module_declarations()
            .into_iter()
            .map(|declaration| EventModuleRegistryEntry {
                declaration,
                policy: EventModuleRegistryPolicy::default(),
            })
            .collect()
    }

    /// Compatibility view for callers interested only in module capabilities.
    fn event_module_declarations(&self) -> Vec<EventModuleDeclaration>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LegacyModule;
    struct LegacyRegistrar;

    #[async_trait]
    impl EventModule for LegacyModule {
        fn declaration(&self) -> EventModuleDeclaration {
            EventModuleDeclaration {
                module_id: "test.legacy".into(),
                emissions: vec!["kernel.test.legacy".into()],
                ..Default::default()
            }
        }

        async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
            Ok(EventModuleOutput::default())
        }
    }

    impl EventModuleRegistrar for LegacyRegistrar {
        fn register_event_module(
            &self,
            _module: Arc<dyn EventModule>,
        ) -> std::result::Result<Arc<dyn EventEmitter>, String> {
            Err("legacy register called".into())
        }

        fn event_module_declarations(&self) -> Vec<EventModuleDeclaration> {
            vec![LegacyModule.declaration()]
        }
    }

    #[test]
    fn legacy_registrar_gets_default_policy_snapshot_without_new_methods() {
        let entries = LegacyRegistrar.event_module_registry();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].declaration.module_id, "test.legacy");
        assert_eq!(
            entries[0].policy.influence_weight_bps,
            oclive_kernel_types::EVENT_INFLUENCE_WEIGHT_SCALE
        );
    }

    #[test]
    fn legacy_registrar_rejects_explicit_non_default_policy() {
        let result = LegacyRegistrar.register_event_module_with_policy(
            Arc::new(LegacyModule),
            EventModuleRegistryPolicy {
                influence_weight_bps: 7_500,
            },
        );

        assert_eq!(
            result.err().as_deref(),
            Some("event registrar does not support explicit registry policy")
        );
    }
}
