# AI 剧场 — 产品状态（STATUS）

**SSOT 范围**：Theater 发行版当前产品姿态、模式阶梯与冻结指针（一页）。路线细节见 [`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md)；本文件**不**复制 RFC / playtest / 冻结长表（G14）。  
**最后更新**：2026-07-16

---

## 当前姿态

| 模式 | 状态 | 说明 |
|------|------|------|
| **模式 1** | **已交付** | 官方剧本微改（戳点 / 预生成骨架）；见路线图 §4 |
| **模式 2** | **已解冻 · 开发中** | 用户大纲 + AI 演绎；产品门见解冻 checklist；RFC 实现中 |
| **模式 3** | **仍冻结** | 角色包自由演绎 / 长对话双 cast；**本 Stage 不解冻** |

台账口径（Product freeze）：模式 2 playtest **扩展中**；**模式 3 仍冻结** — 见 [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) 文首 Product freeze · **§2 冻结 / registry** · Phase 5 结论。

---

## 指针（只链不抄）

| 主题 | 链接 |
|------|------|
| 模式 2 RFC | [`MODE2_RFC.md`](MODE2_RFC.md) |
| 模式 2 解冻 checklist | [`MODE2_UNFREEZE.md`](MODE2_UNFREEZE.md) |
| 试玩矩阵 | [`PLAYTEST_MATRIX.md`](PLAYTEST_MATRIX.md) |
| 思路与开发路线 | [`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md)（§5 / **§5.5** 产品与内核冻结） |
| 活跃债 · 冻结台账 | [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md)（文首 Product freeze · **§2** · Phase 5） |

---

## 纪律摘要

- **允许**：模式 1 交付面维护；模式 2（`outline_rewrite` 等）按 RFC 推进。  
- **禁止（仍冻）**：模式 3、`process_message` 新编排阶段、为模式 3 过早泛化架构 — 详 [`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md) §5.5。
