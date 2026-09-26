//! A-P2-R1 隔离回合 harness · 共享常量、身份校验、**路径先验接缝**（测试面新增，不改生产面）。
//!
//! 依据：主控《A-P2-R1 集中返修与复验任务书》§3–§5。四个 I/O 入口全部 `#[ignore]` 显式 opt-in；
//! 纯内存回归用例统一 `a_p2_r1_` 前缀。
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

pub mod artifacts;
pub mod driver;
pub mod env;
pub mod fixture;
pub mod recording_llm;

use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// 宿主 LLM 端口（= `oclive_kernel_contracts::LlmClient` 的同一 trait）。
pub use oclive_kernel_contracts::LlmClient as HostLlmClient;
/// 对外 DTO（公开契约，测试只读使用）。
pub use oclive_kernel_types::models::{SendMessageRequest, SendMessageResponse};

/// 唯一恢复区根（补充约束固定）。
pub const RECOVERY_BASE: &str = r"E:\OCLive\_recovery";
/// Legacy tree prefix. Every already consumed/observed tree lives under this prefix.
pub const TREE_PREFIX: &str = "CPB3V2-";
/// B4/V1 preparation family prefix. The controller reserved
/// `E:/OCLive/_recovery/CPB4V1-<run_id>/` for the future H02/H03 identities, so the tree name
/// must follow the run-id family instead of one global prefix.
pub const TREE_PREFIX_B4V1: &str = "CPB4V1-";

/// Tree prefix for a run id. Defaults to the legacy prefix so every existing tree keeps its
/// exact path; only the reserved `A-CPINTB4-…` family maps to the B4/V1 prefix.
#[must_use]
pub fn tree_prefix_for(run_id: &str) -> &'static str {
    if run_id.starts_with("A-CPINTB4-") {
        TREE_PREFIX_B4V1
    } else {
        TREE_PREFIX
    }
}

pub const ROLE_ID: &str = "a-probe-role";
pub const ROLE_NAME: &str = "A Probe Role";
pub const MODEL_ID: &str = "a-probe-model:latest";
pub const SCENE_ID: &str = "default";

pub const SCENARIO_TIMEOUT_SECS: u64 = 120;
pub const SHUTDOWN_TIMEOUT_SECS: u64 = 30;

pub const S1: &str = "s1";
pub const S2: &str = "s2";
pub const S3: &str = "s3";
pub const B1: &str = "b1";
pub const ENV_B_APPROVAL: &str = "OCLIVE_B_APPROVAL";
pub const B_APPROVAL: &str = "CPB3-B-qwen2.5-7b-845dbda0";
pub const B_TIMEOUT_SECS: u64 = 240;

// —— 合成输入常量（必须独立完整常量）——
pub const S1_MSG: &str = "随便聊聊吧，你最近有没有什么开心的小事？";
pub const S1_REPLY: &str = "我明白了，这就来。";
pub const S2_MSG: &str = "换个话题，说说你喜欢的季节吧。";
pub const S2_EMPTY_REPLY: &str = "";
pub const S2_REPLY: &str = "我最喜欢初秋的傍晚，风里有桂花的味道。";
pub const S3_MSG: &str = "我想听你讲一句关于清晨的话。";
pub const S3_ERR_MARKER: &str = "A-HARNESS-SYNTHETIC-ERR-3";
/// 修复提示词的必要段落标记（源码字面量，`post.rs:461`）。
pub const REPAIR_PROMPT_MARKER: &str = "【上一候选被拒原因】";
/// 空候选的拒收原因（源码字面量，`post.rs:167`）。
pub const EMPTY_REPLY_REJECTION: &str = "没有可显示台词";
/// S3 期望的 reason 字段（独立常量，不由被测格式函数生成）。
pub const S3_REASON_CODE: &str = "LLM_ERROR";
pub const S3_REASON_MESSAGE: &str = "Ollama error: A-HARNESS-SYNTHETIC-ERR-3";

