//! CP-B3-ALL unit M: external view of the builtin Memory backend.
//!
//! Drives the **public** `QueryMemoryRetrieval` through both of its entries:
//!
//! * the Host port ([`MemoryRetrieval::rank_memories`]) with real `Memory` rows and a real query,
//! * the Base view ([`MemoryBase::retrieve`]) with local borrowed material.
//!
//! Both entries project **one** selection core, so the two views are required to agree on the same
//! call material: the Base view returns the original text of the selected entries, and the Host
//! view returns the same entries as `Memory` rows without losing id, weight, time, scene, mention
//! count or `accessed_at`. The file also pins what this unit deliberately does **not** do: nothing
//! is written back, no content dedupe happens, and the Host compatibility fallback (a query that
//! matched nothing) is not reported as a Base hit.
//!
//! CP-B3-ALL-R1: the selection core and its term helper are **not** public, so this target reaches
//! the rule only through the two real trait entries above. Every rule boundary that used to be
//! asserted on the helper is now stated as an observable consequence on the published entries (for
//! example the scalar-bigram term is shown by a material that contains one of the two scalars but
//! not the pair). Helper-internal boundaries stay in that module's own unit tests.
//!
//! All material and expectations here are local values; no file, database, network or model is
//! touched.

use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{MemoryBase, MemoryRetrieval};
use oclive_kernel_runtime::domain::query_memory::QueryMemoryRetrieval;
use oclive_kernel_types::models::Memory;
use oclive_kernel_types::{MemoryBaseRequest, MemoryRetrievalInput};

/// Drives the Base entry with a no-op waker; an implementation that wrongly stays pending fails
/// the calling test instead of being skipped.
fn retrieve(slot: &dyn MemoryBase, materials: &[&str], query: &str) -> Vec<String> {
    let request = MemoryBaseRequest { materials, query };
    let mut future = slot.retrieve(request);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(selected)) => selected,
        Poll::Ready(Err(error)) => {
            panic!("this implementation performs no I/O and must not fail: {error}")
        }
        Poll::Pending => panic!("this implementation has no pending path"),
    }
}

fn memory(id: &str, content: &str, importance: f64) -> Memory {
    Memory {
        id: id.to_string(),
        role_id: "external".to_string(),
        content: content.to_string(),
        importance,
        weight: 1.0,
        created_at: chrono::Utc::now(),
        scene_id: Some("scene-a".to_string()),
        mention_count: 3,
        accessed_at: None,
    }
}

/// The Base view keeps the original text, the original order and the original duplicates, and an
/// empty (`trim`-empty) query still selects every explicit material.
#[test]
fn chatpro_base_memory_base_view_returns_original_material() {
    let builtin = QueryMemoryRetrieval;
    let slot: &dyn MemoryBase = &builtin;

    let first = String::from("用户喜欢咖啡\r\n");
    let second = String::from("用户不喜欢\u{3000}咖啡");
    let unrelated = String::from("今天星期三");

    // A two-scalar non-ASCII query is one adjacent-scalar term, so it selects both coffee entries;
    // the unrelated row is not selected and nothing is rewritten.
    assert_eq!(
        retrieve(
            slot,
            &[first.as_str(), unrelated.as_str(), second.as_str()],
            "咖啡"
        ),
        vec![
            "用户喜欢咖啡\r\n".to_string(),
            "用户不喜欢\u{3000}咖啡".to_string()
        ]
    );
    // The borrowed material is still usable and unchanged after the call.
    assert_eq!(first, "用户喜欢咖啡\r\n");
    assert_eq!(second, "用户不喜欢\u{3000}咖啡");

    // The scalar-bigram rule, observed on the published entry: a material that contains only one of
    // the two scalars is *not* selected by the pair, while the single-scalar query selects it.
    let half = String::from("用户只提到咖");
    assert!(retrieve(slot, &[half.as_str()], "咖啡").is_empty());
    assert_eq!(
        retrieve(slot, &[half.as_str()], "咖"),
        vec!["用户只提到咖".to_string()]
    );

    // A pure-ASCII query segment is used whole: case is ignored, but it is not a prefix match.
    let ascii = String::from("Alpha 和 alphabet soup");
    assert_eq!(
        retrieve(slot, &[ascii.as_str()], "alpha"),
        vec!["Alpha 和 alphabet soup".to_string()]
    );
    assert_eq!(
        retrieve(slot, &[ascii.as_str()], "ALPHABET"),
        vec!["Alpha 和 alphabet soup".to_string()]
    );
    let short = String::from("Alpha");
    assert!(
        retrieve(slot, &[short.as_str()], "alphabet").is_empty(),
        "a pure-ASCII segment is matched whole, not as a prefix"
    );

    // An empty query selects every material; a query with no usable term selects none.
    assert_eq!(
        retrieve(
            slot,
            &[first.as_str(), unrelated.as_str(), second.as_str()],
            "  "
        )
        .len(),
        3
    );
    assert!(retrieve(slot, &[first.as_str()], "!!!").is_empty());

    // Two entries with identical text stay two entries, and the empty material is a normal value.
    let same = "重复记忆";
    assert_eq!(
        retrieve(slot, &[same, "", same], "重复"),
        vec!["重复记忆".to_string(), "重复记忆".to_string()]
    );
}

