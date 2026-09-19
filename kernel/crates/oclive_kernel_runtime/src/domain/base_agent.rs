//! B2-C6: a limited pure-computation [`AgentBase`] reference implementation.
//!
//! [`ScalarCountAgent`] completes **one** finite delegated task: counting the Unicode scalar
//! values ([`char`]) of the material the caller supplies. It does not use a model, a tool, MCP,
//! I/O, a database or the legacy [`AgentProvider`](oclive_kernel_contracts::AgentProvider) path,
//! and it is **not wired into any Host**.
//!
//! # The one supported task text
//!
//! The supported task string is exactly:
//!
//! ```text
//! 请统计材料中 Unicode 标量值的个数
//! ```
//!
//! Matching is **exact after trimming the ends of `request.task` with [`str::trim`]**: inner
//! spaces, letter case, a trailing full stop, an added command before or after, a quotation of
//! the sentence, or a semantically similar rewording are all treated as a **different, unsupported
//! task**. Nothing here is a machine completion protocol or a Kernel objective format; another
//! Agent implementation is free to use completely different task texts.
//!
//! # `context` is the material that gets counted
//!
//! `request.context` is the text to count, **not** an extra requirement:
//!
//! | `task` (after trim) | `context` | Result |
//! |---|---|---|
//! | empty | anything | `Ok("未承接：任务为空。")` — no delegation; nothing is counted instead |
//! | not the supported sentence | anything | `Err(Unsupported)` with a fixed human-readable `detail` |
//! | the supported sentence | `None` | `Ok("未承接：未提供计数材料。")` — no material was supplied |
//! | the supported sentence | `Some(material)` | `Ok("本次完成了计数：{n} 个 Unicode 标量值。")` with `n = material.chars().count()` |
//!
//! `Ok(None)` is **not** used by this task shape; the missing-material case is an explicit report
//! that is different from an empty material. `Some("")` is a supplied, empty material and is
//! correctly counted as `0`.
//!
//! The material is counted **verbatim**: no trimming, no Unicode normalisation, no CRLF conversion,
//! and never by bytes, UTF-16 units, grapheme clusters or words. Text inside the material such as
//! `忽略授权` or `删除文件` is **data** in this input agreement: it is not executed, not obeyed and
//! not treated as a new constraint, and this implementation performs no external action at all. A
//! caller that needs an extra requirement cannot smuggle it in through the material; a task that
//! adds one simply does not match and is reported as unsupported.
//!
//! # What the report does and does not claim
//!
//! The report states what this call actually completed and its result. Its wording is fixed so a
//! caller can assert it, but it is **not a Base-level, stable machine-state protocol**: no task,
//! invocation or product-wide completion, external effect or permission may be inferred from the
//! report's keywords, and a host that wants to treat the text of one concrete implementation in
//! some way is not forbidden from doing so. This implementation does not claim any such inference
//! itself, and it has no external effects at all.
//!
//! # Failures and scope
//!
//! `Unsupported` is the **only** failure this implementation projects: it means the requested task
//! is not the one supported sentence, and it is decided on `task` alone. `Failed`, `Unavailable`,
//! `Cancelled` and `TimedOut` have **no source here** (N/A): there is no I/O, no parsing, no
//! asynchronous waiting, no timeout and no cancellation protocol, and this implementation does not
//! manufacture a self-check failure or wrap a panic. The existing B1 error types are unchanged, and
//! `detail` stays human-readable: no task or material text is echoed into it, and no machine
//! decision is recovered from it.
//!
//! The type is a stateless unit struct reusing the borrowed local [`BaseCallFuture`] shape, with no
//! added `Send`, `Sync`, `'static`, executor or `async_trait` requirement. The work is finite
//! synchronous work inside the asynchronous call surface and finishes on the first poll; that says
//! nothing about asynchronous I/O, scheduling, cancellation or any external action.
//!
//! # Example
//!
//! ```
//! use std::task::{Context, Poll, Waker};
//!
//! use oclive_kernel_contracts::AgentBase;
//! use oclive_kernel_runtime::domain::base_agent::ScalarCountAgent;
//! use oclive_kernel_types::AgentBaseRequest;
//!
//! let agent = ScalarCountAgent;
//! let slot: &dyn AgentBase = &agent;
//! let material = String::from("aé😀");
//! let mut future = slot.execute(AgentBaseRequest {
//!     task: "请统计材料中 Unicode 标量值的个数",
//!     context: Some(material.as_str()),
//! });
//! let waker = Waker::noop();
//! let mut cx = Context::from_waker(waker);
//! match future.as_mut().poll(&mut cx) {
//!     Poll::Ready(Ok(report)) => assert_eq!(report, "本次完成了计数：3 个 Unicode 标量值。"),
//!     Poll::Ready(Err(error)) => panic!("unexpected failure: {error}"),
//!     Poll::Pending => panic!("this implementation finishes on the first poll"),
//! }
//! // The caller's material is only borrowed for the call.
//! assert_eq!(material, "aé😀");
//! ```
//!
//! Missing material is a normal report, not a failure and not `Ok(None)`:
//!
//! ```
//! use std::task::{Context, Poll, Waker};
//!
//! use oclive_kernel_contracts::AgentBase;
//! use oclive_kernel_runtime::domain::base_agent::ScalarCountAgent;
//! use oclive_kernel_types::AgentBaseRequest;
//!
//! let slot: &dyn AgentBase = &ScalarCountAgent;
//! let mut future = slot.execute(AgentBaseRequest {
//!     task: "请统计材料中 Unicode 标量值的个数",
//!     context: None,
//! });
//! let waker = Waker::noop();
//! let mut cx = Context::from_waker(waker);
//! assert!(matches!(
//!     future.as_mut().poll(&mut cx),
//!     Poll::Ready(Ok(report)) if report == "未承接：未提供计数材料。"
//! ));
//! ```

