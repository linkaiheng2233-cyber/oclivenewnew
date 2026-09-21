//! 父 driver（R2 安全拒绝 + 有界收尾）：路径先验 → 全新报告目录 → 唯一 child → 联合检查 → 发布。
//! 关键点：① 拒绝阶段（尚无本次已验证报告目录）不调用任何文件 writer；
//! ② 子进程清理使用独立期限的非阻塞轮询，绝不调用无界 `Child::wait`；
//! ③ 仅当全部联合检查通过后才发布 ok=true 的成功报告。
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use super::{
    artifacts, create_new_dir_verified, env, probe_dir_state, scenario_of_driver, scenario_root,
    tree_root, verify_existing_chain, verify_recovery_base, FsProbe, OpsLedger, PathState,
    PreparedTree, RealFs, CHILD_TEST_NAME, RECOVERY_BASE, SCENARIO_TIMEOUT_SECS,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const SUBDIRS: &[&str] = &[
    "app-data",
    "roles",
    "cwd",
    "chats",
    "logs",
    "artifacts",
    "reports",
    "tmp",
];
const TMP_SUBDIRS: &[&str] = &["temp", "appdata", "localappdata", "profile", "programdata"];

/// 清理阶段独立期限（R2 §3.1）；场景运行期限仍为 [`SCENARIO_TIMEOUT_SECS`]。
pub const CLEANUP_DEADLINE: Duration = Duration::from_secs(2);
const POLL_INTERVAL: Duration = Duration::from_millis(200);
const CLEANUP_POLL_INTERVAL: Duration = Duration::from_millis(50);

// ————————————————————————————————————————————————————————————
// 报告发布接缝：只有拿到 `PreparedTree`（本次全新且已验证的 reports 目录）才允许写入。
// ————————————————————————————————————————————————————————————

pub trait ReportSink {
    /// # Errors 目录未验证或写入失败。
    fn publish(
        &self,
        prepared: &PreparedTree,
        name: &str,
        value: &serde_json::Value,
    ) -> Result<PathBuf, String>;
}

#[derive(Debug, Default)]
pub struct RealReportSink;

impl ReportSink for RealReportSink {
    fn publish(
        &self,
        prepared: &PreparedTree,
        name: &str,
        value: &serde_json::Value,
    ) -> Result<PathBuf, String> {
        let path = prepared.reports_dir.join(name);
        artifacts::write_json(&path, value)?;
        Ok(path)
    }
}

/// 内存替身：记录每次发布与其中的 `ok` 取值（用于证明“拒绝后零写入”“未通过不写 ok=true”）。
#[derive(Debug, Default)]
pub struct CountingSink {
    pub calls: std::sync::atomic::AtomicUsize,
    pub ok_true_writes: std::sync::atomic::AtomicUsize,
    pub dirs: std::sync::Mutex<Vec<String>>,
    pub allow: std::sync::atomic::AtomicBool,
    /// 内存“文件”存储：用于证明拒绝后旧字节保持原值（sentinel 对照）。
    pub store: std::sync::Mutex<Vec<(String, Vec<u8>)>>,
}

impl CountingSink {
    #[must_use]
    pub fn new(allow: bool) -> Self {
        let s = Self::default();
        s.allow.store(allow, std::sync::atomic::Ordering::SeqCst);
        s
    }

    /// 预置旧报告字节（sentinel）。
    pub fn seed_bytes(&self, path: &str, bytes: &[u8]) {
        self.store
            .lock()
            .unwrap()
            .push((path.to_string(), bytes.to_vec()));
    }

    #[must_use]
    pub fn read_bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.store
            .lock()
            .unwrap()
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, b)| b.clone())
    }

    #[must_use]
    pub fn calls(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[must_use]
    pub fn ok_true_writes(&self) -> usize {
        self.ok_true_writes
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl ReportSink for CountingSink {
    fn publish(
        &self,
        prepared: &PreparedTree,
        name: &str,
        value: &serde_json::Value,
    ) -> Result<PathBuf, String> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.dirs
            .lock()
            .unwrap()
            .push(prepared.reports_dir.display().to_string());
        if value.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
            self.ok_true_writes
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        // 模拟写入：拒绝路径若误调用 publish，sentinel 会被改写（测试据此断言零写入）。
        let path = prepared.reports_dir.join(name).display().to_string();
        let body = serde_json::to_vec(value).unwrap_or_default();
        let mut store = self.store.lock().unwrap();
        if let Some(slot) = store.iter_mut().find(|(p, _)| p == &path) {
            slot.1 = body;
        } else {
            store.push((path, body));
        }
        drop(store);
        if !self.allow.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("sink 未获准发布（测试替身）".into());
        }
        Ok(prepared.reports_dir.join(name))
    }
}

