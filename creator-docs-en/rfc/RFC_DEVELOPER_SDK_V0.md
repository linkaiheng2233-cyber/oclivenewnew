# Source integration toolkit · first module development path

**Status**: the maintainer selected a source toolkit using existing components, an outside-workspace consumer project and dependency/verification guidance. No new public SDK facade.
**Updated**: 2026-10-13. `V0` identifies this document; it is not a released SDK version or a new API stability promise.
**SSOT scope**: SDK delivery shape and expansion limits. Public responsibilities remain in [MODULE_MAP](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries).

[中文](../../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md)

## Start here

Follow the bilingual [source SDK Memory guide](../../examples/developer-sdk-memory/README.md): generate a foundation with the current CLI, explicitly add the shared runtime, copy the existing Memory example, and compile, test and run outside the repository. The guide identifies what to edit, the expected results, source/lock inputs and directory relocation limits.

The existing [foundation template](../cli/OCLIVE_CLI_GUIDE.md) keeps its contracts/types dependencies. Adding consumer support is a subsequent author choice, not a new scaffold default. The project contains no reference Host, desktop shell or turn scheduler. Teaching bindings are supplied explicitly by the caller and are not registered as default modules.

## First delivery

| Component | Purpose and current delivery |
|---|---|
| `oclive_kernel_contracts`, `oclive_kernel_types` | Existing six-slot Base contracts, requests, results and complete failures; existing compatibility rules apply |
| `oclive_kernel_runtime` | Explicitly selected shared integration support; this path uses `MinimalRoleBaseConsumer` |
| Independent consumer project | Existing `init --template kernel-base --base-example none`, then Cargo adds runtime; its own workspace, manifest and lock |
| Existing Memory source | Copy canonical `main.rs`, `memory.rs` and repository license; keep one maintained teaching implementation and edit the author's own copy |
| Usage and verification | [Guide](../../examples/developer-sdk-memory/README.md) and a copyable project README; links to tests, entry points and target Host responsibilities |

The three crates currently have version `0.2.0`. This delivery uses source path dependencies and publishes no new registry SDK package. Contracts/types include compatibility surfaces and runtime has broader support: none of these whole crates should be equated with the small Kernel. See the [source map](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-source-map) and [Base entry points](../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页).

## Interpreting verification

| Layer | What this slice checks | What it does not establish |
|---|---|---|
| Project usability | Outside-workspace metadata/check/test/run; traceable source and lock inputs | Cross-platform packaging or registry SDK publication |
| Limited contract behavior | Selection, unchanged material text, successful empty result, complete Unsupported error and caller scope | Retrieval quality, persistence or performance of all Memory implementations |
| Consumer integration | Author implementation actually called through the existing consumer; zero generation calls | A mandatory six-slot turn sequence or rich reference Host registration |
| Target functionality | Author and target Host supply their own scenario acceptance | Acceptance for every distribution |

Windows PowerShell probes used source input `08348f31b8f3459537c13227302bf4b8e273baa8` and prepared dependencies, offline. See the [wave record](../../handoff/debt-marathon/waves/WAVE-20261013-SOURCE-SDK-FIRST-PATH.md). These are limited local results; CI for that source SHA does not certify a new documentation commit. The standalone recipe was not executed on other platforms in this slice.

## Author responsibilities

- Supply materials, resources, state, permissions and an async executor. A source dependency or successful call grants no network, tool or hardware permissions.
- Define accepted requirements. The example's `literal:` syntax belongs to its author, not the Base contract.
- Select Host wiring. The reference route can continue with the [CLI guide's library-embed recipe](../cli/OCLIVE_CLI_GUIDE.md); a custom Host follows current public contracts. Neither is started in this slice.
- Record actual source SHA, dirty state, toolchain and derived lock. Recheck caller behavior and downstream compatibility when updating sources.

## Stop and expand deliberately

Freeze the first path once an author can generate the project, run checks and locate their implementation and consumer binding. Reports should include a failing command, source SHA, tools and minimal reproduction; use the existing [CLI feedback entry](../cli/OCLIVE_CLI_GUIDE.md).

Consider packaging, other slot examples, Host presets, language bindings or publication only when a concrete development task is blocked. Do not mechanically add six identical examples, discovery, a generic executor or SDK traits. A reproducible contract limitation needs old/new cases; permissions, scheduling authority and mandatory obligations require maintainer agreement. Hardware, real models/TTS, standalone browsers and untested crash windows receive no new acceptance claim from this guide.
