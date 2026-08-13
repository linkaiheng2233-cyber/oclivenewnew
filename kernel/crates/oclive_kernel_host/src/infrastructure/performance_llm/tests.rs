use crate::domain::host_profile::LocalLlmRuntimeProfile;
use crate::domain::ports::LlmClient;
use crate::domain::resource_coordinator::ResourceCoordinator;
use crate::error::{AppError, Result};
#[cfg(test)]
use crate::infrastructure::performance_request_gate::REQUEST_BLOCKED_RESOURCE_TRANSITION;
use crate::infrastructure::resource_adapters::{
    llama_tier, COSYVOICE_ADAPTER_ID, LLAMA_RUNTIME_ADAPTER_ID,
};
use async_trait::async_trait;
use oclive_kernel_types::{ResourceCoordinatorPolicy, ResourcePriority};
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;
use crate::infrastructure::MockLlmClient;
use axum::{
    body::{Body, Bytes},
    response::Response,
    routing::get,
    routing::post,
    Router,
};
use futures_util::StreamExt;
use oclive_kernel_contracts::ResourceSnapshotSource;
use oclive_kernel_types::{CpuSnapshot, GpuDeviceSnapshot, ResourceSnapshot, SystemMemorySnapshot};
use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use tempfile::tempdir;

fn profile(endpoint: String) -> LocalLlmRuntimeProfile {
    LocalLlmRuntimeProfile {
        mode: crate::domain::host_profile::LocalLlmRuntimeMode::Performance,
        endpoint,
        auto_start: false,
        startup_timeout_ms: 1_000,
        retry_cooldown_ms: 1_000,
        model_alias: "test-performance".into(),
        performance_profile: "gpu_balanced".into(),
    }
}

struct FixedResourceSnapshot(ResourceSnapshot);

#[async_trait]
impl ResourceSnapshotSource for FixedResourceSnapshot {
    async fn snapshot(&self) -> ResourceSnapshot {
        self.0.clone()
    }
}

#[test]
fn managed_runtime_passes_selected_lora_to_llama_server() {
    let selection = RuntimeSelection {
        model_path: PathBuf::from("base.gguf"),
        adapter_path: Some(PathBuf::from("adapter.gguf")),
    };
    let mut command = Command::new("llama-server");
    append_runtime_selection_args(&mut command, &selection);
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(args, ["-m", "base.gguf", "--lora", "adapter.gguf"]);
}

#[test]
fn stale_runtime_signature_requires_every_owned_argument() {
    let binary = PathBuf::from(r"D:\OCLive\components\llama.cpp\llama-server.exe");
    let selection = RuntimeSelection {
        model_path: PathBuf::from(r"D:\OCLive\models\qwen.gguf"),
        adapter_path: Some(PathBuf::from(r"D:\OCLive\models\mumu-adapter.gguf")),
    };
    let tier = llama_tier("gpu_balanced").expect("balanced tier");
    let command_line = format!(
        r#""D:\OCLive\components\llama.cpp\llama-server.exe" -m "D:\OCLive\models\qwen.gguf" --lora "D:\OCLive\models\mumu-adapter.gguf" --host 127.0.0.1 --port 8421 --alias oclive-performance --n-gpu-layers {}"#,
        tier.gpu_layers
    );

    assert!(command_line_matches_managed_runtime(
        &command_line,
        &binary,
        &selection,
        "oclive-performance",
        8421,
        tier,
    ));
    assert!(!command_line_matches_managed_runtime(
        &command_line.replace("--alias oclive-performance", "--alias user-server"),
        &binary,
        &selection,
        "oclive-performance",
        8421,
        tier,
    ));
    assert!(!command_line_matches_managed_runtime(
        &command_line.replace("mumu-adapter.gguf", "other-adapter.gguf"),
        &binary,
        &selection,
        "oclive-performance",
        8421,
        tier,
    ));
    assert!(!command_line_matches_managed_runtime(
        &command_line.replace("--port 8421", "--port 8422"),
        &binary,
        &selection,
        "oclive-performance",
        8421,
        tier,
    ));
    assert!(!command_line_matches_managed_runtime(
        &command_line.replace("--port 8421", "--port 84210"),
        &binary,
        &selection,
        "oclive-performance",
        8421,
        tier,
    ));
    assert!(!command_line_matches_managed_runtime(
        &command_line.replace(
            "--alias oclive-performance",
            "--alias oclive-performance-user",
        ),
        &binary,
        &selection,
        "oclive-performance",
        8421,
        tier,
    ));
}

