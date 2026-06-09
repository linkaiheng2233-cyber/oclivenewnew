# AI Theater v0 — 60 秒陌生人验收清单

**发行版**：`distro_id=theater` · profile [`examples/distro-profiles/theater.oclive.toml`](../examples/distro-profiles/theater.oclive.toml)

## 启动

```powershell
$env:OCLIVE_DISTRO_PROFILE = "examples/distro-profiles/theater.oclive.toml"
$env:VITE_OCLIVE_SHELL = "theater"
npm run dev
# 或 npm run dev:theater（仅前端壳；Tauri 需同步设置 OCLIVE_DISTRO_PROFILE）
```

## 自动烟测

```bash
npm run test:unit -- src/theater/theater.acceptance.test.ts
```

## 人工 60 秒清单

| 秒数 | 预期 |
|------|------|
| 0–5 | 首屏出现「早饭」场景标签；**零 LLM** 即出现第一条小焦台词 |
| 5–20 | 双角色台词交替出现（小焦 / 阿懒），可见反差 |
| 20–25 | 点击「喝苦中药」或「快迟到了」芯片 → 全屏「改动加载中」→ 台词有可见变化 **或** 降级提示后仍继续播放 |
| 25–30 | 「改性格」可点击（打开编写器或角色包目录） |
| 全程 | 无 API Key 配置；无六槽/蓝图/插件管理入口 |

## GET /health

`distro_id` 应为 `theater`（spawn 时注入 `OCLIVE_DISTRO_ID` + `OCLIVE_DISTRO_PROFILE`）。

## 刻意不做（v0）

多场景、录制、通用剧情引擎、第二剧场、内核新槽。
