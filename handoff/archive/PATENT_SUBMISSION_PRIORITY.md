# 学校专利 — 优先递交清单（硬件向）

**日期：** 2026-06-09  
**策略：** 优先 4 件「硬件/方法」授权概率高的交底书；创意型可凑数量，不额外打磨。

## 优先递交（A / C / ③ / ④）

| 代号 | 交底书文件（`Desktop/专利/`） | 核心主张 |
|------|------------------------------|----------|
| **A** | `一种多发行版宿主共享内核的能力画像感知附着替换与派生决策方法-交底书v1.md` | 多发行版 `distro.oclive.toml` + `resolve_kernel_action` attach/replace |
| **C** | `一种可组装对话内核的焊接式高耦合编译模式生成方法-交底书v1.md` | Monolith 七焊接键 + `oclive-cli init --monolith` |
| **③** | `一种对话系统中聊天记录与编排记忆解耦的记忆离线回放与相似合并方法-交底书v1.md` | hybrid chat storage + replay_memory_extraction |
| **④** | `一种桌面人工智能插件宿主的能力声明授权双重校验与远程能力分级降级方法-交底书v1.md` | manifest permissions + high_risk_grants + directory 降级 |

## 递交动作

1. 按学校模板填写「技术交底书参考.md」封面信息（发明人、院系、联系方式）。
2. 上述 4 份 v1 交底书 PDF 导出，文件名带序号 A/C/③/④。
3. 说明书附图：可从 `creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md` 与 `handoff/CHAT_STORAGE_ARCHITECTURE.md` 截取架构图（注明非保密公开文档）。
4. 递交后在本文件「递交回执」节记录受理号（手动填写）。

## 递交回执

| 件 | 受理号 | 日期 |
|----|--------|------|
| A | _待填_ | |
| C | _待填_ | |
| ③ | _待填_ | |
| ④ | _待填_ | |

## 创意型（可选凑数，低优先级）

B 插件槽合并、E 预生成骨架+局部补丁、F 共景位移、G 人格演化等 — 已有交底书草稿，**不阻塞 Theater v0**。

E 与 Theater v0 产品相关，但专利流程与产品交付并行，不以专利审查阻塞发版。
