# Robot soul minimal（K3 示例）

面向 **无头 / 嵌入式** 交付的最小角色包目录，满足 `oclive pack validate --profile robot-soul`。

## 校验

在 **oclivenewnew** 仓库根：

```bash
cargo run -p oclive-cli -- pack validate examples/robot-soul-minimal/roles/default --host-version 0.2.0 --profile robot-soul
```

## 字段说明

见主仓 [ROLE_PACK_SPEC.md](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) § RobotSoulPack（中英：`creator-docs-en/role-pack/ROLE_PACK_SPEC.md`）。

## 联调

将 `roles/default` 复制到内核工程的 `roles/` 下，或使用 `OCLIVE_ROLES_DIR` 指向本目录的父级 `roles/`；无头 HTTP 见 [KERNEL_IMPLEMENTATION_PLAN.md](../../creator-docs/getting-started/KERNEL_IMPLEMENTATION_PLAN.md) K1。