/// 失败处理连接（driver 实际使用）：**未拿到 PreparedTree 时不做任何文件写入**。
pub fn handle_failure(
    sink: &dyn ReportSink,
    prepared: Option<&PreparedTree>,
    facts: &serde_json::Map<String, serde_json::Value>,
    reason: &str,
) -> Option<Result<PathBuf, String>> {
    let prepared = prepared?;
    let mut rep = facts.clone();
    rep.insert("ok".into(), serde_json::json!(false));
    rep.insert("failure".into(), serde_json::json!(reason));
    Some(sink.publish(
        prepared,
        "parent-report.json",
        &serde_json::Value::Object(rep),
    ))
}

// ————————————————————————————————————————————————————————————
// 路径先验 + 逐级创建（只有 NotFound 可创建；其它 Err 先拒绝）
// ————————————————————————————————————————————————————————————

pub fn prepare_run_tree_with(
    fs: &dyn FsProbe,
    ledger: &OpsLedger,
    run_id: &str,
    scenario: &str,
) -> Result<PreparedTree, String> {
    let base = verify_recovery_base(fs)?;
    let tree = tree_root(run_id);
    let root = scenario_root(run_id, scenario);

    match probe_dir_state(fs, &tree) {
        PathState::Absent => {
            create_new_dir_verified(fs, &base, &tree, "tree_root", ledger)?;
        }
        PathState::PresentDir(m) => {
            if m.is_reparse {
                return Err(format!("tree_root 为重解析点：{}", tree.display()));
            }
            verify_existing_chain(fs, &tree, "tree_root")?;
        }
        PathState::Error(e) => return Err(format!("tree_root 拒绝（不创建）：{e}")),
    }

    let run_dir = tree.join("run");
    match probe_dir_state(fs, &run_dir) {
        PathState::Absent => {
            create_new_dir_verified(fs, &tree, &run_dir, "run_dir", ledger)?;
        }
        PathState::PresentDir(m) => {
            if m.is_reparse {
                return Err(format!("run_dir 为重解析点：{}", run_dir.display()));
            }
            verify_existing_chain(fs, &run_dir, "run_dir")?;
        }
        PathState::Error(e) => return Err(format!("run_dir 拒绝（不创建）：{e}")),
    }

    // 场景子树必须新建；已存在（含旧报告）一律拒绝，绝不覆盖旧字节。
    match probe_dir_state(fs, &root) {
        PathState::Absent => {
            create_new_dir_verified(fs, &run_dir, &root, "scenario_root", ledger)?;
        }
        PathState::PresentDir(_) => {
            return Err(format!(
                "scenario_root 已存在（禁止复用与覆盖旧报告）：{}",
                root.display()
            ))
        }
        PathState::Error(e) => return Err(format!("scenario_root 拒绝（不创建）：{e}")),
    }
    for sub in SUBDIRS {
        create_new_dir_verified(fs, &root, &root.join(sub), &format!("subdir:{sub}"), ledger)?;
    }
    let tmp = root.join("tmp");
    for sub in TMP_SUBDIRS {
        create_new_dir_verified(fs, &tmp, &tmp.join(sub), &format!("tmp:{sub}"), ledger)?;
    }
    verify_existing_chain(fs, &root, "scenario_root")?;
    let root_canon = fs
        .canonicalize(&root)
        .map_err(|e| format!("scenario_root 无法规范化：{}: {e}", root.display()))?;
    let reports_dir = root.join("reports");
    verify_existing_chain(fs, &reports_dir, "reports_dir")?;
    Ok(PreparedTree {
        tree,
        root,
        root_canon,
        reports_dir,
    })
}