#[tokio::test]
async fn managed_runtime_admission_falls_from_balanced_gpu_to_cpu() {
    let dir = tempdir().unwrap();
    let model_path = dir.path().join("test-model.gguf");
    std::fs::write(&model_path, vec![0_u8; 4 * 1024 * 1024]).unwrap();
    let coordinator = Arc::new(ResourceCoordinator::new(
        ResourceCoordinatorPolicy::default(),
        Arc::new(FixedResourceSnapshot(ResourceSnapshot {
            captured_at_ms: 1,
            source: "test".into(),
            available: true,
            gpu_devices: vec![GpuDeviceSnapshot {
                device_index: 0,
                name: "constrained".into(),
                total_mib: 8192,
                free_mib: 1000,
                used_mib: 7192,
            }],
            system_memory: Some(SystemMemorySnapshot {
                total_mib: 16_384,
                available_mib: 4_000,
                used_mib: 12_384,
            }),
            cpu: Some(CpuSnapshot {
                logical_cores: 8,
                physical_cores: Some(4),
            }),
            reason_codes: Vec::new(),
        })),
    ));
    let client = PerformanceLlmClient::new_with_resource_coordinator(
        profile("http://127.0.0.1:9".into()),
        dir.path().to_path_buf(),
        None,
        Arc::new(MockLlmClient {
            reply: "fallback".into(),
        }),
        None,
        "fallback-model".into(),
        Arc::clone(&coordinator),
    )
    .unwrap();
    client.set_active_runtime_tier(llama_tier("gpu_balanced").unwrap());
    let lease_id = client
        .reserve_runtime_start(&RuntimeSelection {
            model_path,
            adapter_path: None,
        })
        .await
        .unwrap();

    assert_eq!(client.active_runtime_tier().profile_id, "cpu_compatibility");
    let lease = coordinator
        .diagnostics_snapshot()
        .leases
        .into_iter()
        .find(|lease| lease.lease_id == lease_id)
        .unwrap();
    assert_eq!(lease.profile_id.as_deref(), Some("cpu_compatibility"));
    assert_eq!(lease.reservation_mib, 0);
    assert_eq!(lease.cpu_thread_reservation, 4);
    assert!(lease.ram_reservation_mib >= 512);
    client.release_runtime_lease();
}

#[tokio::test]
async fn missing_optional_runtime_falls_back_to_ollama_client() {
    let dir = tempdir().unwrap();
    let fallback: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: "fallback-ok".into(),
    });
    let client = PerformanceLlmClient::new(
        profile("http://127.0.0.1:9".into()),
        dir.path().to_path_buf(),
        None,
        fallback,
        None,
        "fallback-model".into(),
    )
    .unwrap();
    assert!(client
        .resource_coordinator
        .diagnostics_snapshot()
        .adapters
        .iter()
        .any(|adapter| adapter.descriptor.adapter_id == LLAMA_RUNTIME_ADAPTER_ID));
    assert_eq!(
        client.generate("fallback-model", "hello").await.unwrap(),
        "fallback-ok"
    );
    assert_eq!(client.status_snapshot().active_backend, "ollama");
}

#[tokio::test]
async fn unavailable_primary_and_fallback_report_both_causes() {
    let dir = tempdir().unwrap();
    let fallback: Arc<dyn LlmClient> = Arc::new(FailingFallback);
    let client = PerformanceLlmClient::new(
        profile("http://127.0.0.1:9".into()),
        dir.path().to_path_buf(),
        None,
        fallback,
        None,
        "fallback-model".into(),
    )
    .unwrap();

    let error = client
        .generate("fallback-model", "hello")
        .await
        .unwrap_err();
    let AppError::OllamaError(detail) = error else {
        panic!("expected LLM_ERROR-compatible OllamaError");
    };
    assert!(detail.contains("Performance LLM primary unavailable"));
    assert!(detail.contains("Ollama fallback unavailable"));
    assert!(detail.contains("fallback transport offline"));
}

