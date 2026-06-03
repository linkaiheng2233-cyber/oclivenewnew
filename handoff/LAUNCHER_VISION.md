# 启动器愿景（Phase 5 · 仅记录）

**状态**：路线图；**本迭代不实现** `oclive-launcher` 代码变更。

---

## 目标

用户安装任意 OCLive 发行版（桌面、VS Code、未来渠道）后，由 **启动器** 或首次引导统一：

1. **发现** 已安装的发行版与内核二进制  
2. **提升** 最佳质量内核到 `%LOCALAPPDATA%/OCLive/runtime/`  
3. **固定** 数据目录 `%LOCALAPPDATA%/OCLive/data/`（`OCLIVE_APP_DATA`）  
4. **配置** `OCLIVE_ROLES_DIR`（首次选择或沿用已有）  
5. **协调** 单写者：`:8420` health → attach，否则 spawn 一次  

---

## 与现有资产

| 已有 | 启动器职责 |
|------|------------|
| VS Code `discovery.ts` promote runtime | 泛化为全发行版 promote |
| `OCLIVE_APP_DATA` + 迁移 | 首次启动检测 + CLI `migrate-app-data` |
| `oclive-runtimed`（可选） | 健康监督 + per-role 队列 |
| 桌面 in-process `:8420` | 桌面在线时其它宿主 attach |

---

## 非范围（当前）

- OAuth / 账号体系  
- 跨机器云同步  
- 强制 kill 其它发行版进程  

---

## 验收（未来）

- 仅安装启动器 + 角色库 → 可 spawn 内核并完成一轮 `mumu` 对话  
- 后装桌面 → VS Code 自动 attach，好感连续  
- 卸载任一发行版不删除 `OCLive/data`  