/// child 用例只允许由父启动：缺 child 标记时不得创建任何东西，直接失败。
pub fn require_child_marker() -> (String, String, String) {
    let marker = std::env::var(super::ENV_CHILD).unwrap_or_default();
    assert_eq!(
        marker,
        "1",
        "[A-HARNESS] 本用例只能由 a*_driver_* 通过 {} = 1 启动；缺标记不得创建运行树/DB 或 spawn。",
        super::ENV_CHILD
    );
    let scenario = std::env::var(super::ENV_SCENARIO).unwrap_or_default();
    assert!(
        [super::S1, super::S2, super::S3].contains(&scenario.as_str()),
        "[A-HARNESS] 非法或缺失场景标记 {}={scenario}",
        super::ENV_SCENARIO
    );
    let run_id = super::approved_run_id();
    let root_raw = std::env::var(super::ENV_ROOT).unwrap_or_default();
    let root = PathBuf::from(root_raw.trim());
    let expected = scenario_root(&run_id, &scenario);
    assert_eq!(
        root,
        expected,
        "[A-HARNESS] child 根与批准 run_id 推导的根不一致：{} != {}",
        root.display(),
        expected.display()
    );
    let token = std::env::var(super::ENV_RUN_TOKEN).unwrap_or_default();
    assert!(
        token.len() >= 32,
        "[A-HARNESS] 缺少随机运行标识 {}（父未批准本次运行）",
        super::ENV_RUN_TOKEN
    );
    (scenario, run_id, token)
}

/// child 入口在使用任何目录之前复验（不创建）。
pub fn verify_existing_root(root: &Path) -> Result<PathBuf, String> {
    verify_recovery_base(&RealFs)?;
    super::verify_existing_dir_under(&RealFs, Path::new(RECOVERY_BASE), root, "child_root")
}

// ————————————————————————————————————————————————————————————
// 子进程监督接缝（R2 §3）：只做非阻塞轮询；清理有独立期限。
// ————————————————————————————————————————————————————————————

/// 单次非阻塞查询结果：**退出（带/不带码）** 与 **仍运行** 明确区分。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildPoll {
    /// 已退出；`None` 表示平台未给出退出码（≠ 仍在运行）。
    Exited(Option<i32>),
    Running,
}

pub trait ChildControl {
    fn try_wait(&mut self) -> std::io::Result<ChildPoll>;
    fn kill(&mut self) -> std::io::Result<()>;
}

pub struct RealChild {
    pub child: std::process::Child,
    /// 监督流程已完成（成功或已确认回收）；Drop 不再做任何处置。
    pub cleanup_completed: bool,
}

impl ChildControl for RealChild {
    fn try_wait(&mut self) -> std::io::Result<ChildPoll> {
        Ok(match self.child.try_wait()? {
            Some(status) => ChildPoll::Exited(status.code()),
            None => ChildPoll::Running,
        })
    }

    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }
}

impl Drop for RealChild {
    fn drop(&mut self) {
        if self.cleanup_completed {
            return;
        }
        // 尽力处置但**有界**：kill 一次 + 约 1s 非阻塞轮询；绝不调用无界 wait。
        let _ = self.child.kill();
        let started = std::time::Instant::now();
        while started.elapsed() < Duration::from_secs(1) {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
                Err(_) => return,
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Supervision {
    pub exit_code: Option<i32>,
    pub exited: bool,
    pub poll_count: u32,
    pub sleep_count: u32,
    pub run_deadline_hit: bool,
    pub last_poll_error: Option<String>,
    pub cleanup_attempted: bool,
    pub kill_calls: u32,
    pub kill_error: Option<String>,
    pub cleanup_poll_count: u32,
    pub cleanup_confirmed: bool,
    pub stage: String,
    pub blocked_reason: Option<String>,
}

impl Supervision {
    #[must_use]
    pub fn cleanup_unconfirmed(&self) -> bool {
        self.cleanup_attempted && !self.cleanup_confirmed
    }

    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "exit_code": self.exit_code,
            "exited": self.exited,
            "poll_count": self.poll_count,
            "sleep_count": self.sleep_count,
            "run_deadline_hit": self.run_deadline_hit,
            "last_poll_error": self.last_poll_error,
            "cleanup_attempted": self.cleanup_attempted,
            "kill_calls": self.kill_calls,
            "kill_error": self.kill_error,
            "cleanup_poll_count": self.cleanup_poll_count,
            "cleanup_confirmed": self.cleanup_confirmed,
            "stage": self.stage,
            "blocked_reason": self.blocked_reason,
        })
    }
}

