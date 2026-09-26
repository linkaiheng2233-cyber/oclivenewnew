# 三发行版内核阶段结项（Pro · Flash · Theater profile）

**状态**：结项 · **更新**：2026-06-12  
**范围**：Chat Pro（`desktop`）· VS Code Flash（`vscode`）· Theater profile 预埋（`theater`）· 同构建各打包

---

## 1. 产品决策回顾

| 发行版 | `distro_id` | Hero / 定位 | Profile SSOT |
|--------|-------------|-------------|--------------|
| **A.I.Live Chat Pro** | `desktop` | 桌面 Release hero | [`distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml`](../distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml) |
| **VS Code Flash** | `vscode` | 姊妹仓 VSIX | [`examples/distro-profiles/vscode.oclive.toml`](../examples/distro-profiles/vscode.oclive.toml) · 镜像 [`oclive-vscode/distro.oclive.toml`](../../oclive-vscode/distro.oclive.toml) |
| **dev lab** | `desktop-chat` | 实验场 only | [`examples/distro-profiles/desktop-chat.oclive.toml`](../examples/distro-profiles/desktop-chat.oclive.toml) |
| **AI Theater** | `theater` | Phase 4 打包预埋 | [`examples/distro-profiles/theater.oclive.toml`](../examples/distro-profiles/theater.oclive.toml) |

**同构建各打包**：Tauri 安装包经 `bundle-kernel-for-tauri.mjs` 写入 bundled 内核；spawn 决策 **bundled → shared → dev**（K-SCHED-05）。

**内核编排仍冻结**（[`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §5.5）：theater 发行版打包 / 插件 / UI 在模式 1 范围内，**不是**新 `process_message` stage。

---

## 2. 交付物清单

| 项 | 路径 / 机制 | Commit（主仓） |
|----|-------------|----------------|
| Pro bundled profile | `distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml` | `d91789fa` |
| Tauri kernel bundle | `scripts/bundle-kernel-for-tauri.mjs` · `npm run bundle-kernel:tauri` | `63711570` |
| bundled-first spawn | `pick_best_for_spawn` · K-SCHED-05/01 | `63711570` |
| Flash profile 镜像 | `scripts/diff-vscode-distro-profile.mjs` | `d91789fa`（主仓）· `8a4cdae`（vscode） |
| Distro e2e | `scripts/e2e-distro-kernel.mjs`（含 theater scenario） | 本结项 |
| Tauri bundled e2e | `scripts/e2e-tauri-bundled-kernel.mjs` | `63711570` + 本结项 CI |
| 聚合 smoke | `npm run test:distro:smoke` | 本结项 |

---

## 3. Smoke 结果表（R1 · 2026-06-12）

| 命令 | 环境 | 结果 |
|------|------|------|
| `cargo test -p oclive_kernel_runtime kernel_` | Windows 10 · dev | **pass**（20 tests） |
| `npm run test:distro-profile-mirror` | Windows · 主仓 | **pass** |
| `npm run test:e2e:distro-kernel` | Windows · 主仓 | **pass**（spawn / attach / role-snapshot / bundled-first / theater） |
| `npm run test:e2e:tauri-bundled-kernel` | Windows · 主仓 | **pass** |
| `npm run test:distro:smoke` | Windows · 主仓 | **pass**（聚合） |
| `cd oclive-vscode && npm run test:distro-profile-mirror` | Windows · 姊妹仓 | **pass** |
| `cd oclive-vscode && npm run test:ensure-report` | Windows · 姊妹仓 | **pass** |

CI **`cross-host-e2e`**（Ubuntu 22.04）含：`e2e-cross-host-memory` · `e2e-distro-kernel --scenario all` · `e2e-kernel-profile` · **`e2e-tauri-bundled-kernel`** · **`diff-vscode-distro-profile`**（姊妹仓缺失时 skip warn）。

### 2026-09-27 下游发行版适配回归（非小 Kernel 验收）

在当前源码上离线重建 `oclive-kernel-server` 与 `oclive-cli` 后，`node scripts/e2e-distro-kernel.mjs --scenario all` 通过六个场景；`npm run test:distro-profile-mirror` 通过。`role-portability` 现使用桌面与剧场安装资源中的 profile、以及 VS Code 示例 profile，逐一核对 `/health` 的发行版 ID、profile 文件 SHA-256、有效 Prompt 档位、Agent／复杂情绪开关和模式切换策略，再执行同一 `mumu` 角色的 `role/load → /chat`。这证明参考 Host 可按这些 profile 装配并完成受控 mock LLM 回合，也降低了只凭非空回复误判 profile 的风险。

验收方向从小 Kernel 向外：六槽 Base 的独立调用与最低结果／错误边界先由 [B1 + B2 阶段总收口](README.md#six-slot-stage-closure) 中的夹具和具体实现测试验证，再检查 Host／Adapter 如何选用能力，最后才是本段发行版 smoke。后者不能反向定义 Base、要求所有六槽都直接进入 ChatPro 产品路径，或把参考 Host 的完整 `Role` 生命周期当作小 Kernel 的必备部分。最小角色逻辑与资产准备属于外围数据适配的独立证据；当前参考 Host 尚不能激活“仅人设＋单图”的角色，是该 Host 的适配范围，不是六槽 Base 验收失败。这里也没有运行真实模型、Tauri 桌面、VSIX 或剧场 UI；`bundled-first` 只检查调度计划，不证明当前安装包二进制已启动。边界见 [模块职责](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities) 与 [角色包职责边界](ROLE_PACK_BOUNDARY.md)。

---

## 4. CI 覆盖

| Job | Scenario |
|-----|----------|
| **`cross-host-e2e`** | 跨宿主记忆 · distro kernel（含 theater）· profile · **Tauri bundled-first** · VS Code profile mirror diff |
| **`oocp-test-suite`** | OOCP S0–S12 + dual-core S13/S14 · core-api-restart |
| **姊妹仓 `oclive-vscode` CI** | `test:ensure-report`（EnsureReport 契约） |

---

## 5. Explicit Deferred

| 项 | 说明 |
|----|------|
| per-distro 裁剪 binary | 仍用全量 bundled + sidecar `feature_set` |
| Theater roles 子集打包 | Phase 4 首 epic |
| sidecar 分 manifest 字段 | P2b Deferred |
| VS Code F5 / VSIX 发布验收 | V-VSCODE-PERF-05 Deferred |
| 导演 directory 插件 | **已交付** · `distros/chat-pro/plugins/com.oclive.theater_director_official/`；见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §5 |

---

## 6. Phase 4 入口

Pro + Flash smoke **已通过** → Theater **模式 1** greenfield 重建中（产品陌生人测试仍 pending）。

→ [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md)  
→ [`theater/README.md`](./theater/README.md)

---

## Related

- [KERNEL_SCHEDULER_RESCOPE.md](./KERNEL_SCHEDULER_RESCOPE.md) · K-SCHED-01/02/04/05 Done
- [DISTRO_KERNEL_LIFECYCLE.md](../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md)
- 历史记录：[PHASE4_ECOSYSTEM_NOTES.md](./archive/PHASE4_ECOSYSTEM_NOTES.md)
