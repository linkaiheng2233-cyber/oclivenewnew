# Theater P4-3 — Mode 1 反馈 patch 状态

**更新**：2026-06-12  
**触发条件**：真人陌生人 5 人 15s 通过率 **<60%**（见 [`THEATER_STRANGER_TEST_ROUND1.md`](./THEATER_STRANGER_TEST_ROUND1.md) §汇总）

---

## 当前结论：**未触发**

| 信号 | 状态 |
|------|------|
| 工程代理 15s | **100%** pass（5/5） |
| 真人陌生人 | **pending**（表未填） |
| P4-3 patch PR | **不需要**（无真人失败项） |

工程代理 ≥60% → **不触发** Mode 1 反馈 patch（见 [`THEATER_STRANGER_TEST_ROUND1.md`](./THEATER_STRANGER_TEST_ROUND1.md) §P4-3 触发）。

---

## 若真人 <60% 时的对照表（预置，勿提前改代码）

| 失败集中项 | 对策 | 文件 |
|------------|------|------|
| 0–2s 空白 | skeleton / 首 beat | `public/theater/breakfast/skeleton.json` |
| 无反差 | 文案 | skeleton + 角色包 |
| poke 无感 | fallback | `src/theater/useTheaterBeatPatch.ts` |
| 像开发者工具 | UI | `src/shells/theater/TheaterShell.vue` · 隐藏高级/告警 |

**不在 P4-3 开**：导演插件接线、新场景、Mode 2/3 陌生人验收。

Commit 模板：`fix(theater): address stranger test round1 human findings`

---

## Related

- [`THEATER_STRANGER_FACILITATOR.md`](./THEATER_STRANGER_FACILITATOR.md)
- [`THEATER_15S_ACCEPTANCE.md`](./THEATER_15S_ACCEPTANCE.md)
- [`THEATER_PHASE4_READINESS.md`](./THEATER_PHASE4_READINESS.md)