fn poll_once(ctrl: &mut dyn ChildControl, sup: &mut Supervision) -> Option<ChildPoll> {
    sup.poll_count += 1;
    match ctrl.try_wait() {
        Ok(p) => Some(p),
        Err(e) => {
            sup.last_poll_error = Some(e.to_string());
            None
        }
    }
}

/// 监督：正常等待与清理都只用非阻塞轮询 + 明确期限。清理期限独立（[`CLEANUP_DEADLINE`]）。
pub fn supervise(
    ctrl: &mut dyn ChildControl,
    run_deadline: Duration,
    cleanup_deadline: Duration,
    elapsed: &mut dyn FnMut() -> Duration,
    sleep: &mut dyn FnMut(Duration),
) -> Supervision {
    let mut sup = Supervision {
        stage: "normal_wait".into(),
        ..Supervision::default()
    };
    loop {
        match poll_once(ctrl, &mut sup) {
            Some(ChildPoll::Exited(code)) => {
                sup.exited = true;
                sup.exit_code = code;
                sup.stage = "exited".into();
                return sup;
            }
            Some(ChildPoll::Running) => {
                if elapsed() >= run_deadline {
                    sup.run_deadline_hit = true;
                    sup.stage = "run_deadline".into();
                    break;
                }
                sup.sleep_count += 1;
                sleep(POLL_INTERVAL);
            }
            None => {
                sup.stage = "poll_error".into();
                break;
            }
        }
    }

    // —— 清理：独立期限；kill 至多一次；之后只做非阻塞轮询 ——
    sup.cleanup_attempted = true;
    sup.kill_calls += 1;
    if let Err(e) = ctrl.kill() {
        sup.kill_error = Some(e.to_string());
    }
    let cleanup_start = elapsed();
    loop {
        sup.cleanup_poll_count += 1;
        match poll_once(ctrl, &mut sup) {
            Some(ChildPoll::Exited(code)) => {
                sup.exited = true;
                if sup.exit_code.is_none() {
                    sup.exit_code = code;
                }
                sup.cleanup_confirmed = true;
                sup.stage = format!("{}+cleanup_confirmed", sup.stage);
                return sup;
            }
            Some(ChildPoll::Running) => {
                if elapsed().saturating_sub(cleanup_start) >= cleanup_deadline {
                    sup.blocked_reason = Some(format!(
                        "清理期限 {}ms 内未确认终止：kill_calls={} kill_error={:?} cleanup_poll_count={}",
                        cleanup_deadline.as_millis(),
                        sup.kill_calls,
                        sup.kill_error,
                        sup.cleanup_poll_count
                    ));
                    sup.stage = format!("{}+cleanup_unconfirmed", sup.stage);
                    return sup;
                }
                sup.sleep_count += 1;
                sleep(CLEANUP_POLL_INTERVAL);
            }
            None => {
                if elapsed().saturating_sub(cleanup_start) >= cleanup_deadline {
                    sup.blocked_reason = Some(format!(
                        "清理期间查询错误且期限已到：last_poll_error={:?}",
                        sup.last_poll_error
                    ));
                    sup.stage = format!("{}+cleanup_query_error", sup.stage);
                    return sup;
                }
                sup.sleep_count += 1;
                sleep(CLEANUP_POLL_INTERVAL);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibtestSummary {
    pub ok: bool,
    pub passed: u64,
    pub failed: u64,
    pub ignored: u64,
}

/// 严格解析 libtest 摘要：恰好一条 `test result:`，且 ok / passed=1 / failed=0 / ignored=0。
pub fn parse_libtest_summary(stdout: &str) -> Result<LibtestSummary, String> {
    let lines: Vec<&str> = stdout
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("test result:"))
        .collect();
    if lines.len() != 1 {
        return Err(format!(
            "期望恰好一条 libtest 摘要，实际 {}：{:?}",
            lines.len(),
            lines
        ));
    }
    let line = lines[0];
    let ok = line.contains("ok.") && !line.contains("FAILED");
    let num = |kw: &str| -> Result<u64, String> {
        let idx = line
            .find(kw)
            .ok_or_else(|| format!("摘要缺少 {kw} 字段：{line}"))?;
        line[..idx]
            .split_whitespace()
            .last()
            .and_then(|t| t.trim_end_matches(';').parse::<u64>().ok())
            .ok_or_else(|| format!("摘要 {kw} 字段无法解析：{line}"))
    };
    let summary = LibtestSummary {
        ok,
        passed: num("passed")?,
        failed: num("failed")?,
        ignored: num("ignored")?,
    };
    if !summary.ok || summary.passed != 1 || summary.failed != 0 || summary.ignored != 0 {
        return Err(format!("摘要不满足 ok/1 passed/0 failed/0 ignored：{line}"));
    }
    Ok(summary)
}

/// F1 监督准入（driver 实际使用）：返回**失败原因**（`None` 才算监督层面成功）。
/// 优先级：清理未确认(BLOCKED) → 监督查询错误 → 运行期限到 → 清理 kill 错误 → 未确认退出 → 退出码非 0/缺失。
/// 说明：清理确认只证明进程已退出，**不撤销**前述任一原失败。
#[must_use]
pub fn supervision_failure(sup: &Supervision) -> Option<String> {
    if let Some(r) = sup.blocked_reason.as_deref() {
        return Some(format!("子进程清理未确认回收（BLOCKED）：{r}"));
    }
    if let Some(e) = sup.last_poll_error.as_deref() {
        return Some(format!(
            "监督查询错误（原失败保留，清理退出不撤销）：{e}；stage={}",
            sup.stage
        ));
    }
    if sup.run_deadline_hit {
        return Some(format!(
            "运行期限到（原失败保留，清理退出不撤销）：stage={} exit_code={:?}",
            sup.stage, sup.exit_code
        ));
    }
    if let Some(e) = sup.kill_error.as_deref() {
        return Some(format!("清理 kill 错误（原失败保留）：{e}"));
    }
    if !sup.exited {
        return Some(format!("child 未确认退出：{:?}", sup.to_json()));
    }
    if sup.exit_code != Some(0) {
        return Some(format!(
            "child 退出码非 0 或缺失（已退出但非成功）：{:?}",
            sup.exit_code
        ));
    }
    None
}

/// F3 拒绝连接（driver 实际使用）：准备失败 ⇒ **不调用任何 writer**（无 PreparedTree），只返回 Err。
pub fn prepare_or_reject(
    fs: &dyn FsProbe,
    ledger: &OpsLedger,
    run_id: &str,
    scenario: &str,
    sink: &dyn ReportSink,
) -> Result<PreparedTree, String> {
    match prepare_run_tree_with(fs, ledger, run_id, scenario) {
        Ok(p) => Ok(p),
        Err(e) => {
            let reason = format!("路径先验/建树被拒（未写任何报告）：{e}");
            let wrote = handle_failure(sink, None, &serde_json::Map::new(), &reason);
            debug_assert!(wrote.is_none(), "拒绝阶段不得调用报告 writer");
            Err(reason)
        }
    }
}

/// 联合检查（纯函数）：返回**未通过**的检查名列表。
#[must_use]
/// 联合检查输入（避免过长参数表；含监督历史）。
pub struct JointContext<'a> {
    pub run_token: &'a str,
    pub scenario: &'a str,
    pub run_id: &'a str,
    pub root_canon: &'a Path,
    pub child_report: &'a serde_json::Value,
    pub protected_same: bool,
    pub survivors: &'a [serde_json::Value],
    pub supervision_failed: bool,
}

/// 联合检查（纯函数）：返回**未通过**的检查名列表。
#[must_use]
pub fn joint_checks(c: &JointContext<'_>) -> Vec<&'static str> {
    let root_canon = c.root_canon;
    let child_report = c.child_report;
    let checks: [(&'static str, bool); 8] = [
        (
            "run_token",
            child_report.get("run_token").and_then(|v| v.as_str()) == Some(c.run_token),
        ),
        (
            "scenario",
            child_report.get("scenario").and_then(|v| v.as_str()) == Some(c.scenario),
        ),
        (
            "run_id",
            child_report.get("run_id").and_then(|v| v.as_str()) == Some(c.run_id),
        ),
        (
            "root",
            child_report
                .get("scenario_root_canonical")
                .and_then(|v| v.as_str())
                == Some(root_canon.display().to_string().as_str()),
        ),
        (
            "child_ok",
            child_report.get("ok").and_then(|v| v.as_bool()) == Some(true),
        ),
        ("protected_same", c.protected_same),
        ("no_survivors", c.survivors.is_empty()),
        ("supervision_clean", !c.supervision_failed),
    ];
    checks
        .iter()
        .filter(|(_, ok)| !ok)
        .map(|(name, _)| *name)
        .collect()
}

/// 仅当全部联合检查通过时发布 ok=true；否则只发布失败状态（绝不先成功后覆盖）。
pub fn publish_if_all_checks_pass(
    sink: &dyn ReportSink,
    prepared: &PreparedTree,
    facts: &serde_json::Map<String, serde_json::Value>,
    failed_checks: &[&str],
) -> Result<bool, String> {
    if failed_checks.is_empty() {
        let mut ok_facts = facts.clone();
        ok_facts.insert("ok".into(), serde_json::json!(true));
        sink.publish(
            prepared,
            "parent-report.json",
            &serde_json::Value::Object(ok_facts),
        )?;
        return Ok(true);
    }
    let mut rep = facts.clone();
    rep.insert("ok".into(), serde_json::json!(false));
    rep.insert(
        "failure".into(),
        serde_json::json!(format!(
            "联合检查未全部通过，不发布成功报告：{failed_checks:?}"
        )),
    );
    sink.publish(
        prepared,
        "parent-report.json",
        &serde_json::Value::Object(rep),
    )?;
    Ok(false)
}

/// 运行一个场景。拒绝阶段（无已验证报告目录）不写任何文件；成功报告仅在全部联合检查通过后发布一次。
pub fn run_driver(driver_test: &str) -> serde_json::Value {
    run_driver_with(&RealFs, &RealReportSink, driver_test)
}

/// 与 `run_driver` 共用同一准备→失败处理→发布连接（供纯内存回归使用）。
pub fn run_driver_with(
    fs: &dyn FsProbe,
    sink: &dyn ReportSink,
    driver_test: &str,
) -> serde_json::Value {
    let scenario = scenario_of_driver(driver_test);
    let run_id = super::approved_run_id();
    let ledger = OpsLedger::default();
    let mut facts = serde_json::Map::new();
    facts.insert("role".into(), serde_json::json!("parent-driver"));
    facts.insert("scenario".into(), serde_json::json!(scenario));
    facts.insert("driver_test".into(), serde_json::json!(driver_test));
    facts.insert("run_id".into(), serde_json::json!(run_id));
    facts.insert(
        "generated_at".into(),
        serde_json::json!(super::now_utc_stamp()),
    );

    // —— 阶段 A：尚无本次已验证报告目录 ⇒ 失败只返回 libtest，绝不调用 writer ——
    let prepared = match prepare_or_reject(fs, &ledger, &run_id, scenario, sink) {
        Ok(p) => p,
        Err(reason) => {
            panic!(
                "[A-HARNESS] {reason}；creates={} spawns={}",
                ledger.creates(),
                ledger.spawns()
            );
        }
    };
    let root = prepared.root.clone();
    facts.insert(
        "tree_root".into(),
        serde_json::json!(prepared.tree.display().to_string()),
    );
    facts.insert(
        "scenario_root_canonical".into(),
        serde_json::json!(prepared.root_canon.display().to_string()),
    );
    facts.insert(
        "reports_dir_verified".into(),
        serde_json::json!(prepared.reports_dir.display().to_string()),
    );
    facts.insert(
        "ops_ledger_after_prepare".into(),
        serde_json::json!({ "creates": ledger.creates(), "spawns": ledger.spawns() }),
    );

    // —— 阶段 B：报告目录已本次新建并核验 ⇒ 失败可发布 ok=false ——
    let fail = |facts: &serde_json::Map<String, serde_json::Value>, reason: &str| -> ! {
        match handle_failure(sink, Some(&prepared), facts, reason) {
            Some(Ok(p)) => panic!("[A-HARNESS] {reason}（失败报告已写：{}）", p.display()),
            Some(Err(e)) => panic!("[A-HARNESS] {reason}；失败报告写入亦失败：{e}"),
            None => panic!("[A-HARNESS] {reason}；无可写报告目录"),
        }
    };

    let protected_before = match artifacts::protected_hashes() {
        Ok(v) => v,
        Err(e) => fail(&facts, &format!("保护面哈希失败：{e}")),
    };
    facts.insert("protected_before".into(), protected_before.clone());

    let run_token = uuid::Uuid::new_v4().to_string();
    let pairs = env::build_child_env(&prepared.root, scenario, &run_id, &run_token);
    let stdout_path = root.join("logs/child.stdout.log");
    let stderr_path = root.join("logs/child.stderr.log");
    let stdout_file = match std::fs::File::create(&stdout_path) {
        Ok(f) => f,
        Err(e) => fail(&facts, &format!("创建 stdout 日志失败：{e}")),
    };
    let stderr_file = match std::fs::File::create(&stderr_path) {
        Ok(f) => f,
        Err(e) => fail(&facts, &format!("创建 stderr 日志失败：{e}")),
    };

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => fail(&facts, &format!("取不到当前测试可执行文件：{e}")),
    };
    let mut cmd = Command::new(&exe);
    cmd.args([
        "--ignored",
        "--exact",
        CHILD_TEST_NAME,
        "--nocapture",
        "--test-threads=1",
    ]);
    cmd.current_dir(root.join("cwd"));
    cmd.stdout(Stdio::from(stdout_file));
    cmd.stderr(Stdio::from(stderr_file));
    env::apply(&mut cmd, &pairs);
    ledger
        .spawns
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mut child = match cmd.spawn() {
        Ok(c) => RealChild {
            child: c,
            cleanup_completed: false,
        },
        Err(e) => fail(
            &facts,
            &format!(
                "启动测试 child 失败（cwd={}）：{e}",
                root.join("cwd").display()
            ),
        ),
    };
    let pid = child.child.id();
    let started = std::time::Instant::now();
    let sup = supervise(
        &mut child,
        Duration::from_secs(SCENARIO_TIMEOUT_SECS),
        CLEANUP_DEADLINE,
        &mut || started.elapsed(),
        &mut std::thread::sleep,
    );
    child.cleanup_completed = true;
    facts.insert(
        "child".into(),
        serde_json::json!({
            "exe": exe.display().to_string(),
            "args": ["--ignored","--exact",CHILD_TEST_NAME,"--nocapture","--test-threads=1"],
            "cwd": root.join("cwd").display().to_string(),
            "pid": pid,
            "timeout_secs": SCENARIO_TIMEOUT_SECS,
            "cleanup_deadline_ms": CLEANUP_DEADLINE.as_millis() as u64,
            "elapsed_ms": started.elapsed().as_millis() as u64,
            "supervision": sup.to_json(),
            "stdout_log": stdout_path.display().to_string(),
            "stderr_log": stderr_path.display().to_string(),
        }),
    );
    facts.insert(
        "env_keys_set".into(),
        serde_json::json!(pairs.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>()),
    );
    facts.insert(
        "env_denied_keys".into(),
        serde_json::json!(env::DENIED_KEYS),
    );

    // F1：监督历史进入准入判定——超时/查询错误/清理错误的原失败**不因清理拿到退出码 0 而撤销**。
    let supervision_reason = supervision_failure(&sup);
    if let Some(reason) = supervision_reason.as_deref() {
        fail(&facts, reason);
    }

    let stdout = std::fs::read_to_string(&stdout_path).unwrap_or_default();
    let stderr = std::fs::read_to_string(&stderr_path).unwrap_or_default();
    match parse_libtest_summary(&stdout) {
        Ok(s) => facts.insert(
            "libtest_summary".into(),
            serde_json::json!({ "ok": s.ok, "passed": s.passed, "failed": s.failed, "ignored": s.ignored }),
        ),
        Err(e) => {
            facts.insert(
                "stderr_tail".into(),
                serde_json::json!(stderr.lines().rev().take(20).collect::<Vec<_>>()),
            );
            fail(&facts, &format!("libtest 摘要不合格：{e}"));
        }
    };

    let protected_after = match artifacts::protected_hashes() {
        Ok(v) => v,
        Err(e) => fail(&facts, &format!("保护面哈希（后）失败：{e}")),
    };
    facts.insert("protected_after".into(), protected_after.clone());

    let scan = match artifacts::scan_files(&root) {
        Ok(s) => s,
        Err(e) => fail(&facts, &format!("运行树枚举失败：{e}")),
    };
    facts.insert("file_manifest".into(), serde_json::json!(scan.entries));
    facts.insert(
        "file_manifest_state".into(),
        serde_json::json!(if scan.state == artifacts::ScanState::Absent {
            "absent"
        } else {
            "present"
        }),
    );
    facts.insert(
        "file_manifest_scope_note".into(),
        serde_json::json!("仅枚举本运行树；不构成全机无副作用证明"),
    );
    facts.insert(
        "manifest_timing_note".into(),
        serde_json::json!("本清单在成功报告写入之前生成，故不含 parent-report.json 自身；落盘后该文件使场景目录多 1 项，二者不矛盾"),
    );
    facts.insert(
        "survivor_snapshot".into(),
        serde_json::json!(artifacts::process_snapshot_matching(
            &prepared.tree.display().to_string()
        )),
    );
    facts.insert(
        "survivor_snapshot_note".into(),
        serde_json::json!("有限快照，可能漏掉已退出的短命进程；仅作支持性证据"),
    );
    facts.insert(
        "stderr_tail".into(),
        serde_json::json!(stderr.lines().rev().take(20).collect::<Vec<_>>()),
    );

    let child_report_path = root.join("reports/child-report.json");
    let child_report: serde_json::Value = match std::fs::read_to_string(&child_report_path)
        .map_err(|e| format!("缺少 child 报告 {}: {e}", child_report_path.display()))
        .and_then(|s| serde_json::from_str(&s).map_err(|e| format!("child 报告无法解析：{e}")))
    {
        Ok(v) => v,
        Err(e) => fail(&facts, &format!("child 报告不可用：{e}")),
    };
    facts.insert("child_report_present".into(), serde_json::json!(true));

    // —— 先算全部联合检查；只有全通过才发布 ok=true 成功报告 ——
    let failed_checks = joint_checks(&JointContext {
        run_token: &run_token,
        scenario,
        run_id: &run_id,
        root_canon: &prepared.root_canon,
        child_report: &child_report,
        protected_same: protected_before == protected_after,
        survivors: facts
            .get("survivor_snapshot")
            .and_then(|v| v.as_array())
            .map_or(&[][..], Vec::as_slice),
        supervision_failed: supervision_reason.is_some(),
    });
    match publish_if_all_checks_pass(sink, &prepared, &facts, &failed_checks) {
        Ok(true) => serde_json::Value::Object(facts),
        Ok(false) => {
            let detail = child_report
                .get("failures")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            panic!(
                "[A-HARNESS] 联合检查未全部通过，未发布成功报告：{failed_checks:?}；child failures={detail}"
            )
        }
        Err(e) => panic!("[A-HARNESS] 报告发布失败：{e}"),
    }
}
