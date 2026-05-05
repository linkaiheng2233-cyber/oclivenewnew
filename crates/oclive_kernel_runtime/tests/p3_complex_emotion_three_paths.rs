//! P3-3：复杂情感矩阵——异地心声（remote_life）与 Agent 早退路径的集成覆盖（共景见 `p2_session_expert_smoke`）。

use async_trait::async_trait;
use oclive_kernel_runtime::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::error::Result;
use oclive_kernel_runtime::infrastructure::llm::{LlmClient, MockLlmClient};
use oclive_kernel_runtime::models::dto::SendMessageRequest;
use oclive_kernel_runtime::state::KernelAppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
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

fn isolated_roles_shimeng_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_roles_dir().join("shimeng");
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

struct AllPromptsCapturingLlm {
    reply: String,
    prompts: Arc<Mutex<Vec<String>>>,
}

impl AllPromptsCapturingLlm {
    fn pair(reply: impl Into<String>) -> (Arc<dyn LlmClient>, Arc<Mutex<Vec<String>>>) {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let llm: Arc<dyn LlmClient> = Arc::new(Self {
            reply: reply.into(),
            prompts: prompts.clone(),
        });
        (llm, prompts)
    }
}

#[async_trait]
impl LlmClient for AllPromptsCapturingLlm {
    async fn generate(&self, _model: &str, prompt: &str) -> Result<String> {
        self.prompts.lock().await.push(prompt.to_string());
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }
}

/// 异地心声：`process_remote_life` 主 LLM prompt 含上一轮 `complex_emotion_hint`（【复杂情感复盘】块）。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complex_emotion_hint_in_remote_life_main_prompt() {
    let tmp = isolated_roles_shimeng_clone("p3ce_remote");
    let roles = tmp.path().to_path_buf();
    let rid = "p3ce_remote";
    let srid = conversation_state_role_id(rid, None);
    let (llm, prompts) = AllPromptsCapturingLlm::pair("remote_ce_ok".to_string());
    let state = KernelAppState::new_in_memory_with_llm(llm, roles)
        .await
        .expect("state");

    load_role(&state, rid, false).await.expect("load_role");

    state
        .db_manager
        .set_remote_life_enabled(srid.as_str(), true)
        .await
        .expect("remote_life on");

    state
        .db_manager
        .set_current_scene(srid.as_str(), "default")
        .await
        .expect("char scene default");

    let req1 = SendMessageRequest {
        role_id: rid.into(),
        user_message: "都行".into(),
        scene_id: Some("default".into()),
        session_id: None,
    };
    process_message(&state, &req1)
        .await
        .expect("turn1 co_present");

    let hint = state
        .db_manager
        .get_complex_emotion_hint(srid.as_str())
        .await
        .expect("read hint")
        .expect("hint after builtin_keyword");
    assert!(!hint.trim().is_empty(), "expected narrative hint");

    state
        .db_manager
        .set_current_scene(srid.as_str(), "school")
        .await
        .expect("char moves to school");

    let req2 = SendMessageRequest {
        role_id: rid.into(),
        user_message: "那我们聊点别的".into(),
        scene_id: Some("default".into()),
        session_id: None,
    };
    process_message(&state, &req2)
        .await
        .expect("turn2 remote_life");

    let captured = prompts.lock().await;
    let with_block: Vec<&str> = captured
        .iter()
        .filter(|p| p.contains("【复杂情感复盘】"))
        .map(|s| s.as_str())
        .collect();
    assert!(
        with_block.iter().any(|p| p.contains(hint.trim())),
        "expected remote_life main prompt to carry prior hint; n_prompts={} hint_snip={}",
        captured.len(),
        hint.chars().take(120).collect::<String>()
    );
}

/// Agent 早退：`agent_out.handled` 分支内仍调用 `complex_emotion.resolve_turn` 并写入 hint（需 MCP 工具列表非空以进入 ReAct）。
#[cfg(feature = "kernel-agent")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complex_emotion_hint_persisted_on_agent_handled_early_exit() {
    let tmp = isolated_roles_shimeng_clone("p3ce_agent");
    let roles_root = tmp.path().to_path_buf();
    let rid = "p3ce_agent";
    let srid = conversation_state_role_id(rid, None);

    let app_data = roles_root.join(".oclive_directory_plugin_data");
    let mcp_root = app_data.join("mcp-servers");
    fs::create_dir_all(&mcp_root).expect("mcp dir");
    fs::write(
        mcp_root.join("p3_agent_ce.json"),
        r#"{"id":"p3_mcp_ce","name":"t","transport":"http","url":"http://127.0.0.1:9/unused","tools":[{"name":"dummy_tool"}]}"#,
    )
    .expect("write mcp manifest");

    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: "not_json_for_agent".into(),
    });
    let state = KernelAppState::new_in_memory_with_llm(llm, roles_root)
        .await
        .expect("state");

    load_role(&state, rid, false).await.expect("load_role");

    let req = SendMessageRequest {
        role_id: rid.into(),
        user_message: "都行".into(),
        scene_id: Some("default".into()),
        session_id: None,
    };
    let res = process_message(&state, &req).await.expect("agent path");
    assert!(
        res.reply.contains("调度工具") || res.reply.contains("工具"),
        "expected agent fallback reply, got {:?}",
        res.reply
    );

    let hint = state
        .db_manager
        .get_complex_emotion_hint(srid.as_str())
        .await
        .expect("read hint");
    let hint = hint.expect("hint after agent handled + builtin_keyword");
    assert!(
        !hint.trim().is_empty(),
        "unexpected empty hint after agent branch"
    );
}

