//! 报告与清单工具（R1.4）：全部 I/O 以真实 `Result` 呈现，**失败不得降级为空成功**。
//! 文件清单只覆盖**已枚举范围**（本运行树 + 选定源码保护面），不构成全机无副作用证明。
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::MetaFacts;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// 只读文件接缝（真实实现 + 纯内存替身；用于枚举/读取失败的回归）。
pub trait FsReader {
    fn read_dir(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>>;
    fn metadata(&self, path: &Path) -> std::io::Result<MetaFacts>;
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>>;
}

#[derive(Debug, Default)]
pub struct RealReader;

impl FsReader for RealReader {
    fn read_dir(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            out.push(entry?.path());
        }
        Ok(out)
    }

    fn metadata(&self, path: &Path) -> std::io::Result<MetaFacts> {
        std::fs::symlink_metadata(path).map(|md| MetaFacts::from_std(&md))
    }

    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }
}

/// # Errors 文件不可读时返回错误（绝不返回“像哈希的字符串”）。
pub fn sha256_file_with(reader: &dyn FsReader, path: &Path) -> Result<String, String> {
    let bytes = reader
        .read(path)
        .map_err(|e| format!("读取失败（哈希不可用）：{}: {e}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

/// # Errors 见 [`sha256_file_with`]。
pub fn sha256_file(path: &Path) -> Result<String, String> {
    sha256_file_with(&RealReader, path)
}

/// 父目录必须已由调用方预建并可通过本测试的目录验证规则（不再自行 `create_dir_all`）。
fn require_verified_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("写入路径无父目录：{}", path.display()))?;
    match std::fs::symlink_metadata(parent) {
        Ok(md) => {
            let facts = MetaFacts::from_std(&md);
            if facts.is_reparse {
                return Err(format!("父目录为重解析点，拒绝写入：{}", parent.display()));
            }
            if !facts.is_dir {
                return Err(format!("父路径不是目录，拒绝写入：{}", parent.display()));
            }
            Ok(())
        }
        Err(e) => Err(format!(
            "父目录不存在或状态不明（调用方须先预建并核验）：{}: {e}",
            parent.display()
        )),
    }
}

/// # Errors 序列化或写入失败。
pub fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    require_verified_parent(path)?;
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| format!("序列化报告失败 {}: {e}", path.display()))?;
    std::fs::write(path, text).map_err(|e| format!("写报告失败 {}: {e}", path.display()))
}

/// # Errors 序列化或写入失败。
pub fn append_jsonl(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    use std::io::Write;
    require_verified_parent(path)?;
    let line = serde_json::to_string(value)
        .map_err(|e| format!("序列化 JSONL 失败 {}: {e}", path.display()))?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开 JSONL 失败 {}: {e}", path.display()))?;
    writeln!(f, "{line}").map_err(|e| format!("写 JSONL 失败 {}: {e}", path.display()))
}

/// 仓库根（由编译期 `CARGO_MANIFEST_DIR` 推导，不硬编码机器路径）。
#[must_use]
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("[A-HARNESS] 无法从 CARGO_MANIFEST_DIR 推导仓库根")
        .to_path_buf()
}

/// 选定源码保护面（本轮运行前后必须逐字节不变）。
#[must_use]
pub fn protected_files() -> Vec<(String, PathBuf)> {
    let repo = repo_root();
    vec![
        (
            "slot_runner.rs".to_string(),
            repo.join("kernel/crates/oclive_kernel_host/src/domain/slot_runner.rs"),
        ),
        (
            "handoff/README.md".to_string(),
            repo.join("handoff/README.md"),
        ),
    ]
}

