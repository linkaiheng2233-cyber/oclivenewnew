# WAVE-20260717-K-PLUGIN-SEC-01-s0

> 计划书：[`../long-plans/K-PLUGIN-SEC-01.md`](../long-plans/K-PLUGIN-SEC-01.md)

## 摘要

| 字段 | 值 |
|------|-----|
| **债 ID** | K-PLUGIN-SEC-01 |
| **Stage** | 0 · Inventory plugin UI trust boundaries |
| **日期** | 2026-07-17 |
| **Claim** | n/a · Codex 单 Agent，用户已授权本地写入与测试 |
| **Base HEAD** | `c5953026` |
| **执行面** | Codex desktop · no sub-agent · no push/merge capability |

## 证据

| 检查 | 结果 |
|------|------|
| `npm run check:debt-marathon`（新增计划前基线） | **PASS · 10 auto plans** |
| `rg -n "ocliveplugin\|plugin_bridge\|vueComponent\|sandbox" distros kernel` | **PASS · 攻击面已枚举** |
| Git 基线 | `origin/main` + 6 个已审查本地提交；工作树 clean 后进入本 Stage |

## 对齐结论

- embedded slot：共享 custom-protocol origin、无 sandbox、直接 Tauri bridge。
- full-shell：替换主 WebView，受 remote capability 覆盖，必须独立验收。
- 仅校验请求声明不能证明浏览器调用 frame；Stage 1 采用 opaque-origin + `event.source` 绑定代理。
- Voice HTML fallback 仅占位；Stage 2 同时偿还功能回退与 `vue3-sfc-loader` 生产依赖。
- 签名默认与吊销流程仍由 K-SUPPLY-09 共同约束，不能只靠源码提示。

## 下一跳

`K-PLUGIN-SEC-01` Stage 1：source-bound parent broker + iframe sandbox + negative cross-frame tests。

## GATES §6

- [x] Stage 0 只新增计划/队列/证据文档，未改生产代码
- [x] 已读 GATES、流水线、台账、插件契约和关键代码
- [x] 未提升 TECHNICAL_DEBT Done，未合 main，未 push
- [x] 下一 Stage 文件范围、检查和回退条件已写入计划书