/// 远程 Agent：首轮 `handled:true` 早退并写入复盘 hint；次轮 `handled:false` 走共景主 LLM，prompt 仍含【复杂情感复盘】与上一轮 hint。
#[cfg(feature = "kernel-agent")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complex_emotion_hint_in_co_present_prompt_after_remote_agent_early_exit() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    struct RemoteAgentUrlGuard {
        prev: Option<String>,
    }
    impl RemoteAgentUrlGuard {
        fn set(url: String) -> Self {
            let prev = std::env::var("OCLIVE_REMOTE_AGENT_URL").ok();
            std::env::set_var("OCLIVE_REMOTE_AGENT_URL", url);
            Self { prev }
        }
    }
    impl Drop for RemoteAgentUrlGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(s) => std::env::set_var("OCLIVE_REMOTE_AGENT_URL", s),
                None => std::env::remove_var("OCLIVE_REMOTE_AGENT_URL"),
            }
        }
    }

    fn patch_settings_agent_remote(role_dir: &Path) {
        let p = role_dir.join("settings.json");
        let raw = fs::read_to_string(&p).expect("read settings");
        let mut v: serde_json::Value = serde_json::from_str(&raw).expect("settings json");
        let obj = v.as_object_mut().expect("settings object");
        let mut pb = obj
            .get("plugin_backends")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        pb.as_object_mut()
            .expect("plugin_backends")
            .insert("agent".into(), serde_json::json!("remote"));
        obj.insert("plugin_backends".into(), pb);
        fs::write(&p, serde_json::to_string_pretty(&v).expect("ser")).expect("write settings");
    }

    let (tx, rx) = mpsc::channel::<std::net::SocketAddr>();
    let call_no = Arc::new(AtomicUsize::new(0));
    let cn = call_no.clone();
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock agent");
        tx.send(listener.local_addr().expect("addr"))
            .expect("send addr");
        for _ in 0..2 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut buf = Vec::<u8>::new();
            let mut chunk = [0u8; 2048];
            while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = match stream.read(&mut chunk) {
                    Ok(n) => n,
                    Err(_) => break,
                };
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
            }
            let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
                continue;
            };
            let hdr_end = pos + 4;
            let headers = std::str::from_utf8(&buf[..hdr_end]).unwrap_or("");
            let mut clen = None::<usize>;
            for line in headers.split("\r\n") {
                let l = line.to_ascii_lowercase();
                if l.starts_with("content-length:") {
                    clen = line.split(':').nth(1).and_then(|s| s.trim().parse().ok());
                }
            }
            let mut body = buf[hdr_end..].to_vec();
            if let Some(n) = clen {
                while body.len() < n {
                    let r = match stream.read(&mut chunk) {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    if r == 0 {
                        break;
                    }
                    body.extend_from_slice(&chunk[..r]);
                }
                body.truncate(n);
            }
            let Ok(v) = serde_json::from_slice::<serde_json::Value>(&body) else {
                continue;
            };
            let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let c = cn.fetch_add(1, Ordering::SeqCst);
            let result = if c == 0 {
                serde_json::json!({"handled": true, "reply": "remote_agent_early"})
            } else {
                serde_json::json!({"handled": false, "reply": ""})
            };
            let resp_body =
                serde_json::json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string();
            let res = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                resp_body.len(),
                resp_body
            );
            let _ = stream.write_all(res.as_bytes());
        }
    });

    let addr = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("mock agent addr");
    let url = format!("http://127.0.0.1:{}/", addr.port());
    let _url_guard = RemoteAgentUrlGuard::set(url);

    let tmp = isolated_roles_shimeng_clone("p3ce_ragent");
    patch_settings_agent_remote(&tmp.path().join("p3ce_ragent"));
    let roles = tmp.path().to_path_buf();
    let rid = "p3ce_ragent";
    let srid = conversation_state_role_id(rid, None);

    let (llm, prompts) = AllPromptsCapturingLlm::pair("co_present_after_agent".to_string());
    let state = KernelAppState::new_in_memory_with_llm_plugin_grants_and_policy_file(
        llm,
        roles,
        None,
        &[("system:remote_agent_http", "network:*")],
    )
    .await
    .expect("state");

    load_role(&state, rid, false).await.expect("load_role");

    process_message(
        &state,
        &SendMessageRequest {
            role_id: rid.into(),
            user_message: "都行".into(),
            scene_id: Some("default".into()),
            session_id: None,
        },
    )
    .await
    .expect("turn1 agent handled");

    let hint = state
        .db_manager
        .get_complex_emotion_hint(srid.as_str())
        .await
        .expect("read hint")
        .expect("hint after agent+keyword");
    assert!(!hint.trim().is_empty());

    process_message(
        &state,
        &SendMessageRequest {
            role_id: rid.into(),
            user_message: "我们继续随便聊聊".into(),
            scene_id: Some("default".into()),
            session_id: None,
        },
    )
    .await
    .expect("turn2 co_present llm");

    let captured = prompts.lock().await;
    let hits: Vec<&String> = captured
        .iter()
        .filter(|p| p.contains("【复杂情感复盘】") && p.contains(hint.trim()))
        .collect();
    assert!(
        !hits.is_empty(),
        "expected co_present main prompt to inject prior complex_emotion hint; n={} hint_prefix={:?}",
        captured.len(),
        hint.chars().take(80).collect::<String>()
    );
}
