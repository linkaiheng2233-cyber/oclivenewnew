# AI Theater — 陌生人测试主持人指南

**用途**：带 5 名未看过文档的同学做 Mode 1（15 秒惊喜）验收。  
**填表 SSOT**：[`THEATER_STRANGER_TEST_ROUND1.md`](./THEATER_STRANGER_TEST_ROUND1.md) §真人陌生人  
**验收清单**：[`THEATER_15S_ACCEPTANCE.md`](./THEATER_15S_ACCEPTANCE.md)

---

## 两种启动方式

| 方式 | 命令 | 适用 |
|------|------|------|
| **最快（推荐）** | `npm run dev:theater` | 日常带测、迭代快 |
| **贴近 Release** | `npm run tauri:build:theater` → 安装 `src-tauri/target/release/bundle/` 产物 | 验证打包后首屏 |

开发机前置：Node 20+、本仓 `npm ci` 已完成。**Ollama 可不启动**——无 LLM 时 poke 应走 fallback，仍算通过。

---

## 你对测试者只说这些

> 「打开这个，看 15 秒，点一下下面芯片，告诉我感觉。」

**零文档**——不要解释六槽、蓝图、Mode 2/3、插件或内核。

---

## 观察清单（每人 15 秒）

对照 [`THEATER_15S_ACCEPTANCE.md`](./THEATER_15S_ACCEPTANCE.md) 四行：

| 秒数 | 预期 | 失败 = |
|------|------|--------|
| 0–2 | 早饭场景 + 第一条小焦台词 | 空白 / 只有标题 / 要求配置 |
| 2–10 | ≥2 条不同角色台词，反差可感 | 单角色独白 |
| 10–15 | 点 1 个 poke → 台词变化或轻量降级提示 | 无反应 / 卡死 / API 设置 |
| 全程 | 不见模式 Tab（默认）、六槽/插件、startup 告警 | 像开发者工具 |

可选记录是否说「卧槽」（参考指标，非硬门禁）。

---

## 填表

1. 复制 [`THEATER_STRANGER_TEST_ROUND1.md`](./THEATER_STRANGER_TEST_ROUND1.md) §真人陌生人 5 行。
2. 每人填：15s 完成 Y/N · 卧槽 Y/N · 卡在哪 · 是否点 poke · 是否展开高级 · 备注。
3. 更新 §汇总：样本数、15s 通过率、「卧槽」率、是否达标（≥60%）。
4. 执行日期写入文档底部。

通过后可 commit：`docs(theater): stranger test round1 human results`

---

## 常见问题

| 现象 | 处理 |
|------|------|
| Ollama 未开 | 正常；poke 应 instant fallback，不算失败 |
| 测试者展开「高级模式」 | 记录 Y，提醒下一人不点；不算自动失败 |
| 首屏空白 | 记失败项「0–2」；查 DevTools 网络 `/theater/breakfast/skeleton.json` |
| 想聊 Mode 3 | 本轮回不测；引导只玩默认首屏 + 芯片 |

---

## 工程自检（带测前可选）

```powershell
npm run test:theater:smoke
npm run test:distro:smoke
```

工程代理已在 CI；真人表空白时 **产品门槛未过**，见 [`THEATER_PHASE4_READINESS.md`](./THEATER_PHASE4_READINESS.md)。

---

## 测试后分支

| 结果 | 动作 |
|------|------|
| 真人 ≥60% | 更新 readiness · 继续导演插件产品化（C-pass） |
| 真人 <60% | 对照失败项开 P4-3 patch（见 plan §C-fail） |

**P4-3 不触发条件**：工程代理 100% 且真人 ≥60%。
