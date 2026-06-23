//! Virtual clock based on a real-world anchor plus a flow-rate ratio.

/// Align to the minute (in milliseconds).
#[must_use]
pub fn round_to_minute_ms(ts_ms: i64) -> i64 {
    const M: i64 = 60_000;
    (ts_ms / M) * M
}

/// Compute the current virtual time from the anchor and flow rate.
///
/// When `anchor_real_ms` / `anchor_virtual_ms` are 0 it means uninitialized, and the aligned `real_now_ms` is returned.
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

/// Virtual hours between two timestamps (based on the millisecond difference).
#[must_use]
pub fn virtual_hours_between_ms(from_ms: i64, to_ms: i64) -> f64 {
    if to_ms <= from_ms {
        return 0.0;
    }
    (to_ms - from_ms) as f64 / 3_600_000.0
}

/// Virtual days between two timestamps (based on the millisecond difference).
#[must_use]
pub fn virtual_days_between_ms(from_ms: i64, to_ms: i64) -> f64 {
    if to_ms <= from_ms {
        return 0.0;
    }
    (to_ms - from_ms) as f64 / 86_400_000.0
}

/// Convert elapsed real time into virtual days (`real_elapsed_ms * ratio / one day`).
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
    fn virtual_hours_between_timestamps() {
        assert!((virtual_hours_between_ms(0, 3_600_000) - 1.0).abs() < 1e-6);
        assert_eq!(virtual_hours_between_ms(100, 50), 0.0);
    }

    #[test]
    fn ratio_five_advances_virtual_five_times_faster() {
        let anchor_real = round_to_minute_ms(1_700_000_000_000);
        let anchor_virtual = anchor_real;
        let real_after_12min = anchor_real + 12 * 60_000;
        let v = compute_virtual_now_ms(anchor_real, anchor_virtual, real_after_12min, 5.0);
        assert_eq!(v - anchor_virtual, 60 * 60_000);
    }
}
