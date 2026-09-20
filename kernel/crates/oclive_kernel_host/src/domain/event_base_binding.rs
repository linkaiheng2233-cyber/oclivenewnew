//! CP-B3-ALL unit E: the Host-private [`EventBase`] request-binding view over the real event impact
//! execution.
//!
//! [`HostEventBaseView`] borrows the Host's **real assembled** turn inputs (the LLM client, the
//! model of this turn, the user emotion, the personality vector, the recent turns, the recent event
//! chain and the knowledge augment) and runs
//! [`estimate_event_impact_core`](super::event_impact_ai::estimate_event_impact_core) — the same one
//! execution the product entry
//! ([`BuiltinEventEstimator`](super::event_estimator::BuiltinEventEstimator)) runs. This view reads
//! only the **narrative** projection of that response; the product keeps reading the numeric one.
//!
//! The Base request carries two things: `material` and an optional `context`.
//!
//! - `material` is this call's user material, passed to the shared execution **verbatim** (the
//!   shared core is what the product already does with a user message; this view adds no rewriting,
//!   no trimming and no classification of the text).
//! - a non-empty `context` enters that **same** prompt in its own labelled section (see
//!   `build_event_impact_prompt`), so it is never silently ignored or merely logged. The legacy
//!   product call passes no context and none is invented for it.
//!
//! The emotion, personality, history and knowledge inputs are **not** Base request fields: they come
//! from the Host's real assembly, exactly as the legacy call received them. This view does not
//! construct a placeholder emotion, a fabricated personality or a synthetic `Role` to fill a gap.
//!
//! # What a result claims
//!
//! `Ok(Some(text))` is the model's narrative for this material, verbatim. `Ok(None)` means the model
//! explicitly answered `null`: it is not "no event happened", not "impact is exactly zero", and not
//! an inference from the numeric projection. The failure cases and their reasons are documented on
//! [`EventImpactCore`](super::event_impact_ai::EventImpactCore):
//!
//! - the response carried no usable `analysis` member → `Failed` (a missing commitment, never `None`);
//! - the response could not be used under the existing convention → `Failed` (the rule fallback is
//!   not a successful narrative analysis);
//! - the model call returned `Err` → the known typed reason, never a classification read from text;
//! - the current gating forbids the model analysis → `Unsupported`, with **zero** generations.
//!
//! # What this does not do
//!
//! - It does not add a second generation, a retry, a second decode or a second fallback: one Base
//!   call is one real execution of the shared core.
//! - It does not inject the narrative into the main prompt, write it back into domain state,
//!   persist it, or add a DTO/wire field.
//! - It does not run, alter or replace the B2 `LlmEventAnalyzer`
//!   (`oclive_kernel_runtime::domain::base_event`) or its private protocol; those stay as they are.
//! - **No product consumer reads this view today.** The reference Host binds
//!   `Arc<dyn EventEstimator>` and reads the numeric estimate; serving the Base view is an additive
//!   capability of the implementation, not a product migration.
//! - Whether a real model produces an honest narrative, and whether that narrative correctly
//!   distinguishes negation, condition, plan, quotation and subject, is **not** verified here: this
//!   unit verifies the mechanism (member, decode, branch, projection, prompt input), not the model's
//!   semantic quality.

use oclive_kernel_contracts::{BaseCallFuture, EventBase};
use oclive_kernel_types::EventBaseRequest;

use super::event_impact_ai::{estimate_event_impact_core, project_error};
use crate::domain::ports::LlmClient;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, PersonalityVector};
use std::sync::Arc;

/// The Host's real assembled turn inputs for one event analysis.
///
/// Every field is borrowed from values the Host already has for this turn; none of them is a Base
/// request field, and none is defaulted or fabricated by this module.
#[derive(Clone, Copy)]
pub(crate) struct EventTurnMaterial<'a> {
    /// The user emotion the product passes today.
    pub(crate) user_emotion: &'a Emotion,
    /// The personality vector the product passes today.
    pub(crate) personality: &'a PersonalityVector,
    /// The recent dialogue turns the product passes today (old → new).
    pub(crate) recent_turns: &'a [(String, String)],
    /// The recent event chain the product passes today (new → old).
    pub(crate) recent_events: &'a [Event],
    /// The knowledge augment the product passes today, if any.
    pub(crate) knowledge_augment: Option<&'a KnowledgeEventAugment>,
}

/// The Host-private [`EventBase`] view over the real event impact execution.
///
/// See the [module documentation](self) for the material/context agreement, what a result claims and
/// what this view deliberately does not do.
pub(crate) struct HostEventBaseView<'a> {
    llm: &'a Arc<dyn LlmClient>,
    model: &'a str,
    material: EventTurnMaterial<'a>,
    use_llm: bool,
}

impl<'a> HostEventBaseView<'a> {
    /// Binds the Base view to the Host's real assembled inputs for this turn.
    #[must_use]
    pub(crate) fn new(
        llm: &'a Arc<dyn LlmClient>,
        model: &'a str,
        material: EventTurnMaterial<'a>,
        use_llm: bool,
    ) -> Self {
        Self {
            llm,
            model,
            material,
            use_llm,
        }
    }
}

impl EventBase for HostEventBaseView<'_> {
    /// Analyses the event described by `request.material` and returns the narrative projection.
    ///
    /// # Errors
    ///
    /// Returns the narrative projection's failure as documented on this module, plus the typed
    /// projection of a rule-detection error of the shared execution. This view adds no failure
    /// source of its own.
    fn analyze<'a>(&'a self, request: EventBaseRequest<'a>) -> BaseCallFuture<'a, Option<String>> {
        // Copy the borrowed real inputs out of `self`: they are this call's assembled material, and
        // the future borrows only the request for the duration of the call.
        let llm = self.llm;
        let model = self.model;
        let material = self.material;
        let use_llm = self.use_llm;
        Box::pin(async move {
            match estimate_event_impact_core(
                llm,
                model,
                request.material,
                material.user_emotion,
                material.personality,
                material.recent_turns,
                material.recent_events,
                material.knowledge_augment,
                use_llm,
                request.context,
            )
            .await
            {
                Ok(core) => core.analysis,
                Err(error) => Err(project_error(&error)),
            }
        })
    }
}
