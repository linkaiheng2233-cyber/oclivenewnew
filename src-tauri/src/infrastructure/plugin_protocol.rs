//! OCLIVE plugin asset protocol helpers (MIME, URI parsing, HTML bridge injection).
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::must_use_candidate)]

use crate::infrastructure::directory_plugins::OclivePluginManifest;
use std::path::Path;

/// 在 HTML 中注入 `window.OclivePluginBridge`；manifest 含 `bridge` 且资产路径匹配时启用。
pub fn inject_plugin_bridge_script(
    html: &str,
    plugin_id: &str,
    asset_rel: &str,
    manifest: &OclivePluginManifest,
) -> String {
    if !manifest.should_inject_bridge(asset_rel) {
        return html.to_string();
    }
    let Some(b) = manifest.bridge_for_asset_rel(asset_rel) else {
        return html.to_string();
    };
    let inv = serde_json::to_string(&b.invoke).unwrap_or_else(|_| "[]".to_string());
    let ev = serde_json::to_string(&b.events).unwrap_or_else(|_| "[]".to_string());
    let pid = serde_json::to_string(plugin_id).expect("serialize plugin_id");
    let arel = serde_json::to_string(asset_rel).expect("serialize asset_rel");
    static BRIDGE_CORE: &str = include_str!("../../assets/plugin-bridge.iife.js");
    let script = format!(
        "<script>{core}window.__oclivSetupPluginBridge({pid},{arel},{inv},{ev});</script>",
        core = BRIDGE_CORE,
        pid = pid,
        arel = arel,
        inv = inv,
        ev = ev
    );
    let lower = html.to_ascii_lowercase();
    if let Some(idx) = lower.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + script.len());
        out.push_str(&html[..idx]);
        out.push_str(&script);
        out.push_str(&html[idx..]);
        out
    } else {
        format!("{html}{script}")
    }
}

#[must_use]
pub fn mime_for_plugin_asset(rel: &str) -> &'static str {
    let ext = Path::new(rel)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[must_use]
pub fn plugin_asset_from_request_uri(uri: &str) -> Option<(String, String)> {
    let lower = uri.to_ascii_lowercase();
    let marker = "ocliveplugin.localhost/";
    let idx = lower.find(marker)?;
    let after = uri.get(idx + marker.len()..)?;
    let path_only = after.split(['?', '#']).next()?;
    let mut parts = path_only.split('/').filter(|s| !s.is_empty());
    let plugin_id = parts.next()?.to_string();
    let rest: Vec<&str> = parts.collect();
    if rest.contains(&"..") {
        return None;
    }
    let rel = rest.join("/");
    if rel.is_empty() {
        return None;
    }
    Some((plugin_id, rel))
}
