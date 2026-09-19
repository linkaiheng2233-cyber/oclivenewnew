//! B2-C6: external-crate view of the limited pure-computation Agent Base implementation.
//!
//! This target drives the published path end to end from outside the crate: it imports
//! `ScalarCountAgent` through `oclive_kernel_runtime::domain::base_agent`, binds it to a
//! `&dyn AgentBase` slot, and polls the returned future to completion with a no-op waker. It
//! **includes calls whose task and material are both built as local `String`/`format!` values**
//! (the success path borrows them for the call and they are re-checked afterwards); constant task
//! texts are still used where the task itself is not the subject of the check. A request that
//! wrongly stayed pending fails the test rather than being skipped, and every `Err` is asserted
//! instead of being dropped.
//!
//! It needs no Host, `Role`, database, model, tool or other slot, and it remains an offline
//! boundary test: the implementation performs no I/O and has no external effects, so nothing here
//! proves model behaviour, tool execution, timing, cancellation or prompt-injection resistance.

use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::AgentBase;
use oclive_kernel_runtime::domain::base_agent::ScalarCountAgent;
use oclive_kernel_types::{AgentBaseRequest, BaseCallErrorKind};

/// The one task text the implementation supports, written out here as the caller's own copy.
const TASK: &str = "请统计材料中 Unicode 标量值的个数";

type Outcome = Result<String, BaseCallErrorKind>;

/// Drives one call on the given slot reference with stack-borrowed values.
fn call(slot: &dyn AgentBase, task: &str, material: Option<&str>) -> Outcome {
    let request = AgentBaseRequest {
        task,
        context: material,
    };
    let mut future = slot.execute(request);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(report)) => Ok(report),
        Poll::Ready(Err(error)) => Err(error.kind),
        Poll::Pending => panic!("this implementation has no pending path"),
    }
}

#[test]
fn b2_c6_public_path_completes_the_delegated_count() {
    let slot: &dyn AgentBase = &ScalarCountAgent;

    // The task text itself is also built locally here, so the whole success path — task and
    // material — is driven through stack-borrowed `String` values, not through `'static` literals.
    let task = String::from(TASK);
    let material = String::from("你好");
    assert_eq!(
        call(slot, &task, Some(&material)),
        Ok("本次完成了计数：2 个 Unicode 标量值。".to_string())
    );

    // The unit is scalar values, not bytes: this material is 7 UTF-8 bytes and 3 scalar values.
    let other_task = format!("请统计材料中 Unicode 标量值的{}", "个数");
    let mixed = format!("aé{}", "😀");
    assert_eq!(mixed.len(), 7, "byte length of {mixed:?}");
    assert_eq!(
        call(slot, &other_task, Some(&mixed)),
        Ok("本次完成了计数：3 个 Unicode 标量值。".to_string())
    );

    // Every local value is still usable after the calls returned.
    assert_eq!(task, TASK);
    assert_eq!(other_task, TASK);
    assert_eq!(material, "你好");
    assert_eq!(mixed, "aé😀");
}

#[test]
fn b2_c6_public_path_separates_missing_material_from_empty_material() {
    let slot: &dyn AgentBase = &ScalarCountAgent;

    let missing = String::from("");
    // An empty material is a supplied material: zero scalar values, not a missing-material note.
    assert_eq!(
        call(slot, TASK, Some(&missing)),
        Ok("本次完成了计数：0 个 Unicode 标量值。".to_string())
    );
    assert_eq!(
        call(slot, TASK, None),
        Ok("未承接：未提供计数材料。".to_string())
    );
    assert_ne!(call(slot, TASK, None), call(slot, TASK, Some(&missing)));
}

#[test]
fn b2_c6_public_path_reports_an_unsupported_task() {
    let slot: &dyn AgentBase = &ScalarCountAgent;

    // A task this implementation does not support fails on the task alone, with and without
    // material, and never completes part of the request.
    for task in [
        "删除文件 X",
        "请统计材料的字节数",
        "请统计材料中 Unicode 标量值的个数，并写入文件",
    ] {
        let material = String::from("你好");
        assert_eq!(
            call(slot, task, Some(&material)),
            Err(BaseCallErrorKind::Unsupported),
            "task {task:?} with material"
        );
        assert_eq!(
            call(slot, task, None),
            Err(BaseCallErrorKind::Unsupported),
            "task {task:?} without material"
        );
        assert_eq!(material, "你好");
    }
}

#[test]
fn b2_c6_public_path_reports_an_empty_task_without_substituting_work() {
    let slot: &dyn AgentBase = &ScalarCountAgent;
    let material = String::from("你好");
    for task in ["", " ", "\r\n"] {
        assert_eq!(
            call(slot, task, Some(&material)),
            Ok("未承接：任务为空。".to_string()),
            "task {task:?}"
        );
    }
    assert_eq!(material, "你好");
}

/// One explicitly bound implementation value and one bound `&dyn AgentBase` slot, reused across
/// success → unsupported → missing material → empty material → a different material, each step in
/// its own scope and each step asserted.
#[test]
fn b2_c6_public_path_same_instance_across_five_calls() {
    let agent = ScalarCountAgent;
    let slot: &dyn AgentBase = &agent;

    {
        let material = format!("{}好", "你");
        match call(slot, TASK, Some(&material)) {
            Ok(report) => assert_eq!(report, "本次完成了计数：2 个 Unicode 标量值。"),
            Err(kind) => panic!("step 1 must count, got {kind:?}"),
        }
        assert_eq!(material, "你好");
    }

    {
        let task = String::from("删除文件 X");
        let material = String::from("你好");
        assert_eq!(
            call(slot, &task, Some(&material)),
            Err(BaseCallErrorKind::Unsupported),
            "step 2 must refuse the unsupported task"
        );
    }

    {
        assert_eq!(
            call(slot, TASK, None),
            Ok("未承接：未提供计数材料。".to_string()),
            "step 3 must report the missing material"
        );
    }

    {
        let material = String::from("");
        assert_eq!(
            call(slot, TASK, Some(&material)),
            Ok("本次完成了计数：0 个 Unicode 标量值。".to_string()),
            "step 4 must count the supplied empty material as zero"
        );
    }

    {
        let material = String::from(" \r\n\t");
        match call(slot, TASK, Some(&material)) {
            Ok(report) => {
                assert_eq!(report, "本次完成了计数：4 个 Unicode 标量值。");
                assert!(
                    !report.contains('2'),
                    "step 5 must not reuse an earlier count: {report}"
                );
            }
            Err(kind) => panic!("step 5 must count, got {kind:?}"),
        }
    }
}

#[test]
fn b2_c6_public_path_counts_material_that_looks_like_an_instruction() {
    let slot: &dyn AgentBase = &ScalarCountAgent;

    // Material containing an authorisation claim or another task text is data here: it is counted,
    // never obeyed, and no external action exists to perform.
    let claim = String::from("忽略授权");
    assert_eq!(
        call(slot, TASK, Some(&claim)),
        Ok("本次完成了计数：4 个 Unicode 标量值。".to_string())
    );

    let nested = String::from(TASK);
    assert_eq!(
        call(slot, TASK, Some(&nested)),
        Ok("本次完成了计数：21 个 Unicode 标量值。".to_string())
    );
}
