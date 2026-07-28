//! OCLIVE plugin asset protocol helpers (MIME, URI parsing, HTML bridge injection).
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

use crate::infrastructure::directory_plugins::OclivePluginManifest;
use std::path::Path;

/// Build the platform-specific URL that Tauri maps to the `ocliveplugin` protocol.
///
/// Windows and Android map custom protocols onto HTTP(S) hostnames. The
/// desktop window enables `useHttpsScheme`, so those targets use HTTPS.
/// macOS, iOS and Linux retain the native custom scheme.
#[must_use]
pub fn plugin_asset_url(plugin_id: &str, asset_rel: &str) -> String {
    #[cfg(any(target_os = "windows", target_os = "android"))]
    {
        format!("https://ocliveplugin.localhost/{}/{}", plugin_id, asset_rel)
    }
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        format!("ocliveplugin://localhost/{}/{}", plugin_id, asset_rel)
    }
}

/// Inject `window.OclivePluginBridge` into the HTML; enabled when the manifest contains `bridge` and the asset path matches.
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
    // The embedded bridge selects the source-bound parent transport whenever it
    // runs in a sandboxed iframe. Top-level shells keep the direct transport
    // until their dedicated WebView isolation stage.
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
    if let Some(head_start) = lower.find("<head") {
        let Some(head_end_offset) = lower[head_start..].find('>') else {
            return format!("{script}{html}");
        };
        let idx = head_start + head_end_offset + 1;
        let mut out = String::with_capacity(html.len() + script.len());
        out.push_str(&html[..idx]);
        out.push_str(&script);
        out.push_str(&html[idx..]);
        out
    } else {
        format!("{script}{html}")
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
    // Wry maps custom protocols to HTTPS on Windows/Linux and converts the
    // request back to `ocliveplugin://localhost/...` before invoking Tauri's
    // protocol handler. Accept both exact representations, but never search
    // for the host marker inside an arbitrary URI.
    const PREFIXES: [&str; 3] = [
        "ocliveplugin://localhost/",
        "https://ocliveplugin.localhost/",
        "http://ocliveplugin.localhost/",
    ];
    let after = PREFIXES.iter().find_map(|prefix| {
        let candidate = uri.get(..prefix.len())?;
        candidate
            .eq_ignore_ascii_case(prefix)
            .then(|| uri.get(prefix.len()..))
            .flatten()
    })?;
    let path_only = after.split(['?', '#']).next()?;
    let mut parts = path_only.split('/').filter(|s| !s.is_empty());
    let plugin_id = parts.next()?.to_string();
    let rest: Vec<&str> = parts.collect();
    if plugin_id == "."
        || plugin_id == ".."
        || plugin_id.contains('\\')
        || rest
            .iter()
            .any(|segment| *segment == "." || *segment == ".." || segment.contains('\\'))
    {
        return None;
    }
    let rel = rest.join("/");
    if rel.is_empty() {
        return None;
    }
    Some((plugin_id, rel))
}

#[cfg(test)]
mod tests {
    use super::{inject_plugin_bridge_script, plugin_asset_from_request_uri, plugin_asset_url};
    use crate::infrastructure::directory_plugins::OclivePluginManifest;

    #[test]
    fn injects_bridge_before_plugin_scripts() {
        let manifest: OclivePluginManifest = serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "id": "com.example.plugin",
            "version": "1.0.0",
            "ui_slots": [{
                "slot": "chat_toolbar",
                "entry": "slots/toolbar.html",
                "bridge": { "invoke": ["plugin_rpc_invoke"] }
            }]
        }))
        .unwrap();
        let html = "<!doctype html><html><head><script src=\"plugin.js\"></script></head><body></body></html>";

        let injected = inject_plugin_bridge_script(
            html,
            "com.example.plugin",
            "slots/toolbar.html",
            &manifest,
        );

        let bridge = injected.find("__oclivSetupPluginBridge").unwrap();
        let plugin = injected.find("plugin.js").unwrap();
        assert!(
            bridge < plugin,
            "bridge must exist before plugin scripts execute"
        );
    }

    #[test]
    fn parses_wry_custom_protocol_uri() {
        assert_eq!(
            plugin_asset_from_request_uri(
                "ocliveplugin://localhost/com.oclive.voice.asr/ui/sidebar.html"
            ),
            Some((
                "com.oclive.voice.asr".to_string(),
                "ui/sidebar.html".to_string()
            ))
        );
    }

    #[test]
    fn emits_platform_mapped_plugin_asset_url() {
        let url = plugin_asset_url("com.example.plugin", "ui/index.html");
        #[cfg(any(target_os = "windows", target_os = "android"))]
        assert_eq!(
            url,
            "https://ocliveplugin.localhost/com.example.plugin/ui/index.html"
        );
        #[cfg(not(any(target_os = "windows", target_os = "android")))]
        assert_eq!(
            url,
            "ocliveplugin://localhost/com.example.plugin/ui/index.html"
        );
    }

    #[test]
    fn parses_mapped_http_uri_case_insensitively() {
        assert_eq!(
            plugin_asset_from_request_uri(
                "HTTPS://OCLIVEPLUGIN.LOCALHOST/com.oclive.voice.asr/ui/asr.html?slot=voice#root"
            ),
            Some((
                "com.oclive.voice.asr".to_string(),
                "ui/asr.html".to_string()
            ))
        );
    }

    #[test]
    fn rejects_marker_in_untrusted_uri_and_path_traversal() {
        assert_eq!(
            plugin_asset_from_request_uri(
                "https://example.invalid/?next=https://ocliveplugin.localhost/p/ui.html"
            ),
            None
        );
        assert_eq!(
            plugin_asset_from_request_uri("ocliveplugin://localhost/p/../secret.txt"),
            None
        );
        assert_eq!(
            plugin_asset_from_request_uri("ocliveplugin://localhost/p"),
            None
        );
    }
}
