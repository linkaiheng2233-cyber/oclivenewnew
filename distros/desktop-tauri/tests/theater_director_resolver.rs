//! Theater director resolver + directory plugin prompt wiring (moved out of domain for layering ratchet).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use oclive_kernel_contracts::{TheaterDirectorBackendKind, TheaterPromptBuildInput};
use oclive_kernel_host::domain::host_profile::{HostProfile, TheaterProfile};
use oclive_kernel_host::domain::theater_director::{
    build_theater_prompt, resolve_effective_theater_director_config,
};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppStateBuilder;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

struct TestHarness {
    _app_data: TempDir,
    state: oclive_kernel_host::state::AppState,
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    fs::create_dir_all(dst).unwrap();
    for ent in fs::read_dir(src).unwrap() {
        let ent = ent.unwrap();
        let to = dst.join(ent.file_name());
        if ent.file_type().unwrap().is_dir() {
            copy_dir_all(&ent.path(), &to);
        } else {
            fs::copy(ent.path(), to).unwrap();
        }
    }
}

/// Self-contained RPC server for tests (minimal example imports official prompts via repo-relative path).
fn write_test_theater_director_plugin(plugin_dst: &std::path::Path) {
    fs::create_dir_all(plugin_dst).unwrap();
    fs::write(
        plugin_dst.join("manifest.json"),
        r#"{
  "schema_version": 1,
  "id": "theater-director-test",
  "name": "Theater Director Test Fixture",
  "version": "0.0.0",
  "provides": ["theater_director"],
  "rpcMethods": ["theater.build_prompt"],
  "process": { "command": "node", "args": ["rpc_server.mjs"] },
  "permissions": ["process:spawn"]
}"#,
    )
    .unwrap();
    fs::write(
        plugin_dst.join("rpc_server.mjs"),
        r#"import http from "node:http";
const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";
const STYLE_PREFIX = "[test-td] ";

function jsonRpcResult(id, result) {
  return JSON.stringify({ jsonrpc: "2.0", id, result });
}
function jsonRpcError(id, code, message) {
  return JSON.stringify({ jsonrpc: "2.0", id, error: { code, message } });
}

const server = http.createServer((req, res) => {
  if (req.method !== "POST" || !req.url || !req.url.startsWith("/rpc")) {
    res.writeHead(404);
    res.end("not found");
    return;
  }
  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", () => {
    let msg;
    try {
      msg = JSON.parse(Buffer.concat(chunks).toString("utf8"));
    } catch {
      res.writeHead(400, { "Content-Type": "application/json; charset=utf-8" });
      res.end(jsonRpcError(null, -32700, "parse error"));
      return;
    }
    const id = msg.id ?? null;
    res.setHeader("Content-Type", "application/json; charset=utf-8");
    res.setHeader(PROTOCOL_HEADER, PROTOCOL_VALUE);
    if (msg.method === "theater.build_prompt") {
      const mode = msg.params?.mode ?? "ripple";
      res.writeHead(200);
      res.end(jsonRpcResult(id, { prompt: `${STYLE_PREFIX}mode=${mode}` }));
      return;
    }
    res.writeHead(200);
    res.end(jsonRpcError(id, -32601, "method not found"));
  });
});
server.listen(0, "127.0.0.1", () => {
  const addr = server.address();
  const port = typeof addr === "object" && addr ? addr.port : 0;
  process.stdout.write(`OCLIVE_READY http://127.0.0.1:${port}/rpc\n`);
});
process.on("SIGTERM", () => server.close());
process.on("SIGINT", () => server.close());
"#,
    )
    .unwrap();
}

async fn test_state_with_profile(profile: HostProfile) -> TestHarness {
    let app_data = tempfile::tempdir().expect("app data");
    let llm = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let roles = common::roles_dir();
    let state = AppStateBuilder::in_memory_test(llm, roles, None)
        .with_app_data_dir(app_data.path())
        .with_host_profile(profile)
        .build()
        .await
        .expect("state");
    TestHarness {
        _app_data: app_data,
        state,
    }
}

async fn state_with_test_director_plugin() -> TestHarness {
    let app_data = tempfile::tempdir().expect("app data");
    let plugin_dst = app_data.path().join("plugin-src");
    write_test_theater_director_plugin(&plugin_dst);

    let profile = HostProfile {
        theater: TheaterProfile {
            director_plugin: Some("theater-director-test".to_string()),
        },
        ..HostProfile::default()
    };
    let llm = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let roles = common::roles_dir();
    let state = AppStateBuilder::in_memory_test(llm, roles, None)
        .with_app_data_dir(app_data.path())
        .with_host_profile(profile)
        .build()
        .await
        .expect("state");
    let app_plugins = state.directory_plugins.app_data_dir().join("plugins");
    fs::create_dir_all(&app_plugins).unwrap();
    copy_dir_all(&plugin_dst, &app_plugins.join("theater-director-test"));
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    TestHarness {
        _app_data: app_data,
        state,
    }
}

#[tokio::test]
async fn resolve_theater_director_defaults_builtin_without_plugin() {
    let harness = test_state_with_profile(HostProfile::default()).await;
    let eff = resolve_effective_theater_director_config(&harness.state);
    assert_eq!(eff.backend, TheaterDirectorBackendKind::Builtin);
    assert!(eff.directory_plugin_id.is_empty());
}

#[tokio::test]
async fn resolve_theater_director_directory_when_plugin_present() {
    let harness = state_with_test_director_plugin().await;
    let eff = resolve_effective_theater_director_config(&harness.state);
    assert_eq!(eff.backend, TheaterDirectorBackendKind::Directory);
    assert_eq!(eff.directory_plugin_id, "theater-director-test");
}

#[tokio::test(flavor = "multi_thread")]
async fn build_theater_prompt_uses_directory_plugin_prefix() {
    let harness = state_with_test_director_plugin().await;

    let input = TheaterPromptBuildInput {
        mode: "ripple".to_string(),
        persona_a: "傲娇".to_string(),
        persona_b: "温柔".to_string(),
        cast_a_name: "木木".to_string(),
        cast_b_name: "枫侵月".to_string(),
        cast_a_role_id: "mumu".to_string(),
        cast_b_role_id: "枫侵月".to_string(),
        scene_id: "home".to_string(),
        max_beats: 8,
        ripple_prefix_beats: Some(vec![]),
        ripple_skeleton: Some(vec![]),
        ripple_full_rewrite: Some(true),
        ..Default::default()
    };
    let prompt = build_theater_prompt(&harness.state, &input);
    assert!(
        prompt.starts_with("[test-td]"),
        "expected directory plugin prefix, got: {}",
        &prompt[..prompt.len().min(120)]
    );
}
