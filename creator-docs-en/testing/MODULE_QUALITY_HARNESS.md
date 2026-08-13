# Module behavior-quality harness

This tool evaluates the same fixed roles, scenes, and replays across the memory, emotion, prompt, and LLM modules. It answers whether a declared module configuration satisfies explicit fixture expectations. It does not claim that one model is objectively best for every conversation.

## Quick start

Build the kernel server, capture the current remote-slot baseline, and compare it with the checked-in reference observations:

```bash
cargo build -p oclive-kernel-server
npm run quality:modules
```

The output has three sections:

- `observations`: contract-approved data captured through the real HTTP, replay, and remote-slot path;
- `report`: per-case and per-dimension findings for the current configuration;
- `comparison`: side-by-side reference and current results. Both configurations must use the exact same suite digest and must declare different module identity combinations.

To compare existing observation files:

```bash
node scripts/module-quality-compare.mjs \
  --observations path/to/config-a.observations.json \
  --observations path/to/config-b.observations.json
```

Every observation declares an `id` and `version` for all four modules. Changing only `run_id` does not create a second configuration; the comparator rejects duplicate module combinations.

## Inputs and scoring

The versioned suite is `examples/module-quality-harness/fixtures/suite.v1.json`. Each case fixes:

- `role_id`, `scene_id`, and a multi-turn `replay`;
- required and forbidden memory/prompt text;
- allowed emotion labels;
- required-any LLM text, forbidden text, and a maximum user-echo ratio.

The four dimension scores remain separate; there is no aggregate score. A failure means an explicit fixture expectation was not met, not that the module or model is universally poor. A pass covers only the current fixture contract and does not replace subjective role-naturalness review, long-conversation testing, or live-model sampling.

## Privacy and isolation

The runner copies only the fixture roles into a temporary directory, points the four slots at a local JSON-RPC sidecar, and executes through the existing `/chat/storage` and `/chat` routes. Only `mq-*` fixture memories are included in the safe observation prompt. Other runtime memories, the complete production prompt, and user tokens are excluded from the report.

Temporary API/LLM tokens, the database, and role copies exist only inside the run directory. The runner reaps the kernel process tree and removes that directory on exit. Reports still contain fixture conversation text, so never replace fixtures with private real-world transcripts.

## Keep quality and performance separate

`comparison.quality` contains only behavior findings. `comparison.performance.status` is currently always `not_measured`; the harness does not infer latency, throughput, CPU, GPU, or memory from wall-clock execution. Use `oclive bench` and hardware matrices for performance, and present the two report types side by side rather than combining them into one score.

## Maintainer checks

```bash
npm run test:module-quality
npm run check:module-compat
node scripts/check-doc-mirror.mjs
node scripts/dimension5-acceptance.mjs --ci
```

`test:module-quality` is an offline contract self-test and does not start a model. `quality:modules` starts a local kernel capture. Release or debt closure still requires remote CI for the exact target commit; local green checks are not a substitute.

---

[中文](../../creator-docs/testing/MODULE_QUALITY_HARNESS.md)
