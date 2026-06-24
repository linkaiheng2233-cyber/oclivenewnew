# AI 剧场 · 模式 2 解冻 Checklist

**状态**：**Track B 产品门已通过** — 模式 2 可开工（朋友 cohort）；模式 3 仍冻结  
**最后更新**：2026-06-25  
**配套**：[`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md) §5 · §6.1 · [`PLAYTEST_MATRIX.md`](PLAYTEST_MATRIX.md) · [`MODE2_RFC.md`](MODE2_RFC.md)

模式 2 = 用户写**剧本大纲** + 导入角色包，AI 围绕大纲自行演绎（非官方 skeleton 微改）。

---

## 产品门（Track B · 维护者主导）

- [x] 试玩 cohort：**10 位朋友**（非项目协作者；**非**正式「5 名零文档陌生人」矩阵，见下备注）
- [x] 每人填 [`PLAYTEST_MATRIX.md`](PLAYTEST_MATRIX.md)（≥1 场景 + ≥1 chip）
- [x] **7/10（70%）** 出现「卧槽」或同等强度正面反应（Y）— 满足 ≥60% 门槛
- [x] **4/10** 追问「我也能做一个？」/「怎么自己做角色？」
- [x] 失败样本归因写入矩阵「问题一句」列

**维护者签字**：Keven（维护者） · 日期 **2026-06-25**

> **备注（如实记录）**：正式 checklist 要求「5 名零文档陌生人」；本次为**朋友 cohort**（10 人）。产品门按 ≥60% 兴趣结论解冻模式 2；模式 2 上线后可选补 5 陌生人复测。

---

## 工程门（Track A 已就绪 · 发测前复验）

- [x] `node scripts/dimension5-acceptance.mjs --ci`（含 theater prompt drift）
- [x] `node scripts/theater-prompt-drift.mjs`
- [x] `npm run test:theater:smoke`
- [x] `cargo test -p oclive_kernel_host --lib`
- [x] `cargo test -p oclivenewnew-tauri --test theater_director_resolver`
- [ ] `npm run tauri:build:theater`（发测前至少一次）

---

## 架构门（RFC 已开 · 实现进行中）

模式 2 设计约束（Track C = [`MODE2_RFC.md`](MODE2_RFC.md)）：

- [x] **仍不进** `process_message`、**不扩**六槽
- [x] 走 theater 圈外 API（`POST /theater/scene`）+ `theater_director` 新 mode **`outline_rewrite`**
- [x] UI：[`INFORMATION_ARCHITECTURE.md`](INFORMATION_ARCHITECTURE.md) 模式 2 叠层（不污染模式 1 单屏）
- [x] Prompt pack：官方插件 `prompts/modes/outline_rewrite.mjs`

---

## 显式冻结

| 项 | 直到 |
|----|------|
| 模式 3（`send_message` 长对话） | 模式 2 playtest 扩展矩阵通过后（另开计划） |

**工程代理 100% 不替代真人「卧槽」门槛**（朋友 cohort 已落档；陌生人矩阵为可选复测）。
