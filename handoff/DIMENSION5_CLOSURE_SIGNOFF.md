# Dimension 5 工程纪律 — 审查签字页

**验收日期**：2026-06-08  
**审查代号**：Opus 4.8 工程纪律审查  
**状态**：**Closed**

---

## 验收门（8 条）

| # | 门 | 证据 |
|---|-----|------|
| 1 | `scripts/dimension5-acceptance.mjs` 本地与 CI 绿 | 本页 + CI job `dimension5-acceptance` |
| 2 | 本签字页日期 ≥ 当前 `Cargo.lock` 维护周期 | 2026-06-08 |
| 3 | TECHNICAL_DEBT Dimension 5 全 Done/Deferred；Opus 4.7 表无矛盾 Pending | [TECHNICAL_DEBT_INVENTORY.md](./TECHNICAL_DEBT_INVENTORY.md) |
| 4 | OOCP health 场景覆盖 `startup_warnings` | `examples/oocp-test-suite/run.mjs` S0b |
| 5 | `oclive_sqlx` crates 文档 SSOT | [crates/oclive_sqlx/README.md](../crates/oclive_sqlx/README.md) · [crates/README.md](../crates/README.md) |
| 6 | Layering ratchet ≤ 4（D-LAYER-04 生产路径端口化） | [LAYERING_BASELINE.json](./LAYERING_BASELINE.json) |
| 7 | cargo-audit 漏洞级 0 | [KNOWN_VULNERABILITIES.md](../creator-docs/security/KNOWN_VULNERABILITIES.md) |
| 8 | EnsureReport golden | `cargo test -p oclive-cli --test kernel_ensure_plan_snapshot` |

---

## 机器验收命令

```bash
# 完整本地验收
node scripts/dimension5-acceptance.mjs

# CI 快速模式（跳过慢速抽样 cargo test）
node scripts/dimension5-acceptance.mjs --ci
```

---

## 快照（2026-06-08）

| 指标 | 值 |
|------|-----|
| domain→infrastructure import ratchet | **4**（D-LAYER-04；仅 `#[cfg(test)]` 保留 Mock/test_db；见 `LAYERING_BASELINE.json`） |
| `cargo audit` | 退出码 0；`Cargo.lock` 无 `sqlx-mysql` / `rsa` |
| Dimension 5 子项 | D-CI-01…D-FREEZE-01 **Done**；D-POLICY-01 **Deferred** |
| npm audit CI | job `npm-audit`（`continue-on-error: true` 可见性） |

---

## Dimension 5 表（摘要）

完整表见 [TECHNICAL_DEBT_INVENTORY.md §Dimension 5 closure](./TECHNICAL_DEBT_INVENTORY.md#dimension-5-closure工程纪律2026-06-08)。

---

## Opus 4.8 审查 — 未纳入本维度

见 [TECHNICAL_DEBT_INVENTORY.md §Opus 4.8 Deferred](./TECHNICAL_DEBT_INVENTORY.md#opus-48-deferred2026-06-08)。

---

## 相关文档

- [BUS_FACTOR_NOTES.md](./BUS_FACTOR_NOTES.md)
- [ARCHITECTURE_LAYERING.md](./ARCHITECTURE_LAYERING.md)
- [creator-docs/getting-started/DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) §工程纪律 / 审查状态
