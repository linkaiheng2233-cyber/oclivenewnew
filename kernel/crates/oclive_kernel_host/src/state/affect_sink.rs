//! Optional host→UI callback when affect display metrics change (Tauri `emit_all` in desktop setup).

use crate::models::dto::DisplayMetricsDto;
use parking_lot::RwLock;
use std::sync::Arc;

/// Payload for [`AffectMetricsSink`] (role namespace + UI-only metrics).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AffectSnapshotEvent {
    pub role_id: String,
    pub metrics: DisplayMetricsDto,
}

pub type AffectMetricsSink = Arc<dyn Fn(AffectSnapshotEvent) + Send + Sync>;

/// Cloneable handle to invoke the optional sink without holding full [`super::AppState`].
#[derive(Clone, Default)]
pub struct AffectSinkHandle {
    sink: Arc<RwLock<Option<AffectMetricsSink>>>,
}

impl AffectSinkHandle {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sink: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set(&self, sink: Option<AffectMetricsSink>) {
        *self.sink.write() = sink;
    }

    pub fn emit(&self, event: AffectSnapshotEvent) {
        if let Some(cb) = self.sink.read().as_ref() {
            cb(event);
        }
    }
}