pub const ENV_RUN_ID: &str = "OCLIVE_A_RUN_ID";
pub const ENV_CHILD: &str = "OCLIVE_A_CHILD";
pub const ENV_SCENARIO: &str = "OCLIVE_A_SCENARIO";
pub const ENV_ROOT: &str = "OCLIVE_A_ROOT";
pub const ENV_RUN_TOKEN: &str = "OCLIVE_A_RUN_TOKEN";

pub const CHILD_TEST_NAME: &str = "a_child_run_turn";

#[must_use]
pub fn user_message(scenario: &str) -> &'static str {
    if let Some(case) = crate::emotion_turn::find(scenario) {
        return case.message;
    }
    match scenario {
        crate::memory_turn::M1 => crate::memory_turn::HIT_QUERY,
        crate::memory_turn::M2 => crate::memory_turn::MISS_QUERY,
        S1 => S1_MSG,
        S2 => S2_MSG,
        S3 => S3_MSG,
        other => panic!("[A-HARNESS] unknown scenario: {other}"),
    }
}

#[must_use]
pub fn expected_reply(scenario: &str) -> Option<&'static str> {
    if crate::emotion_turn::find(scenario).is_some() {
        return Some(crate::emotion_turn::REPLY);
    }
    match scenario {
        crate::memory_turn::M1 | crate::memory_turn::M2 => Some(crate::memory_turn::REPLY),
        S1 => Some(S1_REPLY),
        S2 => Some(S2_REPLY),
        _ => None,
    }
}

#[must_use]
pub fn scripted_main_calls(scenario: &str) -> usize {
    if crate::emotion_turn::find(scenario).is_some() {
        return 1;
    }
    match scenario {
        crate::memory_turn::M1 | crate::memory_turn::M2 => 1,
        S1 => 1,
        S2 => 2,
        S3 => 1,
        other => panic!("[A-HARNESS] unknown scenario: {other}"),
    }
}

#[must_use]
pub fn scenario_of_driver(driver_test: &str) -> &'static str {
    match driver_test {
        "i1_driver_concurrent_identity" => "i1",
        "i2_driver_disconnect_recovery" => "i2",
        "i3_driver_conflict_and_new_send" => "i3",
        "i4_driver_durable_recovery" => "i4",
        "h1_driver_http_stream_success" => "h1",
        "h2_driver_http_stream_error" => "h2",
        "h3_driver_http_partial_error" => "h3",
        "h4_driver_http_disconnect_retry" => "h4",
        "e1_driver_joy_emotion_turn" => "e1",
        "e2_driver_no_clue_emotion_turn" => "e2",
        "e3_driver_negated_emotion_turn" => "e3",
        "e4_driver_neutral_clue_emotion_turn" => "e4",
        "m1_driver_selected_memory_turn" => crate::memory_turn::M1,
        "m2_driver_no_hit_memory_turn" => crate::memory_turn::M2,
        "a1_driver_normal_turn" => S1,
        "a2_driver_empty_reply_repair" => S2,
        "a3_driver_main_llm_err_fallback" => S3,
        "b1_driver_live_turn" => B1,
        other => panic!("[A-HARNESS] unknown driver test: {other}"),
    }
}

/// 本次批准的 run_id（父与子都必须具备；缺标记不得创建运行树/DB 或 spawn）。
#[must_use]
pub fn approved_run_id() -> String {
    let raw = std::env::var(ENV_RUN_ID).unwrap_or_default();
    let t = raw.trim().to_string();
    let shape_ok = (t.starts_with("A-") || t.starts_with("B-"))
        && (12..=64).contains(&t.len())
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    assert!(
        shape_ok,
        "[A-HARNESS] 缺少或非法的批准 run_id（{ENV_RUN_ID}，长度 {}）：缺授权标记不得创建运行树、DB 或 spawn。",
        t.len()
    );
    t
}

