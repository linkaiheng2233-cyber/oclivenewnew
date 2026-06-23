# AI 剧场 · 模式 2 解冻 Checklist

**状态**：**BLOCKED** — Track B 产品门未过 + 维护者未签字  
**最后更新**：2026-06-18  
**配套**：[`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md) §5 · §6.1 · [`PLAYTEST_MATRIX.md`](PLAYTEST_MATRIX.md)

模式 2 = 用户写**剧本大纲** + 导入角色包，AI 围绕大纲自行演绎（非官方 skeleton 微改）。**解冻前禁止**模式 2 UI / API / 新 `mode` 代码。

---

## 产品门（Track B · 维护者主导）

- [ ] 5 名**零文档**陌生人完成试玩（非项目协作者、未读过 handoff）
- [ ] 每人填 [`PLAYTEST_MATRIX.md`](PLAYTEST_MATRIX.md)（≥1 场景 + ≥1 chip）
- [ ] ≥3/5（**60%**）出现「卧槽」或同等强度正面反应（Y）
- [ ] ≥2/5 追问「我也能做一个？」/「怎么自己做角色？」（建议，与 roadmap spirit 一致）
- [ ] 失败样本归因写入矩阵「问题一句」列；prompt/seed 回流后复测通过

**维护者签字**：________ · 日期 ________

---

## 工程门（Track A 已就绪 · 发测前复验）

- [ ] `node scripts/dimension5-acceptance.mjs --ci`（含 theater prompt drift）
- [ ] `node scripts/theater-prompt-drift.mjs`
- [ ] `npm run test:theater:smoke`
- [ ] `cargo test -p oclive_kernel_host --lib`
- [ ] `cargo test -p oclivenewnew-tauri --test theater_director_resolver`
- [ ] `npm run tauri:build:theater`（发测前至少一次）

---

## 架构门（解冻后再写 RFC · 本 checklist 不实现代码）

模式 2 设计约束（Track C 首 PR = `MODE2_RFC.md` 一页）：

- [ ] **仍不进** `process_message`、**不扩**六槽
- [ ] 走 theater 圈外 API（`POST /theater/scene`）+ `theater_director` 新 mode（如 `outline_rewrite`）
- [ ] UI：[`INFORMATION_ARCHITECTURE.md`](INFORMATION_ARCHITECTURE.md) 模式 2 章节 **解冻后另开 PR**（不污染模式 1 单屏）
- [ ] Prompt pack：官方插件 `prompts/modes/outline_*.mjs` fork 路径文档化

---

## 显式冻结

| 项 | 直到 |
|----|------|
| 模式 2 UI / API / 新 mode 实现 | 本 checklist 产品门 + 工程门 + 维护者签字 |
| `MODE2_RFC.md` | 产品门通过后 |
| 模式 3（`send_message` 长对话） | 模式 2 playtest 扩展矩阵通过后（另开计划） |

**工程代理 100% 不替代真人「卧槽」门槛。**
