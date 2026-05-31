//! `oclive lint` human-readable report (color / pass rate / elapsed time).

use crate::commands::lint::LintItem;
use std::io::{IsTerminal, stdout};
use std::time::{Duration, Instant};

pub struct LintCheck {
    pub item: LintItem,
    pub duration: Duration,
}

pub fn print_human_report(root: &std::path::Path, checks: &[LintCheck], elapsed: Duration) {
    let use_color = stdout().is_terminal();
    let green = if use_color { "\x1b[32m" } else { "" };
    let red = if use_color { "\x1b[31m" } else { "" };
    let yellow = if use_color { "\x1b[33m" } else { "" };
    let dim = if use_color { "\x1b[2m" } else { "" };
    let bold = if use_color { "\x1b[1m" } else { "" };
    let reset = if use_color { "\x1b[0m" } else { "" };

    let total = checks.len();
    let passed = checks
        .iter()
        .filter(|c| c.item.level == "pass")
        .count();
    let failed = checks
        .iter()
        .filter(|c| c.item.level == "fail")
        .count();
    let pct = if total == 0 {
        100.0
    } else {
        (passed as f64) * 100.0 / (total as f64)
    };

    println!("{bold}oclive lint{reset} — {}", root.display());
    println!(
        "{dim}summary:{reset} {passed}/{total} passed ({pct:.0}%) · {failed} failed · {:.2}s",
        elapsed.as_secs_f64()
    );
    println!();

    for c in checks {
        let (icon, status_color) = match c.item.level.as_str() {
            "pass" => ("✅", green),
            "warn" => ("⚠️", yellow),
            _ => ("❌", red),
        };
        let ms = c.duration.as_millis();
        println!(
            "  {icon} {status_color}[{}]{reset} {}{dim} {ms}ms{reset}",
            c.item.check, c.item.message
        );
        if let Some(ref fix) = c.item.fix {
            println!("      → {fix}");
        }
    }

    println!();
    if failed == 0 {
        println!("{green}{bold}lint result: ok{reset}. {passed} passed. 0 failed.");
    } else {
        println!(
            "{red}{bold}lint result: FAILED{reset}. {passed} passed. {failed} failed."
        );
    }
}

pub fn timed<F>(mut f: F) -> (LintItem, Duration)
where
    F: FnMut() -> LintItem,
{
    let start = Instant::now();
    let item = f();
    (item, start.elapsed())
}
