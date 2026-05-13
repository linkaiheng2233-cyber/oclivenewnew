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

impl EmotionResult {
    /// 强中性七维分布（与 stub / 禁用分析器 / `UserEmotionAnalyzer` V2 占位一致）。
    /// 纯 `f64` 字面量，无 FFI；可在 Miri 下覆盖相关单元测试（与 `http_api` 中 `libc::statvfs` 等主机调用不同）。
    #[must_use]
    pub const fn strong_neutral() -> Self {
        Self {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EmotionResult;

    #[test]
    fn strong_neutral_is_normalized() {
        let r = EmotionResult::strong_neutral();
        assert_eq!(r.neutral, 1.0);
        assert_eq!(r.joy + r.sadness + r.anger + r.fear + r.surprise + r.disgust, 0.0);
    }
}
