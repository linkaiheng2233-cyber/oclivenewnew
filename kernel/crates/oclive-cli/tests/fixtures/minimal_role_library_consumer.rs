use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use minimal_consumer::contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome};
use minimal_consumer::{
    types, MinimalRoleMessageError, MinimalRoleMessageRequest, MinimalRoleProductExtensionStatus,
    OcliveKernel, OcliveKernelConfig, PreparedMinimalRole,
};

struct MemoryLlm(AtomicUsize);

#[async_trait]
impl LlmClient for MemoryLlm {
    async fn generate(&self, _: &str, _: &str) -> types::Result<String> {
        panic!("consumer must use checked generation")
    }

    async fn generate_tag(&self, _: &str, _: &str) -> types::Result<String> {
        panic!("basic text does not classify product extensions")
    }

    async fn generate_with_opts(
        &self,
        _: &str,
        prompt: &str,
        _: Option<&LlmGenerateOpts>,
    ) -> types::Result<LlmGenerateOutcome> {
        assert!(prompt.contains("Developer's minimal guide"));
        assert!(prompt.contains("Hello from the generated library"));
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(LlmGenerateOutcome {
            reply: "raw model reply".into(),
            prompt_eval_ms: None,
        })
    }
}

// Keep the rich API usable from the same generated public surface.
#[allow(dead_code)]
async fn old_entry(host: &OcliveKernel) -> minimal_consumer::KernelResult<()> {
    host.load_role("legacy-role").await?;
    host.process_message(&types::SendMessageRequest {
        role_id: "legacy-role".into(),
        user_message: "Hello".into(),
        ..Default::default()
    })
    .await?;
    Ok(())
}

#[test]
fn developer_converter_uses_generated_public_surface() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime")
        .block_on(async {
            let data = std::path::PathBuf::from(
                std::env::var_os("OCLIVE_LIBRARY_FIXTURE_DATA").expect("isolated data root"),
            );
            let role_root = data.join("fixture-role-root");
            std::fs::create_dir_all(&role_root).expect("fixture root");
            let llm = Arc::new(MemoryLlm(AtomicUsize::new(0)));
            let host = OcliveKernel::builder(OcliveKernelConfig::new(&data, role_root))
                .with_host_profile(
                    minimal_consumer::kernel::domain::host_profile::HostProfile::default(),
                )
                .with_llm_client(llm.clone())
                .build()
                .await
                .expect("production Host");
            let role = PreparedMinimalRole::new(
                "developer-owned-id",
                types::MinimalRoleDefinition {
                    persona_prompt: "Developer's minimal guide".into(),
                    visual_assets: vec!["custom:portrait".into()],
                },
                vec![b"synthetic portrait bytes".to_vec()],
            )
            .expect("developer conversion output");
            let mut request = MinimalRoleMessageRequest {
                user_message: "Hello from the generated library".into(),
                requirements: String::new(),
            };
            let response = host
                .process_minimal_message(&role, &request)
                .await
                .expect("basic turn");
            assert_eq!(response.role_id, "developer-owned-id");
            assert_eq!(response.reply, "raw model reply");
            assert_eq!(
                response.product_extensions,
                MinimalRoleProductExtensionStatus::Unavailable
            );
            assert_eq!(llm.0.load(Ordering::SeqCst), 1);
            request.requirements = "relationship extension".into();
            assert!(
                matches!(host.process_minimal_message(&role, &request).await,
                Err(MinimalRoleMessageError::Prompt(error))
                    if error.kind == types::BaseCallErrorKind::Unsupported)
            );
            assert_eq!(llm.0.load(Ordering::SeqCst), 1);
            host.shutdown().await;
        });
}
