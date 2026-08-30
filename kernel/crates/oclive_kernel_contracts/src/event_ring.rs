//! Event Ring module and registration ports.

use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_types::{EventEnvelope, EventModuleDeclaration, EventModuleOutput, Result};

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
///         .event_module_declarations()
///         .into_iter()
///         .map(|declaration| declaration.module_id)
///         .collect()
/// }
/// ```
pub trait EventModuleRegistrar: Send + Sync {
    /// Registers one module declaration and implementation.
    ///
    /// # Errors
    ///
    /// Returns a stable validation message for malformed declarations or conflicting module IDs.
    fn register_event_module(
        &self,
        module: Arc<dyn EventModule>,
    ) -> std::result::Result<(), String>;

    fn event_module_declarations(&self) -> Vec<EventModuleDeclaration>;
}