/// The two views are one projection of one selection: the Host rows and the Base texts describe the
/// same entries in the same call, and the Host view preserves every field it received.
#[test]
fn chatpro_base_memory_host_and_base_views_agree_on_one_selection() {
    let builtin = QueryMemoryRetrieval;
    let rows = vec![
        memory("high-unmatched", "今天星期三", 1.0),
        memory("low-matched", "用户喜欢咖啡", 0.2),
        memory("mid-matched", "用户喜欢咖啡和茶", 0.6),
    ];
    let contents: Vec<&str> = rows.iter().map(|row| row.content.as_str()).collect();

    let ranked = MemoryRetrieval::rank_memories(
        &builtin,
        MemoryRetrievalInput {
            memories: &rows,
            user_query: "咖啡",
            scene_id: Some("scene-a"),
            limit: 8,
        },
    )
    .expect("the builtin Host entry does not fail on local rows");
    let host_ids: Vec<&str> = ranked.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(host_ids, vec!["mid-matched", "low-matched"]);

    let slot: &dyn MemoryBase = &builtin;
    let base_texts = retrieve(slot, &contents, "咖啡");
    // The selection is the same set, but the two views order it differently by contract: the Base
    // view keeps the original input order of the selected entries, while the Host view applies the
    // existing weighted order. The views are two projections of one selection, not two orderings.
    assert_eq!(
        base_texts,
        vec!["用户喜欢咖啡".to_string(), "用户喜欢咖啡和茶".to_string()],
        "the Base view keeps the original input order of the selected entries"
    );
    let mut base_as_host_order: Vec<&str> = base_texts.iter().map(String::as_str).collect();
    let mut host_as_host_order: Vec<&str> = ranked.iter().map(|row| row.content.as_str()).collect();
    base_as_host_order.sort_unstable();
    host_as_host_order.sort_unstable();
    assert_eq!(
        base_as_host_order, host_as_host_order,
        "the Base view and the Host view must describe the same set of selected entries"
    );

    // Every field of the original rows survives into the Host view, and nothing was written back.
    let low = ranked
        .iter()
        .find(|row| row.id == "low-matched")
        .expect("matched row");
    assert_eq!(low.role_id, "external");
    assert_eq!(low.importance, 0.2);
    assert_eq!(low.weight, 1.0);
    assert_eq!(low.scene_id.as_deref(), Some("scene-a"));
    assert_eq!(low.mention_count, 3);
    assert_eq!(low.accessed_at, None);
    assert!(rows.iter().all(|row| row.accessed_at.is_none()));
    assert_eq!(
        rows.iter()
            .map(|row| row.content.as_str())
            .collect::<Vec<_>>(),
        contents,
        "the Host entry must not rewrite the rows it was given"
    );

    // The existing limit is still applied after the existing weighted order over the matched rows.
    let limited = MemoryRetrieval::rank_memories(
        &builtin,
        MemoryRetrievalInput {
            memories: &rows,
            user_query: "咖啡",
            scene_id: Some("scene-a"),
            limit: 1,
        },
    )
    .expect("limit");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].id, "mid-matched");

    // A query that matched nothing keeps the previous all-candidate weighted selection as an
    // explicit Host compatibility fallback, and the Base view reports no hit for the same call.
    let fallback = MemoryRetrieval::rank_memories(
        &builtin,
        MemoryRetrievalInput {
            memories: &rows,
            user_query: "完全不相关词",
            scene_id: Some("scene-a"),
            limit: 8,
        },
    )
    .expect("fallback");
    let fallback_ids: Vec<&str> = fallback.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(
        fallback_ids,
        vec!["high-unmatched", "mid-matched", "low-matched"],
        "the fallback is the previous all-candidate order, not a Base hit"
    );
    assert!(retrieve(slot, &contents, "完全不相关词").is_empty());
}
