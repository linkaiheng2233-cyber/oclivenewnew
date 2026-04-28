# 角色包市场索引（roles.json）v1

> 目标：为 oclive 提供 **角色包一键安装** 的只读索引（不强绑定网站，不强制 GitHub），支持：
>
> - 同一角色包版本的 **多镜像下载**（GitHub / 对象存储 / 网盘等）
> - 客户端安装前 **SHA-256 校验**（避免“链接被换包”）
> - 角色包安装语义：下载 → 校验 → 解压到 `OCLIVE_ROLES_DIR/{roleId}/` → 可覆盖/可回滚（回滚由版本选择实现）
>
> 设计原则（Linux 风格）：
> - **索引负责声明与可验证性**；文件托管可以外部化（多源镜像）。
> - **用户体验在客户端**（安装/回滚/风险提示），网站与论坛可选。

---

## 1. 顶层结构

```json
{
  "generatedAt": "2026-04-28T00:00:00Z",
  "roles": []
}
```

- `generatedAt`：可选，生成时间（ISO8601 字符串）。
- `roles`：角色包条目数组。

---

## 2. RoleIndexEntry（角色包条目）

> JSON 字段为 **camelCase**（与插件市场索引一致）。

最小示例：

```json
{
  "type": "role",
  "id": "mumu",
  "name": "沐沐",
  "description": "……",
  "author": "Oclive",
  "version": "0.2.0",
  "minRuntimeVersion": "0.2.0",
  "tags": ["日常", "治愈"],
  "downloads": [
    {
      "label": "GitHub Release（海外）",
      "kind": "direct",
      "url": "https://github.com/example/mumu/releases/download/v0.2.0/mumu.ocpak",
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "trust": "verified"
    },
    {
      "label": "网盘镜像（国内）",
      "kind": "page",
      "url": "https://pan.example.com/s/xxxx",
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "note": "提取码：ABCD",
      "trust": "community"
    }
  ]
}
```

### 2.1 字段说明

- `type`：固定为 `"role"`。
- `id`：角色 id，安装后目录名即 `OCLIVE_ROLES_DIR/{id}/`（必须与 `manifest.json.id` 一致）。
- `name`：展示名。
- `description`：简介（建议简短；详细介绍放在外链/README）。
- `author`：作者/发布者显示名。
- `version`：角色包版本（建议 semver）。
- `minRuntimeVersion`：可选，最低运行时版本（与角色包 `manifest.min_runtime_version` 语义一致）。
- `tags[]`：可选标签。
- `downloads[]`：下载镜像列表（同一版本可多条）。

### 2.2 `downloads[]`（镜像项）

每个镜像项字段：

- `label`：展示名（如“GitHub Release”“百度网盘镜像”等）。
- `kind`：`direct | page | pan`
  - `direct`：直链下载（客户端可直接 fetch）。
  - `page`：下载页（可能需要手动操作；客户端可打开浏览器，并提示用户自行下载后再导入）。
  - `pan`：网盘类（等同 `page` 的风险提示更强；可选）。
- `url`：链接。
- `sha256`：**必须**。角色包文件 bytes 的 SHA-256（64 个 hex 小写/大写均可；建议小写）。
- `note`：可选说明（提取码、镜像建议等）。**不得包含用户隐私**。
- `trust`：可选信任级别（用于客户端提示/排序）：`official | verified | community | unknown`

---

## 3. 客户端安装语义（摘要）

- 角色包安装优先走 `downloads[].kind=direct` 的镜像（按 `trust` 排序）。
- 客户端下载后必须：
  1) 计算 SHA-256，与索引 `sha256` 一致才允许继续；
  2) 解压 `.ocpak/.zip` 时必须防 zip-slip（拒绝 `..`、绝对路径等非法条目）；
  3) 解压后必须能解析 `manifest.json`，且 `manifest.id == entry.id`；
  4) 写入 `OCLIVE_ROLES_DIR/{id}/`（支持覆盖/回滚）。

---

## 4. 治理与风险提示（v1 保守策略）

- **链接失效**：由条目维护者负责更新镜像；客户端应提示“可切换其它镜像”。  
- **未知来源**：`trust=unknown` 的镜像默认折叠/风险提示（提醒不要输入密钥/不要运行未知脚本）。  
- **不强制网站**：索引可托管在任意 Git 仓库 raw/对象存储；讨论与反馈可放 Discord / GitHub / 国内社区平台。

---

## 5. 创作者：角色包文件托管方式（推荐顺序）

> `roles.json` 只收录“条目与镜像链接”。文件本体可以放在任何可访问的地方，但 **必须提供 SHA-256**，让客户端安装前校验。

推荐顺序（从“最省事 + 最稳定”到“更接地气”）：

1. **GitHub Releases（推荐）**
   - 优点：版本化天然、直链稳定、全球可用（但国内可能访问不稳定）。
   - 建议 `downloads.kind="direct"` 填入 Release asset 的直链（`.../releases/download/<tag>/<file>`）。

2. **对象存储直链（R2 / OSS / COS 等）**
   - 优点：可做国内加速、成本可控。
   - 同样用 `downloads.kind="direct"`。

3. **网盘 / 下载页（国内友好）**
   - 优点：对国内用户更友好。
   - 缺点：通常不是直链，客户端无法自动下载校验；需要用户手动下载再导入。
   - 建议用 `downloads.kind="page"` 或 `downloads.kind="pan"`，并在 `note` 写明提取码等说明。

无论哪种方式，同一版本的多个镜像应指向 **同一份文件 bytes**，因此 `sha256` 应一致。

