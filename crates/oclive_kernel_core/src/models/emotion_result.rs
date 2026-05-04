use serde::{Deserialize, Serialize};

/// 情绪分析结果（七维），与 `EmotionAnalyzer` 输出一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionResult {
    pub joy: f64,
    pub sadness: f64,
    pub anger: f64,
    pub fear: f64,
    pub surprise: f64,
    pub disgust: f64,
    pub neutral: f64,
}
