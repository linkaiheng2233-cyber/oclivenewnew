//! 云端注册表 REST 客户端（`OCLIVE_REGISTRY_URL` + `~/.oclive/auth.json`）。

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::publish_cmd;
use crate::registry::{oclive_home, register_project};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryAuth {
    pub registry_url: String,
    pub token: String,
}

pub fn auth_path() -> PathBuf {
    oclive_home().join("auth.json")
}

pub fn load_auth() -> Result<Option<RegistryAuth>> {
    let p = auth_path();
    if !p.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&p).context("read auth.json")?;
    Ok(Some(serde_json::from_str(&raw).context("parse auth.json")?))
}

pub fn save_auth(auth: &RegistryAuth) -> Result<()> {
    let p = auth_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&p, serde_json::to_string_pretty(auth)?).context("write auth.json")?;
    Ok(())
}

pub fn registry_base_url() -> Result<String> {
    if let Some(u) = crate::config::resolve("OCLIVE_REGISTRY_URL", None) {
        return Ok(u.trim().trim_end_matches('/').to_string());
    }
    if let Some(a) = load_auth()? {
        return Ok(a.registry_url.trim_end_matches('/').to_string());
    }
    bail!("未配置云端注册表：请 `oclive registry login <url> <token>` 或 `oclive config set OCLIVE_REGISTRY_URL <url>`")
}

fn bearer_token() -> Result<String> {
    if let Some(t) = crate::config::resolve("OCLIVE_REGISTRY_TOKEN", None) {
        return Ok(t);
    }
    load_auth()?
        .map(|a| a.token)
        .ok_or_else(|| anyhow::anyhow!("未登录：请 oclive registry login <url> <token>"))
}

#[derive(Parser, Debug)]
pub struct RegistryLoginArgs {
    pub url: String,
    pub token: String,
}

#[derive(Parser, Debug)]
pub struct RegistryPushArgs {
    pub name: String,
    #[arg(short = 'o', long)]
    pub path: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct RegistryPullArgs {
    pub name: String,
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct RegistrySearchArgs {
    pub keyword: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(Deserialize)]
struct RemoteProject {
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    template: Option<String>,
}

#[derive(Deserialize)]
struct RemoteList {
    #[serde(default)]
    projects: Vec<RemoteProject>,
}

pub fn run_login(args: RegistryLoginArgs) -> Result<()> {
    let url = args.url.trim().trim_end_matches('/').to_string();
    save_auth(&RegistryAuth {
        registry_url: url.clone(),
        token: args.token.trim().to_string(),
    })?;
    println!("已登录云端注册表: {}", url);
    println!("凭据: {}", auth_path().display());
    Ok(())
}

pub fn run_logout() -> Result<()> {
    let p = auth_path();
    if p.is_file() {
        fs::remove_file(&p).context("remove auth.json")?;
        println!("已登出云端注册表");
    } else {
        println!("（未登录）");
    }
    Ok(())
}

pub fn run_push(args: RegistryPushArgs) -> Result<()> {
    let base = registry_base_url()?;
    let token = bearer_token()?;
    let root = resolve_project_root(&args.name, args.path.as_deref())?;
    let tmp = tempfile::tempdir()?;
    let archive = tmp.path().join(format!("{}.oclive-template.tar.gz", args.name));
    publish_cmd::pack_template_tarball(&root, &archive)?;
    let bytes = fs::read(&archive)?;
    let url = format!("{base}/api/v1/projects/{}", url_encode(&args.name));
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/gzip")
        .send_bytes(&bytes)
        .map_err(|e| anyhow::anyhow!("push 失败: {e}"))?;
    if !(200..300).contains(&resp.status()) {
        bail!("push HTTP {} — {}", resp.status(), resp.into_string().unwrap_or_default());
    }
    println!("✓ 已推送工程 {} → {}", args.name, base);
    Ok(())
}

pub fn run_pull(args: RegistryPullArgs) -> Result<()> {
    let base = registry_base_url()?;
    let token = bearer_token()?;
    let out = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(&args.name));
    if out.exists() {
        bail!("输出目录已存在: {}", out.display());
    }
    fs::create_dir_all(&out)?;
    let url = format!(
        "{}/api/v1/projects/{}/archive",
        base,
        url_encode(&args.name)
    );
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| anyhow::anyhow!("pull 失败: {e}"))?;
    if !(200..300).contains(&resp.status()) {
        bail!("pull HTTP {} — {}", resp.status(), resp.into_string().unwrap_or_default());
    }
    let tmp = tempfile::tempdir()?;
    let archive = tmp.path().join("pull.tar.gz");
    let mut reader = resp.into_reader();
    let mut file = fs::File::create(&archive)?;
    std::io::copy(&mut reader, &mut file)?;
    publish_cmd::extract_tar_gz(&archive, &out)?;
    register_project(&args.name, &out, None)?;
    println!("✓ 已拉取并注册 {} → {}", args.name, out.display());
    Ok(())
}

pub fn run_search(args: RegistrySearchArgs) -> Result<()> {
    let base = registry_base_url()?;
    let token = bearer_token()?;
    let url = format!(
        "{}/api/v1/projects?q={}",
        base,
        url_encode(&args.keyword)
    );
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| anyhow::anyhow!("search 失败: {e}"))?;
    let body = resp.into_string().context("read body")?;
    if args.json {
        println!("{body}");
        return Ok(());
    }
    let list: RemoteList = serde_json::from_str(&body).unwrap_or(RemoteList {
        projects: vec![],
    });
    if list.projects.is_empty() {
        println!("（无匹配）");
        return Ok(());
    }
    for p in list.projects {
        println!(
            "{} v{} — {} — {}",
            p.name, p.version, p.author, p.description
        );
    }
    Ok(())
}

fn resolve_project_root(name: &str, path: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = path {
        return p.canonicalize().with_context(|| format!("path {}", p.display()));
    }
    let entry = crate::registry::find_entry(name)?
        .ok_or_else(|| anyhow::anyhow!("本地注册表无工程 {name}；请 registry add 或 -o 指定路径"))?;
    PathBuf::from(&entry.path)
        .canonicalize()
        .with_context(|| format!("registry path {}", entry.path))
}

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect()
}
