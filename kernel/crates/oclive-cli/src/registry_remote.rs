//! Cloud registry REST client (`OCLIVE_REGISTRY_URL` + `~/.oclive/auth.json`).

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

pub fn registry_base_url() -> Result<String> {
    if let Some(u) = crate::config::resolve("OCLIVE_REGISTRY_URL", None) {
        return Ok(u.trim().trim_end_matches('/').to_string());
    }
    if let Some(a) = load_auth()? {
        return Ok(a.registry_url.trim_end_matches('/').to_string());
    }
    bail!(
        "Cloud registry not configured: `oclive config set OCLIVE_REGISTRY_URL <url> --global` \
         and `oclive config set OCLIVE_REGISTRY_TOKEN <token> --global`"
    )
}

fn bearer_token() -> Result<String> {
    if let Some(t) = crate::config::resolve("OCLIVE_REGISTRY_TOKEN", None) {
        return Ok(t);
    }
    load_auth()?
        .map(|a| a.token)
        .ok_or_else(|| anyhow::anyhow!("Not logged in: run oclive registry login <url> <token>"))
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
    _template: Option<String>,
}

#[derive(Deserialize)]
struct RemoteList {
    #[serde(default)]
    projects: Vec<RemoteProject>,
}

pub fn run_logout() -> Result<()> {
    let p = auth_path();
    if p.is_file() {
        fs::remove_file(&p).context("remove auth.json")?;
        println!("Logged out of cloud registry");
    } else {
        println!("(not logged in)");
    }
    Ok(())
}

pub fn run_push(args: RegistryPushArgs) -> Result<()> {
    let base = registry_base_url()?;
    let token = bearer_token()?;
    let root =
        crate::project_root::resolve_project_root_for_registry(&args.name, args.path.as_deref())?;
    let tmp = tempfile::tempdir()?;
    let archive = tmp
        .path()
        .join(format!("{}.oclive-template.tar.gz", args.name));
    publish_cmd::pack_template_tarball(&root, &archive)?;
    let bytes = fs::read(&archive)?;
    let url = format!("{base}/api/v1/projects/{}", url_encode(&args.name));
    let resp = crate::http_client::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/gzip")
        .send_bytes(&bytes)
        .map_err(|e| anyhow::anyhow!("push failed: {e}"))?;
    if !(200..300).contains(&resp.status()) {
        bail!(
            "push HTTP {} — {}",
            resp.status(),
            resp.into_string().unwrap_or_default()
        );
    }
    println!("✓ Pushed project {} → {}", args.name, base);
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
        bail!("Output directory already exists: {}", out.display());
    }
    fs::create_dir_all(&out)?;
    let url = format!(
        "{}/api/v1/projects/{}/archive",
        base,
        url_encode(&args.name)
    );
    let resp = crate::http_client::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| anyhow::anyhow!("pull failed: {e}"))?;
    if !(200..300).contains(&resp.status()) {
        bail!(
            "pull HTTP {} — {}",
            resp.status(),
            resp.into_string().unwrap_or_default()
        );
    }
    let tmp = tempfile::tempdir()?;
    let archive = tmp.path().join("pull.tar.gz");
    let mut reader = resp.into_reader();
    let mut file = fs::File::create(&archive)?;
    std::io::copy(&mut reader, &mut file)?;
    publish_cmd::extract_tar_gz(&archive, &out)?;
    register_project(&args.name, &out, None)?;
    println!("✓ Pulled and registered {} → {}", args.name, out.display());
    Ok(())
}

pub fn run_search(args: RegistrySearchArgs) -> Result<()> {
    let base = registry_base_url()?;
    let token = bearer_token()?;
    let url = format!("{}/api/v1/projects?q={}", base, url_encode(&args.keyword));
    let resp = crate::http_client::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| anyhow::anyhow!("search failed: {e}"))?;
    let body = resp.into_string().context("read body")?;
    if args.json {
        println!("{body}");
        return Ok(());
    }
    let list: RemoteList = serde_json::from_str(&body).unwrap_or(RemoteList { projects: vec![] });
    if list.projects.is_empty() {
        println!("(no matches)");
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
