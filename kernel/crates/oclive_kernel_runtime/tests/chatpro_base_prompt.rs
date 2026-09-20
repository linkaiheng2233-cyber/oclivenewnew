//! CP-B3-ALL unit P: external view of the prompt connection core.
//!
//! Drives the **public** `BuiltinPromptAssembler` through both of its entries:
//!
//! * the legacy product entry ([`PromptAssembler::build_prompt`]) with a real `PromptInput`, and
//! * the Base entry ([`PromptBase::assemble`]) with local borrowed material.
//!
//! It proves the public contract of each entry and the consistency between them: the Base entry
//! reproduces the product output byte for byte when that output is passed back as a single
//! material, because both sides join prepared fragments with the same connection rule. It does not
//! claim that any remote/directory prompt backend was migrated, and it does not run a model.
//!
//! All material and expectations here are local values; no file, database, network or model is
//! touched.

use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{PromptAssembler, PromptBase};
use oclive_kernel_runtime::domain::prompt_assembler::BuiltinPromptAssembler;
use oclive_kernel_runtime::domain::prompt_builder::{PromptBuilder, PromptInput};
use oclive_kernel_types::models::{Memory, PersonalityVector, Role};
use oclive_kernel_types::{BaseCallErrorKind, EventType, PromptBaseRequest};

type BaseOutcome = Result<String, BaseCallErrorKind>;

/// Drives the Base entry with a no-op waker; an implementation that wrongly stays pending fails the
/// calling test instead of being skipped.
fn assemble(slot: &dyn PromptBase, materials: &[&str], requirements: &str) -> BaseOutcome {
    let request = PromptBaseRequest {
        materials,
        requirements,
    };
    let mut future = slot.assemble(request);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(text)) => Ok(text),
        Poll::Ready(Err(error)) => Err(error.kind),
        Poll::Pending => panic!("the builtin prompt Base entry has no pending path"),
    }
}

fn product_input<'a>(
    role: &'a Role,
    personality: &'a PersonalityVector,
    memories: &'a [Memory],
    user_input: &'a str,
    event_type: &'a EventType,
) -> PromptInput<'a> {
    PromptInput {
        role,
        personality,
        memories,
        user_input,
        user_emotion: "happy",
        user_relation_id: "friend",
        relation_hint: "你们是朋友。",
        relation_before: "Friend",
        favorability_before: 55.0,
        relation_preview: "CloseFriend",
        favorability_preview: 60.0,
        event_type,
        impact_factor: 0.7,
        scene_label: "家",
        scene_detail: "场景细节",
        topic_hint_line: "在「家」下，你们可能会多聊日常。",
        life_context_line: "",
        worldview_snippet: "世界观外部测试片段",
        mutable_personality: "",
        ephemeral_personality: "",
        reply_quality_anchor: "",
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
        persona_override: None,
        previous_assistant_reply: "",
    }
}

/// The Base entry's own agreement: verbatim, in order, duplicates kept, no separator, and only an
/// empty `requirements` supported.
#[test]
fn chatpro_base_prompt_base_entry_is_verbatim_and_rejects_requirements() {
    let builtin = BuiltinPromptAssembler;
    let slot: &dyn PromptBase = &builtin;

    let first = String::from("第一段\r\n");
    let second = String::from("第二段\u{3000}尾");
    let materials = [first.as_str(), second.as_str()];
    assert_eq!(
        assemble(slot, &materials, "").expect("an empty requirements value is supported"),
        "第一段\r\n第二段\u{3000}尾"
    );
    // The local material is still usable: the call borrowed it for this poll only.
    assert_eq!(first, "第一段\r\n");
    assert_eq!(second, "第二段\u{3000}尾");

    // Order and duplicates are kept.
    assert_eq!(assemble(slot, &["b", "a", "b"], "").unwrap(), "bab");
    // Empty material and an empty element are both normal results.
    let empty: [&str; 0] = [];
    assert_eq!(assemble(slot, &empty, "").unwrap(), "");
    assert_eq!(assemble(slot, &[""], "").unwrap(), "");
    // Every non-empty requirements value is refused before any assembly.
    for requirements in ["逐字保留", " ", "\n", "\u{3000}"] {
        assert_eq!(
            assemble(slot, &materials, requirements).unwrap_err(),
            BaseCallErrorKind::Unsupported
        );
    }
    assert_eq!(
        assemble(slot, &empty, "逐字保留").unwrap_err(),
        BaseCallErrorKind::Unsupported
    );
}

/// Both entries are consistent: the product output fed back as one material is reproduced exactly,
/// and the connection rule adds nothing of its own.
#[test]
fn chatpro_base_prompt_product_and_base_entries_share_the_connection_rule() {
    let role = Role::default();
    let personality = PersonalityVector {
        stubbornness: 0.4,
        clinginess: 0.6,
        sensitivity: 0.7,
        assertiveness: 0.5,
        forgiveness: 0.6,
        talkativeness: 0.6,
        warmth: 0.8,
    };
    let memory = Memory {
        id: "1".to_string(),
        role_id: "external".to_string(),
        content: "用户喜欢咖啡".to_string(),
        importance: 0.8,
        weight: 1.0,
        created_at: chrono::Utc::now(),
        scene_id: None,
        mention_count: 1,
        accessed_at: None,
    };
    let memories = [memory];
    let event_type = EventType::Praise;
    let input = product_input(&role, &personality, &memories, "外部测试输入", &event_type);

    let builtin = BuiltinPromptAssembler;
    let legacy = PromptAssembler::build_prompt(&builtin, &input).expect("the product entry");
    assert!(
        !legacy.is_empty(),
        "the product preparation must keep producing a non-empty layout"
    );

    // The product layout is exactly its prepared blocks joined by the shared connection rule; the
    // segments expose the same rule for their two halves.
    let segments = PromptBuilder::build_prompt_segments(&input);
    assert_eq!(
        segments.full(),
        format!("{}{}", segments.stable_prefix, segments.dynamic_suffix)
    );
    assert_eq!(segments.stable_len(), segments.stable_prefix.len());

    // One material holding the whole product output comes back byte-identical from the Base entry.
    let slot: &dyn PromptBase = &builtin;
    assert_eq!(
        assemble(slot, &[legacy.as_str()], "").expect("single material"),
        legacy
    );
    // Two materials that split the same bytes are joined back to the same output.
    let split_at = legacy
        .char_indices()
        .nth(legacy.chars().count() / 2)
        .map(|(index, _)| index)
        .unwrap_or(0);
    assert_eq!(
        assemble(slot, &[&legacy[..split_at], &legacy[split_at..]], "").expect("two materials"),
        legacy
    );
}
