//! External-crate call of the additive minimal-role Prompt Base implementation.

use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::PromptBase;
use oclive_kernel_runtime::domain::minimal_role_prompt::LocalMinimalRolePrompt;
use oclive_kernel_types::{BaseCallErrorKind, PromptBaseRequest};
use oclive_validation::minimal_role_local_file::load_minimal_role_local_file;

fn drive(
    slot: &dyn PromptBase,
    materials: &[&str],
    requirements: &str,
) -> Result<String, BaseCallErrorKind> {
    let mut future = slot.assemble(PromptBaseRequest {
        materials,
        requirements,
    });
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(Ok(text)) => Ok(text),
        Poll::Ready(Err(error)) => Err(error.kind),
        Poll::Pending => panic!("the local implementation has no pending path"),
    }
}

#[test]
fn one_visual_asset_and_persona_configure_prompt_base_without_legacy_role() {
    let directory = tempfile::tempdir().unwrap();
    let raw = r#"{"persona_prompt":"  A curious guide.\n","visual_assets":["portrait.bin"]}"#;
    std::fs::write(directory.path().join("content.json"), raw).unwrap();
    std::fs::write(directory.path().join("portrait.bin"), b"local-asset").unwrap();

    let snapshot = load_minimal_role_local_file(directory.path(), "content.json", 1024, 64, 64)
        .expect("one prompt and one asset are sufficient");
    assert_eq!(snapshot.assets().len(), 1);
    let implementation = LocalMinimalRolePrompt::new(&snapshot);
    let slot: &dyn PromptBase = &implementation;

    assert_eq!(
        drive(slot, &["User: ", "Hello."], "").unwrap(),
        "【角色设定】\n  A curious guide.\n\n\n【输入材料】\nUser: Hello."
    );
    assert_eq!(
        drive(slot, &["Hello."], "must use a different persona"),
        Err(BaseCallErrorKind::Unsupported)
    );
    assert_eq!(
        snapshot.assets().next().unwrap(),
        ("portrait.bin", b"local-asset".as_slice())
    );
}
