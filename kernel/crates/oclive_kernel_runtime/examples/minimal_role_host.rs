//! A separate, bounded Host example for the portable minimal-role content.
//!
//! The Host chooses a technical ID, owns the validated local snapshot, selects a
//! Prompt Base implementation and an in-memory LLM, and decides when to call them.
//! It does not register a role in the reference `AppState` or define a mandatory
//! six-slot turn order. The immediate poller is valid only for these local adapters.

use std::cell::Cell;
use std::path::Path;
use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{BaseCallFuture, LlmBase, PromptBase};
use oclive_kernel_runtime::domain::minimal_role_prompt::MinimalRolePrompt;
use oclive_kernel_types::{BaseCallErrorKind, LlmBaseRequest, PromptBaseRequest};
use oclive_validation::minimal_role_local_file::{
    load_minimal_role_local_file, LocalMinimalRoleSnapshot,
};

const MAX_DEFINITION_BYTES: usize = 64 * 1024;
const MAX_ASSET_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOTAL_ASSET_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
enum HostError {
    InvalidTechnicalId,
    UnknownRole,
    InvalidLocalFiles,
    InvalidRoleDefinition,
    Capability(BaseCallErrorKind),
    PendingCapability,
    UnexpectedOutput,
}

// The selected Prompt and Echo LLM are immediate. A production Host needs its own
// async executor, deadlines and resource policy; pending is a visible failure here.
fn poll_immediate<T>(mut future: BaseCallFuture<'_, T>) -> Result<T, HostError> {
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(Ok(value)) => Ok(value),
        Poll::Ready(Err(error)) => Err(HostError::Capability(error.kind)),
        Poll::Pending => Err(HostError::PendingCapability),
    }
}

#[derive(Default)]
struct EchoLlm {
    calls: Cell<usize>,
}

impl LlmBase for EchoLlm {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            self.calls.set(self.calls.get() + 1);
            Ok(format!("echo: {}", request.input))
        })
    }
}

struct MinimalHost<L> {
    technical_id: String,
    snapshot: LocalMinimalRoleSnapshot,
    llm: L,
}

impl<L: LlmBase> MinimalHost<L> {
    fn load(
        asset_root: &Path,
        definition_reference: &str,
        technical_id: &str,
        llm: L,
    ) -> Result<Self, HostError> {
        if technical_id.trim().is_empty() {
            return Err(HostError::InvalidTechnicalId);
        }
        let snapshot = load_minimal_role_local_file(
            asset_root,
            definition_reference,
            MAX_DEFINITION_BYTES,
            MAX_ASSET_BYTES,
            MAX_TOTAL_ASSET_BYTES,
        )
        .map_err(|_| HostError::InvalidLocalFiles)?;
        Ok(Self {
            technical_id: technical_id.to_owned(),
            snapshot,
            llm,
        })
    }

    fn reply(
        &self,
        requested_role_id: &str,
        user_text: &str,
        requirements: &str,
    ) -> Result<String, HostError> {
        if requested_role_id != self.technical_id {
            return Err(HostError::UnknownRole);
        }
        let prompt = MinimalRolePrompt::new(self.snapshot.definition())
            .map_err(|_| HostError::InvalidRoleDefinition)?;
        let prepared = poll_immediate(prompt.assemble(PromptBaseRequest {
            materials: &["User: ", user_text],
            requirements,
        }))?;
        poll_immediate(self.llm.generate(LlmBaseRequest { input: &prepared }))
    }

    fn visual_assets(&self) -> impl ExactSizeIterator<Item = (&str, &[u8])> {
        self.snapshot.assets()
    }
}

fn run_demo() -> Result<(), HostError> {
    let directory = tempfile::tempdir().map_err(|_| HostError::InvalidLocalFiles)?;
    std::fs::write(
        directory.path().join("content.json"),
        r#"{"persona_prompt":"A curious guide.","visual_assets":["portrait.bin"]}"#,
    )
    .map_err(|_| HostError::InvalidLocalFiles)?;
    std::fs::write(directory.path().join("portrait.bin"), b"synthetic-portrait")
        .map_err(|_| HostError::InvalidLocalFiles)?;

    let host = MinimalHost::load(
        directory.path(),
        "content.json",
        "host-owned-guide-id",
        EchoLlm::default(),
    )?;
    let answer = host.reply("host-owned-guide-id", "Hello.", "")?;
    if answer != "echo: 【角色设定】\nA curious guide.\n\n【输入材料】\nUser: Hello."
        || host.visual_assets().len() != 1
        || host.llm.calls.get() != 1
    {
        return Err(HostError::UnexpectedOutput);
    }
    Ok(())
}

fn main() {
    if let Err(error) = run_demo() {
        eprintln!("minimal role Host example failed: {error:?}");
        std::process::exit(1);
    }
    println!("minimal role Host example: bounded local call passed");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("content.json"),
            r#"{"persona_prompt":"A curious guide.","visual_assets":["portrait.bin"]}"#,
        )
        .unwrap();
        std::fs::write(directory.path().join("portrait.bin"), b"synthetic-portrait").unwrap();
        directory
    }

    #[test]
    fn host_owns_identity_snapshot_and_one_capability_call() {
        let directory = files();
        let host = MinimalHost::load(
            directory.path(),
            "content.json",
            "host-owned-guide-id",
            EchoLlm::default(),
        )
        .unwrap();
        assert_eq!(host.visual_assets().next().unwrap().0, "portrait.bin");
        assert_eq!(
            host.visual_assets().next().unwrap().1,
            b"synthetic-portrait"
        );
        assert_eq!(
            host.reply("host-owned-guide-id", "Hello.", "").unwrap(),
            "echo: 【角色设定】\nA curious guide.\n\n【输入材料】\nUser: Hello."
        );
        assert_eq!(host.llm.calls.get(), 1);
    }

    #[test]
    fn wrong_identity_or_unsupported_requirement_never_reaches_llm() {
        let directory = files();
        let host = MinimalHost::load(
            directory.path(),
            "content.json",
            "host-owned-guide-id",
            EchoLlm::default(),
        )
        .unwrap();
        assert_eq!(
            host.reply("other-role", "Hello.", ""),
            Err(HostError::UnknownRole)
        );
        assert_eq!(
            host.reply("host-owned-guide-id", "Hello.", "extra policy"),
            Err(HostError::Capability(BaseCallErrorKind::Unsupported))
        );
        assert_eq!(host.llm.calls.get(), 0);
    }

    #[test]
    fn invalid_identity_or_missing_asset_does_not_construct_host() {
        let directory = files();
        assert!(matches!(
            MinimalHost::load(directory.path(), "content.json", " ", EchoLlm::default()),
            Err(HostError::InvalidTechnicalId)
        ));
        std::fs::remove_file(directory.path().join("portrait.bin")).unwrap();
        assert!(matches!(
            MinimalHost::load(
                directory.path(),
                "content.json",
                "host-owned-guide-id",
                EchoLlm::default()
            ),
            Err(HostError::InvalidLocalFiles)
        ));
    }
}