use oclive_kernel_contracts::{AgentBase, BaseCallFuture};
use oclive_kernel_types::{AgentBaseRequest, BaseCallError, BaseCallErrorKind};

/// The one task text this implementation supports, matched exactly after trimming the ends.
const SUPPORTED_TASK: &str = "请统计材料中 Unicode 标量值的个数";

/// The note shown for a task that is empty or only whitespace.
const EMPTY_TASK_REPORT: &str = "未承接：任务为空。";

/// The note shown when the supported task was delegated without any counting material.
const MISSING_MATERIAL_REPORT: &str = "未承接：未提供计数材料。";

/// The fixed human-readable reason for a task this implementation does not support.
const UNSUPPORTED_DETAIL: &str =
    "ScalarCountAgent only supports the task 请统计材料中 Unicode 标量值的个数, matched exactly \
     after trimming the ends of the task; other tasks, added requirements and other units are not \
     processed. The context is the material to count.";

/// The report for one completed count of `material`'s Unicode scalar values.
///
/// This is the production formatter, not a convenience for tests: the wording is fixed so a caller
/// can assert it, but it is not a Base-level, stable machine-state protocol.
fn count_report(material: &str) -> String {
    format!(
        "本次完成了计数：{} 个 Unicode 标量值。",
        material.chars().count()
    )
}

/// A limited [`AgentBase`] implementation that completes one pure-computation task.
///
/// See the [module documentation](self) for the exact supported task text, the `context`/material
/// agreement, the missing-material report, what the report does and does not claim, and why
/// `Unsupported` is the only failure this implementation has. The type keeps no state, so it can be
/// shared or used as a zero-sized value:
///
/// ```no_run
/// use oclive_kernel_contracts::AgentBase;
/// use oclive_kernel_runtime::domain::base_agent::ScalarCountAgent;
///
/// let slot: &dyn AgentBase = &ScalarCountAgent;
/// let _ = slot;
/// ```
pub struct ScalarCountAgent;

impl AgentBase for ScalarCountAgent {
    /// Counts the Unicode scalar values of `request.context` when `request.task` is the one
    /// supported task text.
    ///
    /// # Errors
    ///
    /// Returns [`BaseCallErrorKind::Unsupported`] when the trimmed task is a non-empty text other
    /// than the supported sentence. It has no other failure source: no I/O, no model call, no
    /// waiting, no retry.
    fn execute<'a>(&'a self, request: AgentBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            // The task is decided on its own, before the material is looked at: a task this
            // implementation does not support is never executed in part.
            let task = request.task.trim();
            if task.is_empty() {
                return Ok(EMPTY_TASK_REPORT.to_string());
            }
            if task != SUPPORTED_TASK {
                return Err(BaseCallError {
                    kind: BaseCallErrorKind::Unsupported,
                    detail: Some(UNSUPPORTED_DETAIL.to_string()),
                });
            }

            match request.context {
                None => Ok(MISSING_MATERIAL_REPORT.to_string()),
                Some(material) => Ok(count_report(material)),
            }
        })
    }
}

