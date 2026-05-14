# 社区角色包索引格式（ROLE_PACK_INDEX）

本文定义 **静态 JSON 索引**，供市场站、启动器或脚本拉取「可发现的角色包」列表。与 **单包磁盘结构**（[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md)）正交：索引不替代包内 `manifest.json`。

---

## 文件约定

- **媒体类型**：`application/json; charset=utf-8`
- **根类型**：**数组**，元素为 **对象**（下称「条目」）
- **编码**：UTF-8
- **扩展名建议**：`.json`（例如 `catalog.json`、`role_pack_index.json`）

---

## 条目字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 是 | 与包内 `manifest.id` 一致 |
| `name` | string | 是 | 展示名 |
| `version` | string | 是 | 与包内 `manifest.version` 对齐 |
| `author` | string | 否 | 作者 |
| `description` | string | 否 | 摘要 |
| `tags` | string[] | 否 | 分类标签；客户端可按标签过滤 |
| `download_url` | string (uri) | 是 | `.zip` / `.ocpak` / `.oclivepack` 等下载地址 |
| `sha256` | string | 否 | 小写十六进制 SHA-256，便于完整性校验 |
| `min_runtime_version` | string | 否 | 与 manifest 同名字段一致时复制即可 |

**程序化过滤示例**：按标签 `tags` 包含 `"sf"` 过滤；按 `semver` 比较 `version`（需自行解析）。

---

## 最小示例

```json
[
  {
    "id": "com.example.demo",
    "name": "Demo",
    "version": "0.1.0",
    "author": "Example",
    "description": "Minimal sample",
    "tags": ["sf", "builtin-only"],
    "download_url": "https://cdn.example.com/packs/com.example.demo-0.1.0.oclivepack",
    "sha256": "abcdef0123456789…"
  }
]
```

**JSON Schema**：`crates/oclive-cli/schemas/role_pack_index.schema.json`。