#[tokio::test]
async fn suspended_runtime_cannot_be_restarted_by_a_queued_request() {
    let primary_calls = Arc::new(AtomicUsize::new(0));
    let primary_calls_for_route = Arc::clone(&primary_calls);
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/v1/chat/completions",
            post(move || {
                let calls = Arc::clone(&primary_calls_for_route);
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    r#"{"choices":[{"message":{"content":"primary"}}]}"#
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let fallback: Arc<dyn LlmClient> = Arc::new(CountingFallback {
        calls: Arc::clone(&fallback_calls),
    });
    let dir = tempdir().unwrap();
    let client = PerformanceLlmClient::new(
        profile(format!("http://{addr}")),
        dir.path().to_path_buf(),
        None,
        fallback,
        None,
        "fallback-model".into(),
    )
    .unwrap();
    client.suspend_managed_runtime("cloud provider is active");

    assert_eq!(
        client.generate("fallback-model", "hello").await.unwrap(),
        "fallback"
    );
    assert_eq!(primary_calls.load(Ordering::SeqCst), 0);
    assert_eq!(fallback_calls.load(Ordering::SeqCst), 1);
    assert_eq!(client.status_snapshot().active_backend, "inactive");
    server.abort();
}

#[tokio::test]
async fn resource_pressure_suspension_blocks_every_ollama_fallback_entrypoint() {
    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let fallback: Arc<dyn LlmClient> = Arc::new(CountingFallback {
        calls: Arc::clone(&fallback_calls),
    });
    let dir = tempdir().unwrap();
    let client = PerformanceLlmClient::new(
        profile("http://127.0.0.1:9".into()),
        dir.path().to_path_buf(),
        None,
        fallback,
        None,
        "fallback-model".into(),
    )
    .unwrap();
    client
        .suspend_managed_runtime_for_resource_pressure(
            "bundled voice runtime requires GPU headroom",
        )
        .await
        .unwrap();

    let errors = [
        client
            .generate("fallback-model", "hello")
            .await
            .unwrap_err(),
        client
            .generate_tag("fallback-model", "hello")
            .await
            .unwrap_err(),
        client
            .generate_with_opts("fallback-model", "hello", None)
            .await
            .unwrap_err(),
        client
            .generate_stream_with_opts("fallback-model", "hello", Arc::new(|_| {}), None)
            .await
            .unwrap_err(),
        client.startup_probe().await.unwrap_err(),
    ];
    assert!(errors.iter().all(|error| error
        .to_string()
        .contains(REQUEST_BLOCKED_RESOURCE_TRANSITION)));
    assert_eq!(fallback_calls.load(Ordering::SeqCst), 0);
    assert_eq!(client.status_snapshot().active_backend, "inactive");
    assert_eq!(client.inspect().await.active_backend, "inactive");
    assert!(client
        .suspend_managed_runtime_for_resource_pressure("duplicate suspension")
        .await
        .unwrap_err()
        .to_string()
        .contains("llm_resource_transition_already_active"));

    client.enable_managed_runtime();
    assert_eq!(
        client.generate("fallback-model", "hello").await.unwrap(),
        "fallback"
    );
    assert_eq!(fallback_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn runtime_recovery_refuses_to_overlap_retained_voice_residency() {
    let dir = tempdir().unwrap();
    let client = PerformanceLlmClient::new(
        profile("http://127.0.0.1:9".into()),
        dir.path().to_path_buf(),
        None,
        Arc::new(MockLlmClient {
            reply: "fallback".into(),
        }),
        None,
        "fallback-model".into(),
    )
    .unwrap();
    let voice_lease = client.resource_coordinator.begin_observed_activity(
        COSYVOICE_ADAPTER_ID,
        "retained-after-unconfirmed-release",
        None,
        ResourcePriority::ForegroundMedia,
    );

    assert!(client
        .apply_runtime_selection()
        .await
        .unwrap_err()
        .to_string()
        .contains("llm_recovery_blocked_by_voice_residency"));

    client.resource_coordinator.release(&voice_lease);
    assert!(!client
        .resource_coordinator
        .has_adapter_lease(COSYVOICE_ADAPTER_ID));
}

#[test]
fn runtime_manifest_cannot_escape_component_root() {
    let dir = tempdir().unwrap();
    std::fs::write(
            dir.path().join(RUNTIME_PACK_MANIFEST),
            r#"{
                "schema_version": 1,
                "component_id": "com.oclive.runtime.llama-cpp",
                "component_type": "llm_runtime",
                "engine": "llama.cpp",
                "version": "1.0.0",
                "executable": "../llama-server.exe",
                "executable_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }"#,
        )
        .unwrap();
    assert!(PerformanceLlmClient::binary_from_manifest(dir.path()).is_none());
}

#[test]
fn runtime_manifest_accepts_hashed_binary_inside_component_root() {
    let dir = tempdir().unwrap();
    let bin_dir = dir.path().join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let binary = bin_dir.join("llama-server.test");
    std::fs::write(&binary, b"test-runtime").unwrap();
    let hash = format!("{:x}", Sha256::digest(b"test-runtime"));
    std::fs::write(
        dir.path().join(RUNTIME_PACK_MANIFEST),
        format!(
            r#"{{
                    "schema_version": 1,
                    "component_id": "com.oclive.runtime.llama-cpp",
                    "component_type": "llm_runtime",
                    "engine": "llama.cpp",
                    "version": "1.0.0",
                    "executable": "bin/llama-server.test",
                    "executable_sha256": "{hash}"
                }}"#
        ),
    )
    .unwrap();
    assert_eq!(
        PerformanceLlmClient::binary_from_manifest(dir.path()),
        Some(binary)
    );
}

struct CountingFallback {
    calls: Arc<AtomicUsize>,
}

struct FailingFallback;

#[async_trait]
impl LlmClient for FailingFallback {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(AppError::OllamaError("fallback transport offline".into()))
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        self.generate(model, prompt).await
    }
}

#[async_trait]
impl LlmClient for CountingFallback {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok("fallback".into())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok("fallback".into())
    }

    async fn startup_probe(&self) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct BlockingFallback {
    started: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Semaphore>,
}

struct BlockingThenCountingFallback {
    started: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Semaphore>,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LlmClient for BlockingThenCountingFallback {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            self.started.notify_one();
            let permit = self.release.acquire().await.map_err(|error| {
                AppError::RemoteServiceUnavailable(format!(
                    "test fallback release semaphore closed: {error}"
                ))
            })?;
            permit.forget();
        }
        Ok("fallback".into())
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        self.generate(model, prompt).await
    }
}

