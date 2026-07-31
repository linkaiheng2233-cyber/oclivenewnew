# Good First Issues（策展清单 · 轮次 15）

维护者可在 GitHub 用本表批量创建 `good-first-issue` / `good second issue` 标签 issue。题面已写好，复制标题与正文即可。

## good-first-issue（#1–10 · 轮次 13）

| # | 标题 | 标签 | 说明 |
|---|------|------|------|
| 1 | `docs: CHANGELOG.en.md parity for [Unreleased]` | `good first issue`, `documentation` | 对照 `CHANGELOG.md` `[Unreleased]` 补英文一句摘要 |
| 2 | `docs: onboarding local-link smoke` | `good first issue`, `documentation` | 跑 `node scripts/check-markdown-links.mjs human-docs creator-docs` 并修复失效入口 |
| 3 | `test: portrait_catalog enabled + missing file fallback` | `good first issue`, `rust` | 扩 `persistence.rs` 或 `portrait_facility` 边界单测 |
| 4 | `chore: verify:ui anchor maintenance` | `good first issue` | 跑 `npm run verify:ui`，确认锚点与 `SimplePluginManagerPanel` 一致 |
| 5 | `i18n: align chatStorage capabilitiesDegraded in shared bundle` | `good first issue`, `frontend` | 若 `distros/shared/src/i18n/shared` 有镜像，同步新键 |
| 6 | `docs: CONTRIBUTING Node >=20 engines note` | `good first issue`, `documentation` | 确认各入口文档与 `package.json` `engines` 一致 |
| 7 | `docs: Windows setup appendix link from CONTRIBUTING` | `good first issue`, `documentation` | 链到 `human-docs/10_SETUP_WINDOWS.md` |
| 8 | `docs: PR gate matrix smoke — docs-only PR checklist` | `good first issue`, `documentation` | 验证 `human-docs/08_PR_GATE_MATRIX.md` 命令可执行 |
| 9 | `test: hybrid_store mirror best-effort warn coverage` | `good first issue`, `rust` | 可选 `tracing_test` 断言 warn 行 |
| 10 | `docs: README Contributing card i18n mirror` | `good first issue`, `documentation` | `README.en.md` 与中文 Contributing 三链对齐 |

## good-second-issue（#11–13 · 轮次 15 · V4-ONBOARD-03）

边界单测与可观测性；需熟悉 `oclive_kernel_host` 测试布局。标签建议：`good second issue`, `rust`。

| # | GitHub | 标题 | 说明 |
|---|--------|------|------|
| 11 | [#71](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues/71) | `test: emotion_analyzer empty input and neutral fallback` | 扩 `kernel/crates/oclive_kernel_host/src/domain/emotion_analyzer.rs` 边界：`""` / 纯标点 / 超长输入；断言 `EmotionResult` 维度在 0–1 且 `format_for_prompt` 不 panic |
| 12 | [#72](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues/72) | `test: slot_resolver mock BackendRegistry position merge` | 参照 `distros/desktop-tauri/tests/slot_resolver_v3.rs`；用 mock `SlotRegistryResolver` / 最小 `pipeline.ocblueprint` 断言 memory 多实例按 position 列出 |
| 13 | [#73](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues/73) | `test: hybrid_store mirror rebuild emits tracing warn on failure` | `kernel/crates/oclive_kernel_host/src/infrastructure/chat_storage/hybrid_store.rs`；`tracing_test` 或 capture subscriber 断言 `rebuild_mirror_best_effort` 失败路径 `warn!` 含 session id |

**创建命令（维护者）：**

```powershell
gh issue create --repo linkaiheng2233-cyber/oclivenewnew --title "…" --body "…" --label "good second issue" --label "rust"
```

**过滤 URL**（创建后）：  
`https://github.com/linkaiheng2233-cyber/oclivenewnew/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22`  
`https://github.com/linkaiheng2233-cyber/oclivenewnew/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+second+issue%22`