/// Mode approval is checked before any directory creation, and again in the child.
pub fn validate_run_mode(run_id: &str, scenario: &str, approval: &str) -> Result<(), String> {
    if crate::http_idempotency::is_scenario(scenario) {
        return crate::http_idempotency::validate_mode(run_id, scenario, approval);
    }
    if crate::http_entry::is_scenario(scenario) {
        return crate::http_entry::validate_mode(run_id, scenario, approval);
    }
    if crate::emotion_turn::find(scenario).is_some() {
        return crate::emotion_turn::validate_mode(run_id, scenario, approval);
    }
    match scenario {
        crate::memory_turn::M1 | crate::memory_turn::M2 => {
            crate::memory_turn::validate_mode(run_id, scenario, approval)
        }
        B1 => crate::semantic_cases::select(run_id, approval).map(|_| ()),
        S1 | S2 | S3 if run_id.starts_with("A-") => Ok(()),
        _ => Err("scenario/run_id/explicit live approval mismatch".into()),
    }
}

#[must_use]
pub fn tree_root(run_id: &str) -> PathBuf {
    PathBuf::from(RECOVERY_BASE).join(format!("{}{run_id}", tree_prefix_for(run_id)))
}

#[must_use]
pub fn scenario_root(run_id: &str, scenario: &str) -> PathBuf {
    tree_root(run_id).join("run").join(scenario)
}

#[must_use]
pub fn now_utc_stamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ————————————————————————————————————————————————————————————
// 路径先验接缝（R1）：任何创建/写入/spawn/build 之前，先对**已存在**路径组件逐个判定
// （绝对性、目录、非重解析、metadata 可读）；新组件在已验证父下逐级创建后再核验。
// 只针对既有检查失败与错范围；不声称抗并发替换（无 TOCTOU 保证）。
// ————————————————————————————————————————————————————————————

/// `FILE_ATTRIBUTE_REPARSE_POINT`（覆盖 symlink 与 junction 等重解析点）。
pub const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaFacts {
    pub is_dir: bool,
    pub is_file: bool,
    pub is_reparse: bool,
    pub bytes: u64,
}

impl MetaFacts {
    #[must_use]
    pub fn from_std(md: &std::fs::Metadata) -> Self {
        #[cfg(windows)]
        let is_reparse = {
            use std::os::windows::fs::MetadataExt;
            md.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        };
        #[cfg(not(windows))]
        let is_reparse = md.file_type().is_symlink();
        Self {
            is_dir: md.is_dir(),
            is_file: md.is_file(),
            is_reparse,
            bytes: md.len(),
        }
    }
}

/// 路径判定的最小只读接缝（真实实现 + 纯内存替身）。
pub trait FsProbe {
    fn symlink_metadata(&self, path: &Path) -> std::io::Result<MetaFacts>;
    fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf>;
    fn create_dir(&self, path: &Path) -> std::io::Result<()>;
}

#[derive(Debug, Default)]
pub struct RealFs;

impl FsProbe for RealFs {
    fn symlink_metadata(&self, path: &Path) -> std::io::Result<MetaFacts> {
        std::fs::symlink_metadata(path).map(|md| MetaFacts::from_std(&md))
    }

    fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }

    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir(path)
    }
}

/// 操作台账（拒绝分支必须 create == 0 且 spawn == 0）。
#[derive(Debug, Default)]
pub struct OpsLedger {
    pub creates: AtomicUsize,
    pub spawns: AtomicUsize,
}

impl OpsLedger {
    #[must_use]
    pub fn creates(&self) -> usize {
        self.creates.load(Ordering::SeqCst)
    }

    #[must_use]
    pub fn spawns(&self) -> usize {
        self.spawns.load(Ordering::SeqCst)
    }
}

