# 四仓双语基线（阶段 0）

用于回归扫描与 i18n 挂载核对；**不替代**各仓内更细的进度说明（如 `oclive-plugin-market/docs/I18N_PROGRESS.md`）。

## vue-i18n 方案（已定稿）

- **版本**：`vue-i18n@^11`（与主仓 / 编写器 / 启动器 / 市场站一致）。
- **入口**：各仓 `src/i18n/index.ts` — `createI18n({ legacy: false, locale, fallbackLocale: "zh-CN", messages })`。
- **偏好键**：`LOCALE_PREF_KEY = "oclive.appLocale"`（四端一致）。
- **挂载**：`main.ts` 中 `app.use(i18n)` 在 `mount` 之前。

| 仓库 | `src/i18n/index.ts` | `main.ts` / `App` |
|------|----------------------|-------------------|
| oclivenewnew | 是 | `main.ts` |
| oclive-pack-editor | 是 | `main.ts` |
| oclive-launcher | 是 | `main.ts` |
| oclive-plugin-market | 是 | `main.ts` |

## CJK 扫描（Han 字）

在仓库根目录执行（需 [ripgrep](https://github.com/BurntSushi/ripgrep)）：

```bash
rg -l "\p{Han}" --glob "*.vue" --glob "*.ts" src
```

- **oclivenewnew**：主路径以 `src/i18n/locales/` 词条为准；残留多在开发者面板、脚手架、部分演示组件。
- **oclive-pack-editor**：同上，`src/components/pack/` 与视图为主。
- **oclive-launcher**：`src/` 下视图与公告组件。
- **oclive-plugin-market**：`src/views/`、`src/components/`；论坛 / 管理 / 个人页等见 `docs/I18N_PROGRESS.md` 跟进列表。

## 验收

各仓：`npm run build`。主仓额外：`npm run test:unit`、`cargo test --workspace`。

**最近一次本机验收（Agent 会话内）**：主仓 `npm run test:unit` + `npm run build` + `cargo test --workspace`；`oclive-pack-editor`、`oclive-launcher`、`oclive-plugin-market` 均 `npm run build` 通过。
