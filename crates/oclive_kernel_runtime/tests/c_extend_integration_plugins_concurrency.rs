//! 集成补强：目录插件整链（与 P2 烟测互补的摘要场景）、多会话并发、远程 Prompt 降级、专家编译错误路径。
//!
//! 目录插件完整生命周期（禁用/再启用/卸载）见 `p2_directory_plugin_smoke.rs`。

use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::expert_models::compile_graph_to_llama_local_config;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::domain::scene_commands::switch_scene;
use oclive_kernel_runtime::infrastructure::directory_plugins::{
    directory_plugin_bootstrap_dto, DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
};
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc, RemoteRpcChannel,
};
use oclive_kernel_runtime::models::dto::{SendMessageRequest, SwitchSceneRequest};
use oclive_kernel_runtime::models::expert_models::{ExpertGraph, ExpertNode};
use oclive_kernel_runtime::models::{EventType, EvolutionBounds, Memory, PersonalityVector, Role};
use oclive_kernel_runtime::state::KernelAppState;
use serde_json::json;
use std::any::Any;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(feature = "default-prompt-providers")]
use oclive_kernel_core::prompt::PromptAssembler;
#[cfg(feature = "default-prompt-providers")]
use oclive_kernel_runtime::domain::prompt_assembler::default_prompt_slot_v1;
#[cfg(feature = "default-prompt-providers")]
use oclive_kernel_runtime::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
#[cfg(feature = "default-prompt-providers")]
use oclive_kernel_runtime::infrastructure::remote_plugin::{
    RemotePluginHttpConfig, RemotePromptAssemblerHttp,
};

fn workspace_shimeng_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles/shimeng")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(p) = to.parent() {
                fs::create_dir_all(p)?;
            }
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn roles_dir_with_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_shimeng_dir();
    assert!(src.join("manifest.json").is_file(), "need roles/shimeng");
    let tmp = tempfile::tempdir().expect("tempdir");
    let dest = tmp.path().join(role_dir_name);
    copy_dir_recursive(&src, &dest).expect("copy");
    let manifest_path = dest.join("manifest.json");
    let raw = fs::read_to_string(&manifest_path).expect("read manifest");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    v["id"] = serde_json::Value::String(role_dir_name.to_string());
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&v).expect("serialize"),
    )
    .expect("write manifest");
    tmp
}

fn stub_plugin_exe() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_oclive_test_dir_plugin")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_oclive_test_dir_plugin.exe"))
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_oclive_test_dir_plugin")
}

fn write_stub_plugin(roles_root: &Path, plugin_id: &str) {
    let app_data = roles_root.join(".oclive_directory_plugin_data");
    let root = app_data.join("plugins").join(plugin_id);
    fs::create_dir_all(&root).expect("plugin dir");
    let exe = stub_plugin_exe();
    let exe_s = exe.to_string_lossy().replace('\\', "\\\\");
    let manifest = format!(
        r#"{{
  "schema_version": 1,
  "id": "{id}",
  "version": "1.0.0",
  "process": {{
    "command": "{cmd}",
    "args": []
  }}
}}"#,
        id = plugin_id,
        cmd = exe_s
    );
    fs::write(root.join("manifest.json"), manifest).expect("write manifest");
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "c_ext_ok".into(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn directory_plugin_discover_rpc_teardown_full_chain() {
    let plugin_id = format!(
        "cext_{}",
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .filter(|c| *c != '-')
            .take(8)
            .collect::<String>()
    );
    let tmp = roles_dir_with_clone("c_ext_dir_role");
    let roles_root = tmp.path().to_path_buf();
    write_stub_plugin(&roles_root, &plugin_id);

    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root.clone())
        .await
        .expect("state");
    load_role(&state, "c_ext_dir_role", false)
        .await
        .expect("load");

    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    let dto = directory_plugin_bootstrap_dto(
        state.directory_plugins.as_ref(),
        Some("c_ext_dir_role".into()),
        DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
    );
    assert!(
        dto.plugin_ids.contains(&plugin_id),
        "bootstrap should list {}",
        plugin_id
    );

    let url = state
        .directory_plugins
        .ensure_rpc_url(&plugin_id)
        .expect("rpc url");
    let out = invoke_directory_plugin_rpc(
        url.as_str(),
        "ping",
        json!({ "chain": true }),
        RemoteRpcChannel::Plugin,
    )
    .await
    .expect("rpc");
    assert_eq!(out["p2_stub"], true);

    state.directory_plugins.clear_plugin_process(&plugin_id);
    let plugin_fs = roles_root
        .join(".oclive_directory_plugin_data")
        .join("plugins")
        .join(&plugin_id);
    if plugin_fs.exists() {
        fs::remove_dir_all(&plugin_fs).expect("rm plugin");
    }
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    assert!(!state
        .directory_plugins
        .plugin_roots
        .read()
        .contains_key(&plugin_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_sessions_process_message_and_scene_consistency() {
    let tmp = roles_dir_with_clone("c_ext_conc");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");
    load_role(&state, "c_ext_conc", false).await.expect("load");

    switch_scene(
        &state,
        &SwitchSceneRequest {
            role_id: "c_ext_conc".into(),
            scene_id: "school".into(),
            together: true,
        },
    )
    .await
    .expect("switch");

    let st = Arc::new(state);
    let a = st.clone();
    let b = st.clone();
    let req_a = SendMessageRequest {
        role_id: "c_ext_conc".into(),
        user_message: "sess a".into(),
        scene_id: Some("school".into()),
        session_id: Some("conc_a".into()),
    };
    let req_b = SendMessageRequest {
        role_id: "c_ext_conc".into(),
        user_message: "sess b".into(),
        scene_id: Some("school".into()),
        session_id: Some("conc_b".into()),
    };
    let (ra, rb) = tokio::join!(
        process_message(a.as_ref(), &req_a),
        process_message(b.as_ref(), &req_b),
    );
    let oa = ra.expect("a");
    let ob = rb.expect("b");
    assert_eq!(oa.scene_id, "school");
    assert_eq!(ob.scene_id, "school");
}

#[test]
fn expert_compile_empty_graph_yields_no_model_path_without_error() {
    let tmp = tempfile::tempdir().unwrap();
    let gguf = tmp.path().join("models").join("gguf");
    let loras = tmp.path().join("models").join("loras");
    fs::create_dir_all(&gguf).unwrap();
    fs::create_dir_all(&loras).unwrap();
    let graph = ExpertGraph {
        version: 1,
        nodes: vec![],
        edges: vec![],
    };
    let out = compile_graph_to_llama_local_config(&graph, gguf.as_path(), loras.as_path())
        .expect("compile");
    assert!(out.model_path.is_none());
    assert!(out.llama_args.is_none());
}

#[test]
fn expert_compile_rejects_base_model_not_under_gguf_root() {
    let tmp = tempfile::tempdir().unwrap();
    let gguf = tmp.path().join("models").join("gguf");
    let loras = tmp.path().join("models").join("loras");
    fs::create_dir_all(&gguf).unwrap();
    fs::create_dir_all(&loras).unwrap();
    let rogue = tmp.path().join("outside.gguf");
    fs::write(&rogue, b"x").unwrap();
    let graph = ExpertGraph {
        version: 1,
        nodes: vec![ExpertNode::BaseModel {
            id: "b".into(),
            gguf_path: rogue.to_string_lossy().into_owned(),
            ui: None,
        }],
        edges: vec![],
    };
    let e = compile_graph_to_llama_local_config(&graph, gguf.as_path(), loras.as_path())
        .expect_err("path");
    assert!(
        e.to_string().contains("gguf") || e.to_string().contains("under"),
        "{}",
        e
    );
}

fn spawn_one_shot_http(status_line: &str, body: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let status = status_line.to_string();
    let body = body.to_string();
    let h = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0u8; 24_576];
        let _ = stream.read(&mut buf);
        let resp = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            body.len(),
            body
        );
        let _ = stream.write_all(resp.as_bytes());
    });
    (format!("http://127.0.0.1:{}/rpc", port), h)
}

