# OOCP 协议测试套件（说明）

**当前 `main` 状态**：本仓库 **未** 包含独立的 **OOCP（Open Oclive Chat Protocol）** 协议黑盒测试 crate 或 `examples/oocp-test-suite` 目录；CI（`.github/workflows/ci.yml`）**未** 运行 OOCP 专用 job。

若后续引入 OOCP 场景测试，建议：

1. 将可执行套件置于 **`examples/oocp-test-suite/`** 或独立 crate，并在 CI 中增加 job（`--json` 输出与 schema 校验）。
2. 在本文件记录 **场景编号（S0–Sn）**、**JSON schema 路径** 与 **本地运行命令**。
3. 在 [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)「测试与质量」中更新链接。

**相关（当前存在）**：Rust 集成测试 `src-tauri/tests/`、`cargo test`；前端 **`npm run build`**（CI）作为静态构建守门。
