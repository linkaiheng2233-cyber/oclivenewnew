//! B2-C2: external-crate view of the reference Memory Base implementation.
//!
//! This target checks the published path end to end from outside the crate: it imports
//! `KeywordMemoryBase` through `oclive_kernel_runtime::domain::base_memory`, calls it as
//! `&dyn MemoryBase` with stack-borrowed materials and query, and drives the returned future to
//! completion with a no-op waker. It needs no Host, `Role`, database, model or emotion data.
//!
//! The future is polled for real here (not merely constructed), but this is still an offline
//! boundary test: the implementation performs no I/O, so nothing here proves network behaviour,
//! timing, cancellation or remote execution.

use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::MemoryBase;
use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
use oclive_kernel_types::MemoryBaseRequest;

fn call(materials: &[&str], query: &str) -> Vec<String> {
    let slot: &dyn MemoryBase = &KeywordMemoryBase;
    let mut future = slot.retrieve(MemoryBaseRequest { materials, query });
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(hits)) => hits,
        Poll::Ready(Err(error)) => {
            panic!("unexpected failure from the reference implementation: {error}")
        }
        Poll::Pending => panic!("the reference implementation completes on the first poll"),
    }
}

#[test]
fn b2_c2_public_path_selects_material_and_preserves_it() {
    let materials = ["Alice 不喜欢咖啡", "Bob 喜欢茶", "Coffee break at 10:00"];

    assert_eq!(call(&materials, "咖啡"), vec!["Alice 不喜欢咖啡"]);
    assert_eq!(call(&materials, "coffee"), vec!["Coffee break at 10:00"]);
    // Literal substring: "喜欢" also occurs inside "不喜欢", so both materials are selected. That is
    // the documented rule, not a polarity judgement about the text.
    assert_eq!(
        call(&materials, "喜欢"),
        vec!["Alice 不喜欢咖啡", "Bob 喜欢茶"]
    );
    assert_eq!(call(&materials, "喜欢茶"), vec!["Bob 喜欢茶"]);
    assert!(call(&materials, "巧克力").is_empty());
}

#[test]
fn b2_c2_public_path_keeps_order_duplicates_and_empty_query_rule() {
    let materials = ["beta coffee", "alpha coffee", "beta coffee"];
    assert_eq!(
        call(&materials, "coffee"),
        vec!["beta coffee", "alpha coffee", "beta coffee"]
    );

    // An empty query selects every material; a list holding one empty string is not an empty list.
    assert_eq!(call(&["", "coffee"], ""), vec!["", "coffee"]);
    assert_eq!(call(&[""], ""), vec![""]);
    let none: [&str; 0] = [];
    assert!(call(&none, "").is_empty());
}

#[test]
fn b2_c2_public_path_is_scoped_to_the_materials_of_each_call() {
    let first = ["coffee one"];
    assert_eq!(call(&first, "coffee"), vec!["coffee one"]);
    let second = ["tea two"];
    assert!(call(&second, "coffee").is_empty());
    assert_eq!(call(&second, "tea"), vec!["tea two"]);
}
