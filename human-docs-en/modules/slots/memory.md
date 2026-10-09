# Slot pack · `memory` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/memory.md`](../../../human-docs/modules/slots/memory.md)
> Scope: developer entry points and checklist. Public entry points: [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页); reference Host definitions: [§4](../../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory).
> Updated: 2026-10-09.

## Choose your entry point

| Your task | Start here |
|---|---|
| Implement basic retrieval called through the current public interface | The **MemoryBase example** below; use caller-supplied materials and resources |
| Maintain the reference Host's rich backend, storage or recollection policy | **Reference Host** below; follow existing `MemoryRetrieval`, configuration and storage wiring |

### MemoryBase example: run it, then replace the implementation

1. Read the author's [`memory.rs`](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/memory.rs) for its retrieval agreement and `impl MemoryBase`. In [`main.rs`](../../../kernel/crates/oclive_kernel_runtime/examples/external_memory_base/main.rs), follow the borrowed implementation in `MinimalRoleBaseBindings.memory` and the real `MemoryBase::retrieve(&consumer, request)` call.
2. Run these commands from the repository root. These reuse the verified offline invocation and require a prepared Rust dependency cache; prepare dependencies first if the cache is missing. The example implementation compiles separately from the runtime library. It needs no blueprint, STM/LTM, model configuration or running reference Host.

   ```powershell
   cargo run --locked --offline -p oclive_kernel_runtime --example external_memory_base
   cargo test --locked --offline -p oclive_kernel_runtime --example external_memory_base
   ```

3. The current example prints `external memory: selected=2 baseline=3 memory_calls=1 llm_calls=0`. Both implementations receive the same materials but follow their own retrieval agreements: the new implementation selects the latest two matches; the existing keyword implementation selects three. `literal:<exact substring>` is this author's agreement, not a Base query standard.
4. Replace the algorithm and its corresponding call requirements, then check the output, successful empty results and complete failures through the consumer. The caller supplies materials, resource access and the executor. This example only drives immediately ready in-memory implementations; its `immediate` helper is not an executor for arbitrary asynchronous providers.

**Checklist**:

- [ ] State the accepted query and material agreement; preserve the complete failure for unsupported requests.
- [ ] Call through the existing consumer and distinguish an empty success from a failure. A successful call does not prove all relevant facts were found.
- [ ] Keep data sources and resource scope explicit; do not implicitly read old materials or databases, or acquire new authority.

Use the [public entry-point checklist](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页) for requests, failures and binding. This example calls only Memory; binding six references does not require running all six slots on every turn. It does not register the custom implementation as a blueprint backend in the reference Host.

## Reference Host

**You plug in**: blueprint `slot_registry.type: memory` → folded `PluginBackends.memory` (legacy v1: `settings.json.plugin_backends.memory`) · trait `MemoryRetrieval` · `pre.rs` retrieval. Reference Host orchestration performs STM/LTM writes in `post_llm`; those writes are not authority granted by the retrieval trait.

**Three stores** (do not confuse): chat log (`chat_*`) ≠ `short_term_memory` ≠ `long_term_memory`. Deep dive: [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md).

Memory may propose `memory.recall.candidate`; an Event decision module may produce `memory.recollection.activated`. The event carries IDs and scores, not memory text. Text is fetched outside the ring only after admission for one-turn recollection context. Contract: [EVENT_RING](../../../creator-docs-en/plugin-and-architecture/EVENT_RING.md).

**Don't**: Declare a candidate already remembered · copy memory text into event payloads · use chat messages as memory truth · clear STM/LTM when deleting UI chat.

**Read next**: [01 architecture (EN)](../../01_ARCHITECTURE_SIMPLE.md) · [chat-storage pack (ZH)](../../../human-docs/modules/side-channels/chat-storage.md).
