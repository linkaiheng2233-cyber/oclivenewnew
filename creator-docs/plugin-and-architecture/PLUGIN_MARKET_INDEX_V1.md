# 插件市场索引（plugins.json）v1

> 目标：为 oclive 官方插件市场提供**只读索引**（不托管插件包），支持：
> - 默认 Git 安装（`git clone --branch <tag>`）
> - 多版本回滚（`download_url` + `signature_url` + ED25519 验签）
> - 开发者自签 + 官方登记公钥（支持 revoke/rotate）

---

## 1. 顶层结构

```json
{
  "generated_at": "2026-04-27T00:00:00Z",
  "plugins": []
}
```

- `generated_at`：可选，生成时间（ISO8601 字符串）。
- `plugins`：插件条目数组。

---

## 2. PluginIndexEntry（市场条目）

> JSON 字段为 **camelCase**（与后端 serde 配置一致）。

条目类型（v1 冻结）：

- **`type: "plugin"`**：有代码的插件条目（默认）。支持 `git` 安装、`versions` 回滚、`publicKeys` 验签。
- **`type: "module"`**：**无代码**的模块条目（meta package）。只包含“声明 + 依赖列表 + 可选后端预设”，用于一键拉取一组插件并写入配置。
- **`type: "profile"`**：保留（无代码）。用于一键部署更大粒度环境（可复用 module / plugin 机制实现）。

### 2.1 `type:"plugin"` 最小示例（包含 Git 安装 + 多版本回滚 + 公钥登记）

```json
{
  "type": "plugin",
  "id": "com.example.foo",
  "name": "Foo Plugin",
  "description": "…",
  "author": "Alice",
  "version": "0.3.0",
  "git": "https://github.com/alice/foo-oclive-plugin.git",
  "permissions": ["network:*"],
  "tags": ["llm"],
  "category": "llm",
  "publisher": "alice",
  "publicKeys": [
    {
      "pubkeyId": "alice-2026-01",
      "publicKey": "BASE64_ED25519_PUBKEY_32_BYTES",
      "status": "active"
    }
  ],
  "versions": [
    {
      "version": "0.3.0",
      "gitTag": "v0.3.0",
      "downloadUrl": "https://github.com/alice/foo-oclive-plugin/releases/download/v0.3.0/com.example.foo.oclive-plugin",
      "signatureUrl": "https://github.com/alice/foo-oclive-plugin/releases/download/v0.3.0/com.example.foo.signature.json"
    },
    {
      "version": "0.2.0",
      "gitTag": "v0.2.0",
      "downloadUrl": "https://github.com/alice/foo-oclive-plugin/releases/download/v0.2.0/com.example.foo.oclive-plugin",
      "signatureUrl": "https://github.com/alice/foo-oclive-plugin/releases/download/v0.2.0/com.example.foo.signature.json"
    }
  ],
  "dependencies": {
    "com.example.bar": "^1.2.0"
  }
}
```

### 2.2 兼容字段（旧客户端）

- `version`：用于旧 UI 展示/更新提示；建议写入 `versions` 中的最新版本号。
- `git`：默认安装路径的 Git 仓库 URL。

### 2.3 新字段（治理/安全/回滚）

- `publisher`：发布者 id（字符串）。用于把“开发者身份/历史”治理与公钥登记绑定到同一主体。
- `publicKeys[]`：发布者公钥环（可多把钥，支持轮换）。
  - `pubkeyId`：公钥标识（索引用于选择与撤销）。
  - `publicKey`：Ed25519 公钥（32 bytes）base64。
  - `status`：`active | revoked | rotated`（建议值；具体由官方索引约定）。
  - `rotatedTo`：当 `status=rotated` 时指向新的 `pubkeyId`。
- `versions[]`：多版本信息（用于回滚/离线包安装）。
  - `version`：版本号（建议 semver）。
  - `gitTag`：可选。Git 安装时使用的 tag；省略时客户端可默认使用 `version` 字符串。
  - `downloadUrl` / `signatureUrl`：回滚安装所需的不可变 URL（建议 GitHub Releases Assets）。

---

## 2.4 `type:"module"`（无代码模块条目）

最小示例：

```json
{
  "type": "module",
  "id": "module.creator-min",
  "name": "创作者最小闭环（模块）",
  "description": "拉取一组创作者常用插件，并应用一套后端预设。",
  "author": "Oclive",
  "version": "1.0.0",
  "git": "",
  "permissions": [],
  "tags": ["module"],
  "dependencies": {},
  "module": {
    "plugins": [
      { "id": "com.example.llm.bridge", "version": "1.2.3", "source": "official" }
    ],
    "backends": {
      "llm": "directory",
      "directory_plugins": { "llm": "com.example.llm.bridge" }
    }
  }
}
```

硬性规则（安全冻结）：

- `type="module"` 条目 **不包含代码**，因此：
  - `git` 必须为空字符串（或省略但实现侧应视为空）
  - 禁止提供 `versions` / `publicKeys`（这些仅属于 `type="plugin"`）
- `module.plugins[]` 为“依赖插件清单”：安装模块时应逐个拉取这些插件，并对每个插件走安装权限确认。
- `module.backends` 为“配置预设”：安装后写入会话级后端覆盖（并可包含 `directory_plugins.*` 槽位）。
- **保守策略（v1）**：`type:"module"` / `type:"profile"` 仅负责“安装依赖插件 + 写入后端覆盖/槽位配置”，**不包含 UI 插槽位置/顺序**，实现侧不得修改用户的 `plugin_state`（如 `slot_order` / `disabled_slot_contributions` / `slot_appearance`）。用户可在应用后自行到「插槽顺序」调整位置。

---

## 3. signature.json（插件包签名文件）

> 开发者使用 ED25519 对 **整个 `.oclive-plugin` 包文件 bytes** 签名。

```json
{
  "pluginId": "com.example.foo",
  "pubkeyId": "alice-2026-01",
  "algorithm": "ed25519",
  "signature": "BASE64_ED25519_SIGNATURE_64_BYTES",
  "signedAt": "2026-04-27T00:00:00Z",
  "covers": "archive_bytes"
}
```

- `pluginId`：必须与索引条目的 `id` 一致。
- `pubkeyId`：必须存在于索引条目的 `publicKeys[]` 中，且不能是 `revoked`。
- `algorithm`：固定 `"ed25519"`。
- `signature`：Ed25519 签名（64 bytes）base64。
- `signedAt` / `covers`：可选元信息（用于人类理解与未来扩展）。

---

## 4. 客户端安装语义（摘要）

- **默认一键安装**：优先走 Git（`git clone --branch <gitTag>`）。
- **回滚/指定版本安装**：使用 `downloadUrl` + `signatureUrl`：
  - 下载 `.oclive-plugin`
  - 下载 `signature.json`
  - 用索引登记的公钥验签，不通过则阻止安装

