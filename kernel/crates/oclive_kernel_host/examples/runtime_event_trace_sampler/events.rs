use std::sync::Arc;

use anyhow::bail;
use async_trait::async_trait;
use oclive_kernel_contracts::{EventEmitter, EventModule, EventModuleRegistrar};
use oclive_kernel_host::domain::event_ring::ProactiveTurnPermit;
use oclive_kernel_host::OcliveKernel;
use oclive_kernel_types::models::dto::TurnOrigin;
use oclive_kernel_types::{
    EventDraft, EventEnvelope, EventModuleDeclaration, EventModuleOutput,
    EventModuleRegistryPolicy, ProactiveSignalKind, ProactiveTurnProposal,
    PROACTIVE_TURN_PROPOSED_EVENT_KIND,
};

use super::contract::SampleContract;

const TRACE_SINGLE_KIND: &str = "kernel.sample.trace.single";
const TRACE_DERIVE_KIND: &str = "kernel.sample.trace.derive";
const TRACE_DERIVED_KIND: &str = "kernel.sample.trace.derived";
const TRACE_AFTER_RESTART_KIND: &str = "kernel.sample.trace.after_restart";

struct SampleTraceSource;

#[async_trait]
impl EventModule for SampleTraceSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "sample.trace_source".into(),
            emissions: vec![
                TRACE_SINGLE_KIND.into(),
                TRACE_DERIVE_KIND.into(),
                TRACE_AFTER_RESTART_KIND.into(),
            ],
            ..Default::default()
        }
    }

    async fn handle(
        &self,
        _event: &EventEnvelope,
    ) -> oclive_kernel_types::Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
}

struct SampleTraceDeriver;

#[async_trait]
impl EventModule for SampleTraceDeriver {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "sample.trace_deriver".into(),
            subscriptions: vec![TRACE_DERIVE_KIND.into()],
            emissions: vec![TRACE_DERIVED_KIND.into()],
            priority: 100,
        }
    }

    async fn handle(
        &self,
        _event: &EventEnvelope,
    ) -> oclive_kernel_types::Result<EventModuleOutput> {
        Ok(EventModuleOutput {
            emitted: vec![EventDraft {
                kind: TRACE_DERIVED_KIND.into(),
                payload: serde_json::json!({"synthetic_private_body": "must-not-be-traced"}),
                metadata: [("synthetic.secret".into(), serde_json::json!(true))]
                    .into_iter()
                    .collect(),
            }],
            ..Default::default()
        })
    }
}

struct SampleProactiveSource {
    module_id: &'static str,
}

#[async_trait]
impl EventModule for SampleProactiveSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: self.module_id.into(),
            emissions: vec![PROACTIVE_TURN_PROPOSED_EVENT_KIND.into()],
            ..Default::default()
        }
    }

    async fn handle(
        &self,
        _event: &EventEnvelope,
    ) -> oclive_kernel_types::Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
}

struct SampleEmitters {
    trace: Arc<dyn EventEmitter>,
    proactive_high: Arc<dyn EventEmitter>,
    proactive_low: Arc<dyn EventEmitter>,
}

pub(crate) async fn execute_phase(
    kernel: &OcliveKernel,
    contract: &SampleContract,
    phase: &str,
) -> anyhow::Result<()> {
    let emitters = register_sample_modules(kernel)?;
    for scenario in contract
        .scenarios
        .iter()
        .filter(|scenario| scenario.phase == phase)
    {
        match scenario.action.as_str() {
            "single_root" => {
                emit_trace(&emitters.trace, scenario.id.as_str(), TRACE_SINGLE_KIND).await?;
            }
            "derived_chain" => {
                emit_trace(&emitters.trace, scenario.id.as_str(), TRACE_DERIVE_KIND).await?;
            }
            "proactive_admitted" => {
                let permit = propose_sample(
                    kernel,
                    emitters.proactive_high.as_ref(),
                    scenario.id.as_str(),
                    9_000,
                    8_000,
                )
                .await?;
                if permit.is_none() {
                    bail!("{} was expected to be admitted", scenario.id);
                }
            }
            "proactive_rejected" => {
                let permit = propose_sample(
                    kernel,
                    emitters.proactive_low.as_ref(),
                    scenario.id.as_str(),
                    1_000,
                    1_000,
                )
                .await?;
                if permit.is_some() {
                    bail!("{} was expected to be rejected", scenario.id);
                }
            }
            "single_root_after_restart" => {
                emit_trace(
                    &emitters.trace,
                    scenario.id.as_str(),
                    TRACE_AFTER_RESTART_KIND,
                )
                .await?;
            }
            action => bail!("unsupported sample action {action}"),
        }
    }
    Ok(())
}

fn register_sample_modules(kernel: &OcliveKernel) -> anyhow::Result<SampleEmitters> {
    kernel
        .register_event_module_with_policy(
            Arc::new(SampleTraceDeriver),
            EventModuleRegistryPolicy {
                influence_weight_bps: 8_200,
                ..Default::default()
            },
        )
        .map_err(anyhow::Error::msg)?;
    let trace = kernel
        .register_event_module_with_policy(
            Arc::new(SampleTraceSource),
            EventModuleRegistryPolicy {
                influence_weight_bps: 6_400,
                ..Default::default()
            },
        )
        .map_err(anyhow::Error::msg)?;
    let proactive_high = kernel
        .register_event_module_with_policy(
            Arc::new(SampleProactiveSource {
                module_id: "sample.proactive_high",
            }),
            EventModuleRegistryPolicy {
                influence_weight_bps: 9_000,
                ..Default::default()
            },
        )
        .map_err(anyhow::Error::msg)?;
    let proactive_low = kernel
        .register_event_module_with_policy(
            Arc::new(SampleProactiveSource {
                module_id: "sample.proactive_low",
            }),
            EventModuleRegistryPolicy {
                influence_weight_bps: 1_000,
                ..Default::default()
            },
        )
        .map_err(anyhow::Error::msg)?;
    Ok(SampleEmitters {
        trace,
        proactive_high,
        proactive_low,
    })
}

async fn emit_trace(
    emitter: &Arc<dyn EventEmitter>,
    correlation_id: &str,
    kind: &str,
) -> anyhow::Result<()> {
    emitter
        .emit(
            "synthetic:runtime-event-trace-sample",
            Some(correlation_id),
            EventDraft {
                kind: kind.into(),
                payload: serde_json::json!({
                    "synthetic_private_body": "must-not-be-traced",
                }),
                metadata: [("synthetic.secret".into(), serde_json::json!(true))]
                    .into_iter()
                    .collect(),
            },
        )
        .await
        .map(|_| ())
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

async fn propose_sample(
    kernel: &OcliveKernel,
    emitter: &dyn EventEmitter,
    correlation_id: &str,
    confidence_bps: u16,
    urgency_bps: u16,
) -> anyhow::Result<Option<ProactiveTurnPermit>> {
    kernel
        .propose_proactive_turn(
            emitter,
            "synthetic:runtime-event-trace-sample",
            correlation_id,
            ProactiveTurnProposal {
                role_id: "synthetic-role".into(),
                session_id: Some("synthetic-session".into()),
                scene_id: Some("synthetic-scene".into()),
                origin: TurnOrigin::Sensor,
                signal_kind: ProactiveSignalKind::SensorObservation,
                observation: "synthetic_signal=present".into(),
                confidence_bps,
                urgency_bps,
            },
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}
