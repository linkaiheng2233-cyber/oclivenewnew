# AI Theater · 人工 Playtest 验收矩阵

> **非 CI 门禁** — 自动化 [`npm run test:theater:smoke`](../../package.json) 不替代 §4.8 陌生人「卧槽」门槛。  
> **工程就绪（Track A · 2026-06-18）**：dimension5 + drift + kernel 239 测 + `theater_director_resolver` 3 测 + smoke 全绿；模式 2 解冻见 [`MODE2_UNFREEZE.md`](MODE2_UNFREEZE.md)。  
> Prompt pack **v0.2** SSOT：[`distros/chat-pro/plugins/com.oclive.theater_director_official/`](../../distros/chat-pro/plugins/com.oclive.theater_director_official/)

## 前置

- 本地：`npm run tauri:dev:theater`
- 默认卡司：mumu × 枫侵月（或你常用的对照组）
- 模型：本地 7B 或已配置 cloud BYOK
- 每场景至少戳 **1 个代表 chip**（见下表）

## 四场景 × 代表 chip

| 场景 | 必测 chip | 通过信号 |
|------|-----------|----------|
| **breakfast** | 苦中药 **或** 快迟到 | 性格对撞、有动作/神态、接得上下一拍 |
| **supermarket** | 买牛奶 **或** 卖完了 | 场景物件感（货架/价签/空柜）、非泛化聊天 |
| **way_home** | 遇到小猫 **或** 撞电线杆 | 突发感、关心/吐槽有区分 |
| **bedtime** | 失眠 **或** 打雷下雨 | 语气变柔/变怂、夜间氛围 |

## 额外路径

- [ ] 换非默认卡司 → `cast_rewrite` 缓存命中后，poke patch 仍有人设区分
- [ ] （可选）`patch_variant=1` 双候选切换，两版措辞/走向明显不同
- [ ] （可选）RPC 失败路径：临时改错 `OCLIVE_THEATER_DIRECTOR_PLUGIN` → 确认 fallback 仍可用

## Playtest 笔记（朋友 cohort · 2026-06-25）

> **样本**：10 位朋友试玩（非零文档陌生人）；维护者签字见 [`MODE2_UNFREEZE.md`](MODE2_UNFREEZE.md)。  
> **结论**：卧槽 **7/10（70%）** · 追问「我也能做一个？」**4/10** — Track B 产品门通过。

| 日期 | 场景 | Chip | 模型 | 卡司 A×B | 卧槽？(Y/N) | 问题一句 |
|------|------|------|------|----------|-------------|----------|
| 2026-06-20 | breakfast | 苦中药 | 本地 7B | mumu×枫侵月 | Y | 性格对撞明显，想录屏 |
| 2026-06-20 | breakfast | 快迟到 | 本地 7B | mumu×枫侵月 | Y | 追问怎么换角色 |
| 2026-06-21 | supermarket | 买牛奶 | 本地 7B | mumu×枫侵月 | N | 第二拍略平，像聊天 |
| 2026-06-21 | supermarket | 卖完了 | DeepSeek BYOK | mumu×枫侵月 | Y | 空柜反应好笑 |
| 2026-06-22 | way_home | 遇到小猫 | 本地 7B | mumu×枫侵月 | Y | 关心/吐槽有区分 |
| 2026-06-22 | way_home | 撞电线杆 | 本地 7B | mumu×枫侵月 | N | 加载略久，耐心下降 |
| 2026-06-23 | bedtime | 失眠 | 本地 7B | mumu×枫侵月 | Y | 夜间语气变柔 |
| 2026-06-23 | bedtime | 打雷下雨 | DeepSeek BYOK | mumu×枫侵月 | Y | 卧槽，想自己做 |
| 2026-06-24 | breakfast | 换称呼 | 本地 7B | 非默认卡司 | Y | cast_adapt 后仍有人设 |
| 2026-06-24 | way_home | 崴脚 | 本地 7B | mumu×枫侵月 | N | patch 接锚点偶发生硬 |

## Playtest 笔记模板（后续轮次）

复制下表，每测一轮填一行：

| 日期 | 场景 | Chip | 模型 | 卡司 A×B | 卧槽？(Y/N) | 问题一句 |
|------|------|------|------|----------|-------------|----------|
| | breakfast | tea / late / … | | | | |
| | supermarket | buyMilk / milkSoldOut / … | | | | |
| | way_home | strayCat / hitPole / … | | | | |
| | bedtime | insomnia / thunderstorm / … | | | | |

**卧槽判定**（对齐 [DEVELOPMENT_ROADMAP.md §4.8](DEVELOPMENT_ROADMAP.md)）：陌生人脱口「卧槽」，并追问「我也能做一个？」——才开模式 2/3。

## 反馈回流

- **seed 文案**：[`theaterSceneCatalog.ts`](../../distros/theater/distros/shared/src/composables/theater/theaterSceneCatalog.ts) `dramaSeed` / `sceneBrief`
- **纪律与 mode 模板**：官方插件 [`prompts/drama_guardrails.mjs`](../../distros/chat-pro/plugins/com.oclive.theater_director_official/prompts/drama_guardrails.mjs) 与各 `prompts/modes/*`
- **Rust fallback**（仅 RPC 失败）：[`drama_guardrails.rs`](../../kernel/crates/oclive_kernel_host/src/domain/theater/drama_guardrails.rs) — 改插件后按需同步，见插件 README sync 清单

Drift 烟测（子串，不测 LLM 质量）：`node scripts/theater-prompt-drift.mjs`
