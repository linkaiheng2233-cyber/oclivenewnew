# AI Theater — 15 秒惊喜验收（主清单）

**发行版**：`distro_id=theater`  
**模式**：**Mode 1 微调 only**（大纲 / 自由演绎不在本清单范围）  
**路线图**：[`THEATER_DISTRO_ROADMAP.md`](./THEATER_DISTRO_ROADMAP.md)

> 完整 60 秒体验与 GET /health 检查见 [`THEATER_V0_ACCEPTANCE.md`](./THEATER_V0_ACCEPTANCE.md)。

---

## 启动

```powershell
npm run dev:theater
# Tauri 安装包验证时：
# $env:OCLIVE_DISTRO_PROFILE = "examples/distro-profiles/theater.oclive.toml"
# $env:VITE_OCLIVE_SHELL = "theater"
```

---

## 自动烟测

```bash
npm run test:unit -- src/theater/
```

Wave T2 起增加：前 3 beat 累计 delay 预算断言（见 roadmap T2-TEST-01）。

---

## 15 秒人工清单（陌生人 · 零文档）

| 秒数 | 预期 | 失败 = |
|------|------|--------|
| **0–2** | 看见「早饭」场景 + **第一条小焦台词**（零 LLM） | 空白 / 只有标题 / 要求配置 |
| **2–10** | **至少 2 条**不同角色台词，性格反差可感 | 单角色独白 / 读不懂在干什么 |
| **10–15** | 点 **任意 1 个** poke 芯片 → 台词有可见变化 **或** 轻量降级提示后继续 | 无反应 / 卡死 / 弹出 API 设置 |
| **全程** | 不见：模式 Tab（默认）、六槽/插件管理、内核 startup 告警条 | 像开发者工具而非产品 |

### 通过标准

- **≥60%** 测试者（建议 5 人）在 15 秒内完成上表 **且无失败项**。
- 汇总写入 [`THEATER_STRANGER_TEST_ROUND1.md`](./THEATER_STRANGER_TEST_ROUND1.md)。

---

## 性能 mark（开发机 · Wave T2）

在 DevTools Performance 或 `readTheaterPokePerfSample()` 旁路记录：

| 段 | 目标（P50 开发机） |
|----|-------------------|
| 打开 → 第一条台词 | **≤ 2s**（含 skeleton fetch） |
| 前 3 beat 播完 | **≤ 12s** |
| poke → 首条新台词 | **≤ 8s**（有 Ollama）；无 Ollama 即时 fallback |

---

## 刻意不做（本阶段）

- 第二场景、赌场 DLC
- 发行版专用内核二进制
- Mode 2/3 陌生人验收
