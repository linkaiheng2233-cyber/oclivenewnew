//! A separate test process owns these process-wide environment variables.

use async_trait::async_trait;
use oclive_kernel_host::domain::ports::AppSettingsPort;
use oclive_kernel_host::domain::user_llm_env::{
    apply_user_llm_env_from_db, KEY_LLM_PROVIDER, KEY_OLLAMA_BASE,
};
use oclive_kernel_host::error::Result;
use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Barrier, Mutex};
use tokio::time::{sleep, timeout, Duration};

const ENV_KEYS: &[&str] = &[
    "OLLAMA_BASE_URL",
    "OCLIVE_REMOTE_LLM_URL",
    "OCLIVE_REMOTE_LLM_TOKEN",
    "OCLIVE_LLM_CLOUD_API_STYLE",
    "OCLIVE_LOCAL_LLM_MODEL_PATH",
    "OCLIVE_LOCAL_LLM_LORA_PATH",
    "OCLIVE_LLM_BACKEND",
];

struct RestoreEnvironment(Vec<(&'static str, Option<OsString>)>);

impl RestoreEnvironment {
    fn capture() -> Self {
        Self(
            ENV_KEYS
                .iter()
                .map(|&key| (key, std::env::var_os(key)))
                .collect(),
        )
    }
}

impl Drop for RestoreEnvironment {
    fn drop(&mut self) {
        for (key, value) in &self.0 {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

struct GatedSettings {
    base_url: Mutex<String>,
    reads: AtomicUsize,
    first_read: mpsc::UnboundedSender<()>,
    release_first: Mutex<Option<oneshot::Receiver<()>>>,
}

#[async_trait]
impl AppSettingsPort for GatedSettings {
    async fn get_app_setting(&self, _key: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn get_app_settings(&self, _keys: &[&str]) -> Result<HashMap<String, String>> {
        let base_url = self.base_url.lock().await.clone();
        if self.reads.fetch_add(1, Ordering::AcqRel) == 0 {
            self.first_read
                .send(())
                .expect("first read observer closed");
            let receiver = self
                .release_first
                .lock()
                .await
                .take()
                .expect("first read gate missing");
            receiver.await.expect("first read gate dropped");
        }
        Ok(HashMap::from([
            (KEY_LLM_PROVIDER.to_owned(), "local".to_owned()),
            (KEY_OLLAMA_BASE.to_owned(), base_url),
        ]))
    }

    async fn upsert_app_setting(&self, _key: &str, _value: &str) -> Result<()> {
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn old_db_snapshot_cannot_overwrite_newer_concurrent_apply() {
    let _environment = RestoreEnvironment::capture();
    let (first_read_tx, mut first_read_rx) = mpsc::unbounded_channel();
    let (release_tx, release_rx) = oneshot::channel();
    let settings = Arc::new(GatedSettings {
        base_url: Mutex::new("http://127.0.0.1:11111".to_owned()),
        reads: AtomicUsize::new(0),
        first_read: first_read_tx,
        release_first: Mutex::new(Some(release_rx)),
    });

    let old = {
        let settings = Arc::clone(&settings);
        tokio::spawn(async move { apply_user_llm_env_from_db(settings.as_ref()).await })
    };
    timeout(Duration::from_secs(2), first_read_rx.recv())
        .await
        .expect("old read did not start")
        .expect("old read observer closed");

    *settings.base_url.lock().await = "http://127.0.0.1:22222".to_owned();
    const WAITERS: usize = 32;
    let launch = Arc::new(Barrier::new(WAITERS + 1));
    let (attempting_tx, mut attempting_rx) = mpsc::unbounded_channel();
    let waiting = (0..WAITERS)
        .map(|_| {
            let settings = Arc::clone(&settings);
            let launch = Arc::clone(&launch);
            let attempting = attempting_tx.clone();
            tokio::spawn(async move {
                launch.wait().await;
                attempting.send(()).expect("attempt observer closed");
                apply_user_llm_env_from_db(settings.as_ref()).await
            })
        })
        .collect::<Vec<_>>();
    launch.wait().await;
    for _ in 0..WAITERS {
        timeout(Duration::from_secs(2), attempting_rx.recv())
            .await
            .expect("new caller did not start")
            .expect("attempt observer closed");
    }
    sleep(Duration::from_millis(100)).await;
    assert_eq!(
        settings.reads.load(Ordering::Acquire),
        1,
        "a newer caller read DB while the old transaction was still blocked"
    );

    release_tx.send(()).expect("old read task exited early");
    assert_eq!(
        old.await
            .expect("old task panicked")
            .expect("old apply failed"),
        "local"
    );
    for task in waiting {
        assert_eq!(
            task.await
                .expect("new task panicked")
                .expect("new apply failed"),
            "local"
        );
    }
    assert_eq!(settings.reads.load(Ordering::Acquire), WAITERS + 1);
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").expect("base URL missing"),
        "http://127.0.0.1:22222"
    );
    assert_eq!(std::env::var("OCLIVE_LLM_BACKEND").as_deref(), Ok("ollama"));
}