/// # Errors 任一保护面文件不可读/不可哈希 ⇒ Err（不得“两次同样失败也算相等”）。
pub fn protected_hashes() -> Result<serde_json::Value, String> {
    let mut m = serde_json::Map::new();
    for (label, path) in protected_files() {
        let sha = sha256_file(&path)?;
        let bytes = std::fs::metadata(&path)
            .map_err(|e| format!("保护面 metadata 失败 {}: {e}", path.display()))?
            .len();
        m.insert(
            label,
            serde_json::json!({
                "path": path.display().to_string(),
                "bytes": bytes,
                "sha256": sha,
            }),
        );
    }
    Ok(serde_json::Value::Object(m))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanState {
    /// 目标不存在（与“存在但为空”明确区分）。
    Absent,
    /// 目标存在，条目如下（可能为空）。
    Present,
}

#[derive(Debug, Clone)]
pub struct FileScan {
    pub state: ScanState,
    pub entries: Vec<serde_json::Value>,
}

impl FileScan {
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn relative_paths(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(|v| v.get("relative_path").and_then(|x| x.as_str()))
            .map(str::to_string)
            .collect()
    }
}

fn walk(
    reader: &dyn FsReader,
    dir: &Path,
    base: &Path,
    out: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    let entries = reader
        .read_dir(dir)
        .map_err(|e| format!("目录枚举失败（不得当作空目录）：{}: {e}", dir.display()))?;
    for path in entries {
        let meta = reader
            .metadata(&path)
            .map_err(|e| format!("条目 metadata 失败（状态不明）：{}: {e}", path.display()))?;
        if meta.is_reparse {
            return Err(format!(
                "枚举中发现重解析点（不继续递归未知目标）：{}",
                path.display()
            ));
        }
        if meta.is_dir {
            walk(reader, &path, base, out)?;
        } else if meta.is_file {
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .display()
                .to_string()
                .replace('\\', "/");
            let sha = sha256_file_with(reader, &path)?;
            out.push(serde_json::json!({
                "relative_path": rel,
                "bytes": meta.bytes,
                "sha256": sha,
            }));
        } else {
            return Err(format!("非文件非目录条目（状态不明）：{}", path.display()));
        }
    }
    Ok(())
}

/// # Errors 枚举/metadata/读取/哈希任一失败 ⇒ Err。
pub fn scan_files_with(reader: &dyn FsReader, root: &Path) -> Result<FileScan, String> {
    let root_meta = match reader.metadata(root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FileScan {
                state: ScanState::Absent,
                entries: Vec::new(),
            })
        }
        Err(e) => {
            return Err(format!(
                "根 metadata 失败（状态不明）：{}: {e}",
                root.display()
            ))
        }
        Ok(m) => m,
    };
    if root_meta.is_reparse {
        return Err(format!("根为重解析点：{}", root.display()));
    }
    if !root_meta.is_dir {
        return Err(format!("根不是目录：{}", root.display()));
    }
    let mut out = Vec::new();
    walk(reader, root, root, &mut out)?;
    out.sort_by(|a, b| {
        a.get("relative_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(
                b.get("relative_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            )
    });
    Ok(FileScan {
        state: ScanState::Present,
        entries: out,
    })
}

/// # Errors 见 [`scan_files_with`]。
pub fn scan_files(root: &Path) -> Result<FileScan, String> {
    scan_files_with(&RealReader, root)
}

/// # Errors 读取失败或任一行 JSON 解码失败 ⇒ Err（不得静默跳过）。
pub fn read_jsonl_with(
    reader: &dyn FsReader,
    path: &Path,
) -> Result<Vec<serde_json::Value>, String> {
    let bytes = reader
        .read(path)
        .map_err(|e| format!("JSONL 读取失败：{}: {e}", path.display()))?;
    let text =
        String::from_utf8(bytes).map_err(|e| format!("JSONL 非 UTF-8：{}: {e}", path.display()))?;
    let mut out = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).map_err(|e| {
            format!(
                "JSONL 第 {} 行解码失败（不得当作空成功）：{}: {e}",
                idx + 1,
                path.display()
            )
        })?;
        out.push(v);
    }
    Ok(out)
}

/// # Errors 见 [`read_jsonl_with`]。
pub fn read_jsonl(path: &Path) -> Result<Vec<serde_json::Value>, String> {
    read_jsonl_with(&RealReader, path)
}

/// 有限进程快照（**支持性证据**：可能漏掉已退出的短命进程，不构成全程无 spawn/网络证明）。
#[must_use]
pub fn process_snapshot_matching(needle: &str) -> Vec<serde_json::Value> {
    let sys = sysinfo::System::new_all();
    let mut hits = Vec::new();
    for (pid, entry) in sys.processes() {
        let exe = entry
            .exe()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let cmd = entry
            .cmd()
            .iter()
            .map(|c| c.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        if exe.contains(needle) || cmd.contains(needle) {
            hits.push(serde_json::json!({
                "pid": pid.as_u32(),
                "exe": exe,
                "cmd": cmd,
            }));
        }
    }
    hits
}
