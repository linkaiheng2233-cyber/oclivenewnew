//! Explicit Event Base binding over the Host's already composed generation path.
//! Ordinary chat never schedules this analysis merely by exposing the view.

use oclive_kernel_contracts::{BaseCallFuture, EventBase};
use oclive_kernel_runtime::domain::base_event::LlmEventAnalyzer;
use oclive_kernel_types::EventBaseRequest;

use crate::state::AppState;

pub(crate) struct HostEventAnalysisBase<'a> {
    pub(crate) state: &'a AppState,
}

impl EventBase for HostEventAnalysisBase<'_> {
    fn analyze<'a>(&'a self, request: EventBaseRequest<'a>) -> BaseCallFuture<'a, Option<String>> {
        Box::pin(async move {
            let generator = super::minimal_llm::HostTextGenerationBase { state: self.state };
            let analyzer = LlmEventAnalyzer::new(&generator);
            analyzer.analyze(request).await
        })
    }
}
