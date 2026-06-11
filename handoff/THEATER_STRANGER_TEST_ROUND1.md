# 陌生人测试 — AI Theater Mode 1（15 秒惊喜）

**目标：** 5 人，**零文档**，仅 `npm run dev:theater` 或 theater 安装包。

**通过标准（T4-TEST-01 · 2026-06-12 升级）**：≥60% 在 **15 秒内**完成 [`THEATER_15S_ACCEPTANCE.md`](./THEATER_15S_ACCEPTANCE.md) 主清单且无失败项。

## 启动

```powershell
npm run dev:theater
# 或 Tauri 安装包（Windows 实机）：
# $env:VITE_OCLIVE_SHELL = "theater"
# $env:OCLIVE_DISTRO_PROFILE = "examples/distro-profiles/theater.oclive.toml"
```

自动烟测：`npm run test:theater:smoke`

## 记录表

| # | 15s 完成 (Y/N) | 是否「卧槽」(Y/N) | 卡在哪一步 | 是否点 poke | 是否展开高级 | 备注 |
|---|----------------|------------------|------------|-------------|--------------|------|
| 1 | | | | | | |
| 2 | | | | | | |
| 3 | | | | | | |
| 4 | | | | | | |
| 5 | | | | | | |

### 15 秒检查项（每人）

| 秒数 | 预期 |
|------|------|
| 0–2 | 早饭场景 + 第一条小焦台词 |
| 2–10 | ≥2 条不同角色台词，反差可感 |
| 10–15 | 点 1 个 poke → 台词变化或轻量降级提示 |
| 全程 | 不见模式 Tab（默认）、六槽/插件、startup 告警 |

## 汇总

| 指标 | 结果 |
|------|------|
| 样本数 | _待填_ |
| 15s 通过率 | _待填_ % |
| 「卧槽」率 | _待填_ % |
| 是否达标 (≥60%) | _待填_ |

**执行日期：** _待填（需维护者 Windows 实机 + 5 名陌生人）_