/// 逐个校验 `p` 的**全部已存在组件**：缺失/非目录/重解析/metadata 失败一律 Err。
pub fn verify_existing_chain(fs: &dyn FsProbe, p: &Path, label: &str) -> Result<(), String> {
    let mut cur = PathBuf::new();
    for comp in p.components() {
        cur.push(comp.as_os_str());
        if matches!(comp, Component::Prefix(_) | Component::RootDir) {
            continue;
        }
        match fs.symlink_metadata(&cur) {
            Ok(m) => {
                if m.is_reparse {
                    return Err(format!(
                        "{label} 组件为重解析点（symlink/junction 等）：{}",
                        cur.display()
                    ));
                }
                if !m.is_dir {
                    return Err(format!("{label} 组件不是目录：{}", cur.display()));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(format!(
                    "{label} 组件不存在（须先建再核验）：{}",
                    cur.display()
                ));
            }
            Err(e) => {
                return Err(format!(
                    "{label} 组件 metadata 读取失败（状态不明，关闭）：{}: {e}",
                    cur.display()
                ));
            }
        }
    }
    Ok(())
}

/// 校验恢复区根自身及其全部已存在祖先（任何创建/写入之前必须调用）。
pub fn verify_recovery_base(fs: &dyn FsProbe) -> Result<PathBuf, String> {
    let base = PathBuf::from(RECOVERY_BASE);
    if !base.is_absolute() {
        return Err(format!("恢复区根不是绝对路径：{}", base.display()));
    }
    verify_existing_chain(fs, &base, "recovery_base")?;
    fs.canonicalize(&base).map_err(|e| {
        format!(
            "恢复区根无法规范化（状态不明，关闭）：{}: {e}",
            base.display()
        )
    })
}

/// 校验一个**已存在**目录：词法范围 + 规范化范围 + 全组件 + 非重解析。
pub fn verify_existing_dir_under(
    fs: &dyn FsProbe,
    base: &Path,
    target: &Path,
    label: &str,
) -> Result<PathBuf, String> {
    if !target.is_absolute() {
        return Err(format!("{label} 必须是绝对路径：{}", target.display()));
    }
    if !target.starts_with(base) {
        return Err(format!(
            "{label} 越出恢复区（词法范围）：{} !⊂ {}",
            target.display(),
            base.display()
        ));
    }
    verify_existing_chain(fs, target, label)?;
    let base_can = fs
        .canonicalize(base)
        .map_err(|e| format!("{label}: 恢复区根无法规范化：{e}"))?;
    let t_can = fs
        .canonicalize(target)
        .map_err(|e| format!("{label} 无法规范化：{}: {e}", target.display()))?;
    if !t_can.starts_with(&base_can) {
        return Err(format!(
            "{label} 规范化后越出恢复区：{} !⊂ {}",
            t_can.display(),
            base_can.display()
        ));
    }
    Ok(t_can)
}

/// 在**已验证父**下新建一个目录并立即核验；已存在则拒绝复用。
pub fn create_new_dir_verified(
    fs: &dyn FsProbe,
    parent: &Path,
    child: &Path,
    label: &str,
    ledger: &OpsLedger,
) -> Result<PathBuf, String> {
    verify_existing_chain(fs, parent, &format!("{label}.parent"))?;
    ledger.creates.fetch_add(1, Ordering::SeqCst);
    fs.create_dir(child).map_err(|e| {
        format!(
            "{label} 不可新建（已存在则拒绝复用）：{}: {e}",
            child.display()
        )
    })?;
    let m = fs
        .symlink_metadata(child)
        .map_err(|e| format!("{label} 新建后 metadata 读取失败：{}: {e}", child.display()))?;
    if m.is_reparse {
        return Err(format!("{label} 新建后为重解析点：{}", child.display()));
    }
    if !m.is_dir {
        return Err(format!("{label} 新建后不是目录：{}", child.display()));
    }
    Ok(child.to_path_buf())
}

/// 纯内存路径替身（不创建真实 junction、不写批准目录之外、不扫描用户目录）。
#[derive(Debug, Default)]
pub struct FakeFs {
    dirs: Mutex<Vec<PathBuf>>,
    reparse: Mutex<Vec<PathBuf>>,
    meta_errors: Mutex<Vec<PathBuf>>,
    canon_map: Mutex<Vec<(PathBuf, PathBuf)>>,
    created: Mutex<Vec<PathBuf>>,
}

impl FakeFs {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dir(&self, p: &Path) {
        self.dirs.lock().unwrap().push(p.to_path_buf());
    }

    /// 标记为重解析点（同时视为存在）。
    pub fn add_reparse(&self, p: &Path) {
        self.reparse.lock().unwrap().push(p.to_path_buf());
        self.add_dir(p);
    }

    pub fn add_meta_error(&self, p: &Path) {
        self.meta_errors.lock().unwrap().push(p.to_path_buf());
    }

    pub fn add_canon(&self, from: &Path, to: &Path) {
        self.canon_map
            .lock()
            .unwrap()
            .push((from.to_path_buf(), to.to_path_buf()));
    }

    #[must_use]
    pub fn created_dirs(&self) -> Vec<PathBuf> {
        self.created.lock().unwrap().clone()
    }

    /// 精确匹配（目录存在性：祖先存在不代表子路径存在）。
    fn matches_exact(list: &Mutex<Vec<PathBuf>>, p: &Path) -> bool {
        list.lock().unwrap().iter().any(|q| q.as_path() == p)
    }

    /// 前缀匹配（重解析/metadata 失败按祖先传播到后代）。
    fn matches(list: &Mutex<Vec<PathBuf>>, p: &Path) -> bool {
        list.lock()
            .unwrap()
            .iter()
            .any(|q| q.as_path() == p || p.starts_with(q))
    }
}

impl FsProbe for FakeFs {
    fn symlink_metadata(&self, path: &Path) -> std::io::Result<MetaFacts> {
        if Self::matches(&self.meta_errors, path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "fake metadata error",
            ));
        }
        let is_reparse = Self::matches(&self.reparse, path);
        let exists = is_reparse || Self::matches_exact(&self.dirs, path);
        if !exists {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "fake not found",
            ));
        }
        Ok(MetaFacts {
            is_dir: true,
            is_file: false,
            is_reparse,
            bytes: 0,
        })
    }

    fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf> {
        for (from, to) in self.canon_map.lock().unwrap().iter() {
            if path == from {
                return Ok(to.clone());
            }
        }
        if Self::matches(&self.meta_errors, path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "fake canonicalize error",
            ));
        }
        if Self::matches_exact(&self.dirs, path) {
            return Ok(path.to_path_buf());
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "fake not found",
        ))
    }

    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        if Self::matches_exact(&self.dirs, path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "fake dir exists",
            ));
        }
        self.created.lock().unwrap().push(path.to_path_buf());
        self.add_dir(path);
        Ok(())
    }
}

/// 三态路径判定（R2 §2.3）：只有 `Absent` 允许在已验证父下创建；
/// 其它 `Err`（PermissionDenied 等状态不明）必须在任何 create/write/spawn 之前拒绝。
#[derive(Debug, Clone)]
pub enum PathState {
    PresentDir(MetaFacts),
    Absent,
    Error(String),
}

#[must_use]
pub fn probe_dir_state(fs: &dyn FsProbe, path: &Path) -> PathState {
    match fs.symlink_metadata(path) {
        Ok(m) => PathState::PresentDir(m),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PathState::Absent,
        Err(e) => PathState::Error(format!(
            "路径状态不明（非 NotFound，拒绝且不创建）：{}: {e}",
            path.display()
        )),
    }
}

/// 本次**全新且已验证**的报告目录（只有完整建树后才存在此值）。
#[derive(Debug, Clone)]
pub struct PreparedTree {
    pub tree: PathBuf,
    pub root: PathBuf,
    pub root_canon: PathBuf,
    pub reports_dir: PathBuf,
}