#[async_trait]
impl LlmClient for BlockingFallback {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        self.started.notify_one();
        let permit = self.release.acquire().await.map_err(|error| {
            AppError::RemoteServiceUnavailable(format!(
                "test fallback release semaphore closed: {error}"
            ))
        })?;
        permit.forget();
        Ok("fallback".into())
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        self.generate(model, prompt).await
    }
}

#[tokio::test]
async fn resource_pressure_suspension_waits_for_inflight_fallback_to_drain() {
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let fallback: Arc<dyn LlmClient> = Arc::new(BlockingFallback {
        started: Arc::clone(&started),
        release: Arc::clone(&release),
    });
    let dir = tempdir().unwrap();
    let client = Arc::new(
        PerformanceLlmClient::new(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap(),
    );

    let started_wait = started.notified();
    let request_client = Arc::clone(&client);
    let request =
        tokio::spawn(async move { request_client.generate("fallback-model", "hello").await });
    started_wait.await;

    let suspension_client = Arc::clone(&client);
    let mut suspension = tokio::spawn(async move {
        suspension_client
            .suspend_managed_runtime_for_resource_pressure(
                "bundled voice runtime requires GPU headroom",
            )
            .await
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(50), &mut suspension)
            .await
            .is_err(),
        "resource suspension must wait for the active fallback request"
    );

    release.add_permits(1);
    assert_eq!(request.await.unwrap().unwrap(), "fallback");
    suspension.await.unwrap().unwrap();
    assert!(client
        .generate("fallback-model", "blocked")
        .await
        .unwrap_err()
        .to_string()
        .contains(REQUEST_BLOCKED_RESOURCE_TRANSITION));
}

#[tokio::test]
async fn cancelled_suspension_reopens_requests_before_voice_ownership_transfer() {
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let fallback: Arc<dyn LlmClient> = Arc::new(BlockingThenCountingFallback {
        started: Arc::clone(&started),
        release: Arc::clone(&release),
        calls: Arc::clone(&fallback_calls),
    });
    let dir = tempdir().unwrap();
    let client = Arc::new(
        PerformanceLlmClient::new(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap(),
    );

    let started_wait = started.notified();
    let request_client = Arc::clone(&client);
    let request =
        tokio::spawn(async move { request_client.generate("fallback-model", "first").await });
    started_wait.await;

    let suspension_client = Arc::clone(&client);
    let suspension = tokio::spawn(async move {
        suspension_client
            .suspend_managed_runtime_for_resource_pressure(
                "bundled voice runtime requires GPU headroom",
            )
            .await
    });
    tokio::time::timeout(Duration::from_secs(1), async {
        while !client.resource_suspension_active() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    suspension.abort();
    assert!(suspension.await.unwrap_err().is_cancelled());
    assert!(!client.resource_suspension_active());

    release.add_permits(1);
    assert_eq!(request.await.unwrap().unwrap(), "fallback");
    assert_eq!(
        client.generate("fallback-model", "second").await.unwrap(),
        "fallback"
    );
    assert_eq!(fallback_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn resource_pressure_suspension_does_not_interrupt_inflight_primary_request() {
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let started_for_route = Arc::clone(&started);
    let release_for_route = Arc::clone(&release);
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/v1/chat/completions",
            post(move || {
                let started = Arc::clone(&started_for_route);
                let release = Arc::clone(&release_for_route);
                async move {
                    started.notify_one();
                    let permit = release.acquire().await.unwrap();
                    permit.forget();
                    r#"{"choices":[{"message":{"content":"primary"}}]}"#
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let dir = tempdir().unwrap();
    let client = Arc::new(
        PerformanceLlmClient::new(
            profile(format!("http://{addr}")),
            dir.path().to_path_buf(),
            None,
            Arc::new(MockLlmClient {
                reply: "fallback".into(),
            }),
            None,
            "fallback-model".into(),
        )
        .unwrap(),
    );

    let started_wait = started.notified();
    let request_client = Arc::clone(&client);
    let request =
        tokio::spawn(async move { request_client.generate("fallback-model", "hello").await });
    started_wait.await;

    let suspension_client = Arc::clone(&client);
    let mut suspension = tokio::spawn(async move {
        suspension_client
            .suspend_managed_runtime_for_resource_pressure(
                "bundled voice runtime requires GPU headroom",
            )
            .await
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(50), &mut suspension)
            .await
            .is_err(),
        "resource suspension must wait for the active primary request"
    );

    release.add_permits(1);
    assert_eq!(request.await.unwrap().unwrap(), "primary");
    suspension.await.unwrap().unwrap();
    assert!(client
        .generate("fallback-model", "blocked")
        .await
        .unwrap_err()
        .to_string()
        .contains(REQUEST_BLOCKED_RESOURCE_TRANSITION));
    server.abort();
}

#[tokio::test]
async fn explicit_recovery_supersedes_a_draining_resource_suspension_without_opening_early() {
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let fallback: Arc<dyn LlmClient> = Arc::new(BlockingFallback {
        started: Arc::clone(&started),
        release: Arc::clone(&release),
    });
    let dir = tempdir().unwrap();
    let client = Arc::new(
        PerformanceLlmClient::new(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap(),
    );

    let started_wait = started.notified();
    let request_client = Arc::clone(&client);
    let request =
        tokio::spawn(async move { request_client.generate("fallback-model", "hello").await });
    started_wait.await;

    let suspension_client = Arc::clone(&client);
    let suspension = tokio::spawn(async move {
        suspension_client
            .suspend_managed_runtime_for_resource_pressure(
                "bundled voice runtime requires GPU headroom",
            )
            .await
    });
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if client.status_snapshot().active_backend == "inactive" {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    client.enable_managed_runtime();
    assert!(
        client.request_gate.is_blocked(),
        "recovery must not reopen fallback while the suspension is still draining"
    );

    release.add_permits(1);
    assert_eq!(request.await.unwrap().unwrap(), "fallback");
    assert!(suspension
        .await
        .unwrap()
        .unwrap_err()
        .to_string()
        .contains("llm_resource_transition_superseded_by_recovery"));
    assert!(!client.request_gate.is_blocked());

    release.add_permits(1);
    assert_eq!(
        client
            .generate("fallback-model", "recovered")
            .await
            .unwrap(),
        "fallback"
    );
}

#[tokio::test]
async fn stream_failure_after_first_token_does_not_duplicate_via_fallback() {
    async fn broken_stream() -> Response {
        Response::builder()
            .header("content-type", "text/event-stream")
            .body(Body::from(
                "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\
                     data: {broken-json}\n",
            ))
            .unwrap()
    }
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/chat/completions", post(broken_stream));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let calls = Arc::new(AtomicUsize::new(0));
    let fallback: Arc<dyn LlmClient> = Arc::new(CountingFallback {
        calls: Arc::clone(&calls),
    });
    let dir = tempdir().unwrap();
    let client = PerformanceLlmClient::new(
        profile(format!("http://{addr}")),
        dir.path().to_path_buf(),
        None,
        fallback,
        None,
        "fallback-model".into(),
    )
    .unwrap();
    let emitted = Arc::new(Mutex::new(String::new()));
    let emitted_for_sink = Arc::clone(&emitted);
    let result = client
        .generate_stream(
            "fallback-model",
            "hello",
            Arc::new(move |token| emitted_for_sink.lock().push_str(token)),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(emitted.lock().as_str(), "partial");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    server.abort();
}

#[tokio::test]
async fn performance_stream_emits_first_token_before_generation_finishes() {
    async fn delayed_stream() -> Response {
        let chunks = [
            (
                Duration::from_millis(10),
                "data: {\"choices\":[{\"delta\":{\"content\":\"first\"}}]}\n\n",
            ),
            (
                Duration::from_millis(180),
                "data: {\"choices\":[{\"delta\":{\"content\":\"-second\"}}]}\n\n",
            ),
            (Duration::from_millis(10), "data: [DONE]\n\n"),
        ];
        let stream = futures_util::stream::iter(chunks).then(|(delay, chunk)| async move {
            tokio::time::sleep(delay).await;
            Ok::<Bytes, Infallible>(Bytes::from_static(chunk.as_bytes()))
        });
        Response::builder()
            .header("content-type", "text/event-stream")
            .body(Body::from_stream(stream))
            .unwrap()
    }
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/chat/completions", post(delayed_stream));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let fallback: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: "fallback".into(),
    });
    let dir = tempdir().unwrap();
    let client = PerformanceLlmClient::new(
        profile(format!("http://{addr}")),
        dir.path().to_path_buf(),
        None,
        fallback,
        None,
        "fallback-model".into(),
    )
    .unwrap();
    let started = Instant::now();
    let first_token_ms = Arc::new(AtomicU64::new(0));
    let first_token_for_sink = Arc::clone(&first_token_ms);
    let reply = client
        .generate_stream(
            "fallback-model",
            "hello",
            Arc::new(move |_| {
                let elapsed = started.elapsed().as_millis() as u64;
                let _ = first_token_for_sink.compare_exchange(
                    0,
                    elapsed.max(1),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }),
        )
        .await
        .unwrap();
    let total_ms = started.elapsed().as_millis() as u64;
    assert_eq!(reply, "first-second");
    assert!(first_token_ms.load(Ordering::SeqCst) + 100 < total_ms);
    server.abort();
}
