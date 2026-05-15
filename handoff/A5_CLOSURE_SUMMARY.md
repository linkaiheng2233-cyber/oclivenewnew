# A5 版本、兼容与升级 — 结项摘要（2026-05-15）

## 范围与结论

**A5.1 对外兼容一页表（P0）**：[`creator-docs/COMPATIBILITY.md`](../creator-docs/COMPATIBILITY.md) 与英文镜像 [`creator-docs-en/COMPATIBILITY.md`](../creator-docs-en/COMPATIBILITY.md) 已充实为可审阅基线：

- 明确 **主程序 `0.2.0`**、**`oclive_kernel_runtime` `0.2.0`**、**`oclive-cli` `0.1.0`**（独立 semver）与 **编写器 `0.2.x`** 的快照句。
- **一页表**新增行：**共享运行时 crate**、**脚手架 CLI**、**HTTP 载荷 `API_VERSION`（u32）/ `RUNTIME_API_VERSION`（字符串）**、**宿主 SQLite 迁移**与降级风险提示。
- **发版审阅三步**：对齐三处 `Cargo.toml`/`package.json` → 过 [`PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md)「对外说明」→ 若 bump 契约则更新 OOCP 文档/用例。

**A5.2 CHANGELOG 纪律（P0）**：[`CONTRIBUTING.md`](../CONTRIBUTING.md) 已要求 PR 带 **`CHANGELOG.md` / `CHANGELOG.en.md`** 双语条目；[`PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md) 闸门节含「已写双语」勾选；本次在 **CHANGELOG `[Unreleased]`** 增加 A5 文档条目以保持镜像同步。

## 主清单与发版表

- [`PRODUCT_AND_KERNEL_GAP_CHECKLIST.md`](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A5** 两项均已勾选（与上表一致）。
- [`PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md) **§A 映射 A5.1** 勾选为已满足基线（**每次发版**仍须按清单「对外说明」行人工核对是否需改表）。

## 工程备注

- **姊妹仓**：编写器 / 启动器 **未在本 PR 改版本号**；兼容表通过 README 链接与「快照」提醒维护者对拍。
- **DTO 路径**：宿主 `src-tauri/src/models/mod.rs` 再导出内核模型；契约以 **`oclive_kernel_runtime`** 为准（`CONTRIBUTING` 已改为指向 crate 内模型，避免失效路径）。

## 仍属后续（不记入 A5 必勾）

- **semver 自动化**：三处版本号仍人工对齐；日后可加 CI 脚本断言一致。
- **安装包 / 原生窗 E2E**：仍属 A1.1c，与 A5 正交。
