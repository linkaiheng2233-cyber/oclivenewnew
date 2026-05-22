//! `oclive test` 人类可读报告（颜色 / 通过率 / 耗时）。

use crate::test_cmd::CheckResult;
use std::io::{IsTerminal, stdout};
use std::time::Duration;

pub fn print_human_report(root: &std::path::Path, checks: &[CheckResult], elapsed: Duration) {
    let use_color = stdout().is_terminal();
    let green = if use_color { "\x1b[32m" } else { "" };
    let red = if use_color { "\x1b[31m" } else { "" };
    let dim = if use_color { "\x1b[2m" } else { "" };
    let bold = if use_color { "\x1b[1m" } else { "" };
    let reset = if use_color { "\x1b[0m" } else { "" };

    let total = checks.len();
    let passed = checks.iter().filter(|c| c.ok).count();
    let failed = total.saturating_sub(passed);
    let pct = if total == 0 {
        100.0
    } else {
        (passed as f64) * 100.0 / (total as f64)
    };

    println!("{bold}oclive test{reset} — {}", root.display());
    println!(
        "{dim}summary:{reset} {passed}/{total} passed ({pct:.0}%) · {failed} failed · {:.2}s",
        elapsed.as_secs_f64()
    );
    println!();

    for c in checks {
        let (icon, status_color) = if c.ok {
            ("✅", green)
        } else {
            ("❌", red)
        };
        let dur = c
            .duration_ms
            .map(|ms| format!(" {dim}{ms}ms{reset}"))
            .unwrap_or_default();
        println!(
            "  {icon} {status_color}{}{reset} — {}{dur}",
            c.name, c.detail
        );
    }

    println!();
    if failed == 0 {
        println!("{green}{bold}test result: ok{reset}. {passed} passed; 0 failed");
    } else {
        println!(
            "{red}{bold}test result: FAILED{reset}. {passed} passed; {failed} failed"
        );
    }
}