#[cfg(feature = "default-prompt-providers")]
#[test]
fn remote_prompt_build_prompt_falls_back_on_http_502() {
    let (url, h) = spawn_one_shot_http("502 Bad Gateway", r#"{"detail":"no"}"#);
    let remote = RemotePromptAssemblerHttp::new(RemotePluginHttpConfig {
        endpoint: url,
        timeout: Duration::from_secs(3),
        bearer_token: None,
    });
    let builtin = default_prompt_slot_v1();
    let role = Role {
        id: "t".into(),
        name: "T".into(),
        description: "".into(),
        version: "1".into(),
        author: "".into(),
        core_personality: ".".into(),
        default_personality: oclive_kernel_runtime::models::PersonalityDefaults {
            stubbornness: 0.5,
            clinginess: 0.5,
            sensitivity: 0.5,
            assertiveness: 0.5,
            forgiveness: 0.5,
            talkativeness: 0.5,
            warmth: 0.5,
        },
        evolution_bounds: EvolutionBounds::full_01(),
        user_relations: vec![],
        evolution_config: oclive_kernel_runtime::models::EvolutionConfig::default(),
        memory_config: None,
        default_relation: "friend".into(),
        ollama_model: None,
        identity_binding: oclive_kernel_runtime::models::role::IdentityBinding::default(),
        life_trajectory: None,
        life_schedule: None,
        remote_presence: None,
        autonomous_scene: None,
        interaction_mode: None,
        min_runtime_version: None,
        dev_only: false,
        plugin_backends: oclive_kernel_runtime::models::PluginBackends::default(),
        ui_config: oclive_kernel_runtime::models::UiConfig::default(),
        knowledge_index: None,
        author_pack: None,
        reply_quality_anchor: None,
        creator_message_to_downloader: None,
    };
    let personality = PersonalityVector::zero();
    let memories: Vec<Memory> = vec![];
    let input = PromptInput {
        role_any: &role as &dyn Any,
        role_prompt: role.prompt_slice(),
        personality: &personality,
        memories: &memories,
        user_input: "hi",
        user_emotion: "neutral",
        user_relation_id: "",
        relation_hint: "",
        relation_before: "Stranger",
        favorability_before: 50.0,
        relation_preview: "Stranger",
        favorability_preview: 50.0,
        event_type: &EventType::Ignore,
        impact_factor: 0.0,
        scene_label: "",
        scene_detail: "",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        reply_quality_anchor: effective_reply_quality_anchor(role.prompt_slice()),
        complex_emotion_hint: None,
    };
    let a = remote.build_prompt(&input);
    let b = builtin.build_prompt(&input);
    assert_eq!(a, b);
    let _ = h.join();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_jsonrpc_plugin_channel_timeout_surfaces_transport_kind() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let h = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0u8; 24_576];
        let _ = stream.read(&mut buf);
        thread::sleep(Duration::from_secs(20));
    });
    let url = format!("http://127.0.0.1:{}/rpc", port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(300))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let err = oclive_kernel_runtime::infrastructure::remote_plugin::remote_plugin_call_async(
        RemoteRpcChannel::Plugin,
        &client,
        &url,
        "memory.rank",
        json!({"memories":[],"user_query":"x","limit":1}),
        None,
    )
    .await
    .expect_err("timeout");
    let s = err.to_string();
    assert!(
        s.contains("remote_plugin") && s.contains("timeout"),
        "{}",
        s
    );
    let _ = h.join();
}
