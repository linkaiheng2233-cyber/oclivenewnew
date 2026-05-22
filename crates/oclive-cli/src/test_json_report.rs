//! Machine-readable report for `oclive test --json` (see `schemas/oclive_test_report.schema.json`).

use crate::test_cmd::CheckResult;
use serde::Serialize;

pub const TEST_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub struct TestJsonReport {
    pub schema_version: u32,
    pub summary: TestSummary,
    pub suites: Vec<TestSuite>,
    pub failures: Vec<TestFailure>,
}

#[derive(Debug, Serialize)]
pub struct TestSummary {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Serialize)]
pub struct TestSuite {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestFailure {
    pub suite: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    pub error: String,
}

pub fn build_report(checks: &[CheckResult]) -> TestJsonReport {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut suites = Vec::with_capacity(checks.len());
    let mut failures = Vec::new();

    for c in checks {
        let status = if c.ok {
            if c.detail.contains("skipped") {
                skipped += 1;
                "skipped"
            } else {
                passed += 1;
                "passed"
            }
        } else {
            failed += 1;
            failures.push(TestFailure {
                suite: c.name.clone(),
                file: None,
                line: None,
                error: c.detail.clone(),
            });
            "failed"
        };
        suites.push(TestSuite {
            name: c.name.clone(),
            status: status.to_string(),
            duration_ms: c.duration_ms,
            detail: if c.ok { None } else { Some(c.detail.clone()) },
        });
    }

    TestJsonReport {
        schema_version: TEST_REPORT_SCHEMA_VERSION,
        summary: TestSummary {
            passed,
            failed,
            skipped,
        },
        suites,
        failures,
    }
}

pub fn print_json(checks: &[CheckResult]) -> anyhow::Result<bool> {
    let report = build_report(checks);
    let ok = report.summary.failed == 0;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(ok)
}
