//! 现实锚点 + 流速比的虚拟时钟。

/// 对齐到分钟（毫秒）。
#[must_use]
pub fn round_to_minute_ms(ts_ms: i64) -> i64 {
    const M: i64 = 60_000;
    (ts_ms / M) * M
}

/// 由锚点与流速计算当前虚拟时间。
///
/// `anchor_real_ms` / `anchor_virtual_ms` 为 0 时表示未初始化，返回对齐后的 `real_now_ms`。
#[must_use]
pub fn compute_virtual_now_ms(
    anchor_real_ms: i64,
    anchor_virtual_ms: i64,
    real_now_ms: i64,
    real_to_virtual_ratio: f64,
) -> i64 {
    let ratio = if real_to_virtual_ratio.is_finite() && real_to_virtual_ratio > 0.0 {
        real_to_virtual_ratio
    } else {
        1.0
    };
    if anchor_real_ms <= 0 {
        return round_to_minute_ms(real_now_ms);
    }
    let real_elapsed = real_now_ms.saturating_sub(anchor_real_ms);
    let virtual_elapsed = (real_elapsed as f64 * ratio).round() as i64;
    round_to_minute_ms(anchor_virtual_ms.saturating_add(virtual_elapsed))
}

/// 两时间戳之间的虚拟日数（基于毫秒差）。
#[must_use]
pub fn virtual_days_between_ms(from_ms: i64, to_ms: i64) -> f64 {
    if to_ms <= from_ms {
        return 0.0;
    }
    (to_ms - from_ms) as f64 / 86_400_000.0
}

/// 现实经过时间折算为虚拟日数（`real_elapsed_ms * ratio / 一天`）。
#[must_use]
pub fn virtual_days_from_real_elapsed_ms(real_elapsed_ms: i64, real_to_virtual_ratio: f64) -> f64 {
    if real_elapsed_ms <= 0 {
        return 0.0;
    }
    let ratio = if real_to_virtual_ratio.is_finite() && real_to_virtual_ratio > 0.0 {
        real_to_virtual_ratio
    } else {
        1.0
    };
    (real_elapsed_ms as f64 * ratio) / 86_400_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_five_advances_virtual_five_times_faster() {
        let anchor_real = round_to_minute_ms(1_700_000_000_000);
        let anchor_virtual = anchor_real;
        let real_after_12min = anchor_real + 12 * 60_000;
        let v = compute_virtual_now_ms(anchor_real, anchor_virtual, real_after_12min, 5.0);
        assert_eq!(v - anchor_virtual, 60 * 60_000);
    }
}