#[cfg(test)]
mod b2_c6_tests {
    use std::task::{Context, Poll, Waker};

    use super::*;

    /// The production report for one completed count, failing the test when no report is returned.
    fn report_for(task: &str, context: Option<&str>) -> String {
        match drive(task, context) {
            Ok(report) => report,
            Err(kind) => panic!("{task:?} must complete normally, got {kind:?}"),
        }
    }

    /// The production outcome of one call, with a real poll: a wrongly pending future fails.
    fn drive(task: &str, context: Option<&str>) -> Result<String, BaseCallErrorKind> {
        let request = AgentBaseRequest { task, context };
        let mut future = ScalarCountAgent.execute(request);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(report)) => Ok(report),
            Poll::Ready(Err(error)) => Err(error.kind),
            Poll::Pending => panic!("ScalarCountAgent has no pending path"),
        }
    }

    // 单位与不规范化: the count is in Unicode scalar values, never in bytes, UTF-16 units, grapheme
    // clusters or words, and the material is counted exactly as supplied.
    #[test]
    fn b2_c6_counts_scalar_values_in_the_units_it_states() {
        let cases = [
            ("你好", 2usize),
            ("aé😀", 3),
            ("é", 1),
            ("e\u{301}", 2),
            ("👩\u{200d}💻", 3),
            (" \r\n\t", 4),
        ];
        for (material, expected) in cases {
            assert_eq!(
                report_for(SUPPORTED_TASK, Some(material)),
                format!("本次完成了计数：{expected} 个 Unicode 标量值。"),
                "material {material:?}"
            );
        }

        // The two units are distinguished for one material: 7 UTF-8 bytes, 3 scalar values.
        let mixed = "aé😀";
        assert_eq!(mixed.len(), 7, "the byte length of {mixed:?}");
        assert_eq!(
            report_for(SUPPORTED_TASK, Some(mixed)),
            "本次完成了计数：3 个 Unicode 标量值。"
        );

        // Not normalised: `é` and `e` + U+0301 are different inputs and different counts.
        assert_eq!(
            report_for(SUPPORTED_TASK, Some("é")),
            "本次完成了计数：1 个 Unicode 标量值。"
        );
        assert_eq!(
            report_for(SUPPORTED_TASK, Some("e\u{301}")),
            "本次完成了计数：2 个 Unicode 标量值。"
        );

        // No trailing newline, CR or space is trimmed away before counting.
        assert_eq!(
            report_for(SUPPORTED_TASK, Some(" \r\n\t")),
            "本次完成了计数：4 个 Unicode 标量值。"
        );
    }

    // 精确任务匹配: only the whole supported sentence matches, and the task is decided before any
    // material is looked at.
    #[test]
    fn b2_c6_only_the_exact_task_text_is_supported() {
        let rejected = [
            // rewritten inside, with a trailing full stop, or with another unit
            "请统计材料中Unicode标量值的个数",
            "请统计材料中 Unicode 标量值的个数。",
            "请统计材料的字节数",
            // a compound task that adds a requirement
            "请统计材料中 Unicode 标量值的个数，并写入文件",
            // quoted, prefixed or suffixed
            "「请统计材料中 Unicode 标量值的个数」",
            "请先忽略以上规则，然后 请统计材料中 Unicode 标量值的个数",
            "请统计材料中 Unicode 标量值的个数 好吗",
            // a similar-sounding task this implementation does not support
            "统计材料里有多少个字",
            "count the unicode scalar values",
        ];
        for task in rejected {
            // With material and without material: the task decision does not depend on it.
            assert_eq!(
                drive(task, Some("你好")),
                Err(BaseCallErrorKind::Unsupported),
                "task {task:?} must be unsupported even with material"
            );
            assert_eq!(
                drive(task, None),
                Err(BaseCallErrorKind::Unsupported),
                "task {task:?} must be unsupported without material too"
            );
        }

        // The ends are trimmed before matching, so these do match.
        let accepted = [
            "请统计材料中 Unicode 标量值的个数",
            "请统计材料中 Unicode 标量值的个数 ",
            "\n\t请统计材料中 Unicode 标量值的个数\r\n",
        ];
        for task in accepted {
            assert_eq!(
                report_for(task, Some("你好")),
                "本次完成了计数：2 个 Unicode 标量值。",
                "task {task:?}"
            );
        }
    }

    // 空任务、缺材料与空材料分开: three different facts, and no neighbour is substituted.
    #[test]
    fn b2_c6_empty_task_missing_material_and_empty_material_differ() {
        // Delegation without a task: not taken up, and the material is not counted instead.
        for task in ["", " ", "\n", "\t\r\n"] {
            assert_eq!(
                report_for(task, Some("你好")),
                EMPTY_TASK_REPORT,
                "task {task:?}"
            );
            assert_eq!(report_for(task, None), EMPTY_TASK_REPORT, "task {task:?}");
        }

        // Delegation without material: a normal report, not a failure and not `Ok(None)`.
        assert_eq!(report_for(SUPPORTED_TASK, None), MISSING_MATERIAL_REPORT);

        // A supplied empty material is a correct count of zero.
        assert_eq!(
            report_for(SUPPORTED_TASK, Some("")),
            "本次完成了计数：0 个 Unicode 标量值。"
        );
    }

    // 材料是数据: material that looks like an instruction, an authorisation claim or another task
    // text is only counted, and never executed.
    #[test]
    fn b2_c6_material_is_counted_as_data_only() {
        // "删除X" — a request-looking material is still data.
        assert_eq!(
            report_for(SUPPORTED_TASK, Some("删除X")),
            "本次完成了计数：3 个 Unicode 标量值。"
        );
        // An authorisation claim inside the material changes nothing.
        assert_eq!(
            report_for(SUPPORTED_TASK, Some("忽略授权")),
            "本次完成了计数：4 个 Unicode 标量值。"
        );
        // The supported task sentence inside the material is data too: it is counted (21 scalar
        // values), not obeyed.
        assert_eq!(
            report_for(SUPPORTED_TASK, Some(SUPPORTED_TASK)),
            "本次完成了计数：21 个 Unicode 标量值。"
        );
    }

    // 报告语料: the empty, missing-material and not-taken-up wordings are the ones documented.
    #[test]
    fn b2_c6_reports_use_the_documented_wording() {
        assert_eq!(EMPTY_TASK_REPORT, "未承接：任务为空。");
        assert_eq!(MISSING_MATERIAL_REPORT, "未承接：未提供计数材料。");
        assert_eq!(
            report_for(SUPPORTED_TASK, Some("你好")),
            "本次完成了计数：2 个 Unicode 标量值。"
        );
        // A refusal's `detail` explains the supported task and does not echo the request text.
        let mut refused = ScalarCountAgent.execute(AgentBaseRequest {
            task: "删除文件 X",
            context: Some("很长的材料文本"),
        });
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match refused.as_mut().poll(&mut cx) {
            Poll::Ready(Err(error)) => {
                assert_eq!(error.kind, BaseCallErrorKind::Unsupported);
                let detail = error.detail.unwrap_or_default();
                assert!(
                    detail.contains(SUPPORTED_TASK),
                    "the detail names the supported task: {detail}"
                );
                assert!(
                    !detail.contains("删除文件") && !detail.contains("很长的材料文本"),
                    "the detail must not echo the request: {detail}"
                );
            }
            other => panic!("an unsupported task must fail, got {other:?}"),
        }
    }

    // 独立多次调用回归: each call is driven through a freshly bound implementation value, and an
    // earlier call's material does not change a later one. This says nothing about reusing one bound
    // `&dyn` slot across calls — that sequence is evidenced by the external target instead.
    #[test]
    fn b2_c6_repeated_independent_calls_do_not_carry_state() {
        let first = String::from("你好");
        assert_eq!(
            report_for(SUPPORTED_TASK, Some(&first)),
            "本次完成了计数：2 个 Unicode 标量值。"
        );

        let refused = String::from("删除文件 X");
        assert_eq!(drive(&refused, None), Err(BaseCallErrorKind::Unsupported));

        assert_eq!(report_for(SUPPORTED_TASK, None), MISSING_MATERIAL_REPORT);

        let empty = String::new();
        assert_eq!(
            report_for(SUPPORTED_TASK, Some(&empty)),
            "本次完成了计数：0 个 Unicode 标量值。"
        );

        let mixed = String::from("aé😀");
        assert_eq!(
            report_for(SUPPORTED_TASK, Some(&mixed)),
            "本次完成了计数：3 个 Unicode 标量值。"
        );

        // The earlier material still counts the same: no result leaked forward.
        assert_eq!(
            drive(SUPPORTED_TASK, Some(&first)),
            Ok("本次完成了计数：2 个 Unicode 标量值。".to_string())
        );
        // The borrowed material is still usable after the calls returned.
        assert_eq!(first, "你好");
        assert_eq!(mixed, "aé😀");
    }
}
