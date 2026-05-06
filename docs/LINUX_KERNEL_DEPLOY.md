# Oclive 无头内核 — Linux 部署权威指南

本文面向 **Ubuntu 22.04 LTS x86_64**（及同类 Debian 系）运维与集成方，说明如何在 **无 Tauri / 无 Vue** 的前提下部署 **`oclive_kernel_server`**，与桌面版共享同一套 **`oclive_kernel_runtime` HTTP/OOCP 契约**。

- 架构背景与阶段路线：[`LINUX_KERNEL_ENGINE.md`](./LINUX_KERNEL_ENGINE.md)  
- 合成模板（Compose、systemd 片段）：[`../delivery/README.md`](../delivery/README.md)

---

## 1. 交付物与边界

| 组件 | 说明 |
|------|------|
| 二进制 | `cargo build -p oclive_kernel_server --release` → `target/release/oclive_kernel_server` |
| 协议 | `GET /health`（无鉴权）、`POST /chat`、OOCP WebSocket `/oocp`；鉴权见下文 |
| 角色包 | 与仓库 `roles/README_MANIFEST.md` 及校验 crate 一致；Windows 下打的包在 Linux 上同版本应行为一致 |

**不在本文范围**：桌面安装包、ASR/TTS/CV 进内核（须外挂进程，见 `LINUX_KERNEL_ENGINE.md`）。

---

## 2. 环境变量（必读）

### 2.1 路径三要素（生产必须显式设置）

| 变量 | 生产要求 | 未设置时的行为（仅开发/排障） |
|------|-----------|--------------------------------|
| **`OCLIVE_ROLES_DIR`** | 设为角色根目录绝对路径 | exe/cwd 启发式探测，不可靠 |
| **`OCLIVE_DB_PATH`** | 设为持久化卷上的 `.db` 文件 | 系统临时目录下按端口命名，易丢 |
| **`OCLIVE_APP_DATA_DIR`** | 设为可写目录 | 派生自 DB 父目录 |

**严格模式（推荐生产开启）**：设置 **`OCLIVE_REQUIRE_EXPLICIT_PATHS=1`**（`1` / `true` / `yes` / `on`）。  
任一上述变量未设置或为空时，**进程退出码 2**，避免误用临时目录或 cwd。

Docker 镜像 `Dockerfile.kernel-server` 已默认 `OCLIVE_REQUIRE_EXPLICIT_PATHS=1` 并注入三路径。

### 2.2 网络与安全

| 变量 | 默认 | 说明 |
|------|------|------|
| **`OOCP_API_PORT`** | `48888` | 监听端口 |
| **`OOCP_API_BIND`** | `127.0.0.1` | 仅本机；容器/内网监听常用 `0.0.0.0`，**必须**配合防火墙与 **`OOCP_API_TOKEN`** |
| **`OOCP_API_TOKEN`** | （空） | 非空时：`/chat`、`/role-feedback*` 与 OOCP WS 均需 `Authorization: Bearer <token>`；**`/health` 始终不要求鉴权** |

### 2.3 日志

| 变量 | 说明 |
|------|------|
| **`RUST_LOG`** | `error` / `warn` / `info` / `debug` / `trace` | 无头进程由 **`tracing-subscriber`** 解析（与 `log` 桥接）；级别与 target 建议见 **[LOGGING_GUIDE.md](./LOGGING_GUIDE.md)**。 |

### 2.4 XDG 路径建议（非代码强制）

裸机可将数据放在：

- `~/.local/share/oclive/`（数据）  
- `~/.local/state/oclive/`（状态）

通过 **`OCLIVE_DB_PATH`** / **`OCLIVE_APP_DATA_DIR`** 指到上述目录即可。

**完整变量清单与中文注释**：[`../delivery/config.example.env`](../delivery/config.example.env)。

---

## 3. 裸机快速启动

```bash
export OCLIVE_ROLES_DIR=/srv/oclive/roles
export OCLIVE_DB_PATH=/srv/oclive/data/oclive.db
export OCLIVE_APP_DATA_DIR=/srv/oclive/data/app
export OCLIVE_REQUIRE_EXPLICIT_PATHS=1
export OOCP_API_BIND=127.0.0.1
# export OOCP_API_TOKEN='your-secret'   # 若需鉴权

cargo build -p oclive_kernel_server --release
./target/release/oclive_kernel_server
```

验证：

```bash
curl -sS "http://127.0.0.1:${OOCP_API_PORT:-48888}/health"
# 期望：ok（本机探活请用 127.0.0.1；监听地址为 0.0.0.0 时表示所有接口均可连）
```

带鉴权的聊天（示例）：

```bash
curl -sS -H "Authorization: Bearer $OOCP_API_TOKEN" -H "Content-Type: application/json" \
  -d '{"role_path":"/srv/oclive/roles/shimeng","message":"你好"}' \
  "http://127.0.0.1:${OOCP_API_PORT:-48888}/chat"
```

`role_path` 须为**进程内可见**的目录路径（与 `OCLIVE_ROLES_DIR` 下子目录一致）。

---

## 4. Docker

### 4.1 单独构建镜像

在**仓库根目录**：

```bash
docker build -f Dockerfile.kernel-server -t oclive-kernel-server .
```

镜像特点：多阶段构建、Release **`strip`** 减小体积、**非 root** 用户 `oclive`、默认已设三路径与 **`OCLIVE_REQUIRE_EXPLICIT_PATHS=1`**。

### 4.2 Compose（推荐集成起点）

```bash
cp delivery/config.example.env .env
# 编辑 .env：OCLIVE_ROLES_HOST_PATH、生产请设 OOCP_API_TOKEN

docker compose -f delivery/docker-compose.yml up --build
```

数据卷 `oclive-kernel-data` 持久化 `/data`（数据库与 app 数据）；角色目录由宿主挂载为只读 **`/roles`**。

---

## 5. systemd

1. 参考 **`delivery/systemd/oclive-kernel.service.example`** 复制到 `/etc/systemd/system/oclive-kernel.service`。  
2. 准备 **`/etc/oclive/kernel.env`**（权限 `600`），内容可参考 **`delivery/config.example.env`** 中的裸机变量。  
3. 单元中 **`EnvironmentFile=-/etc/oclive/kernel.env`**：文件缺失时不失败；生产建议固定存在并 `chmod 600`。

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now oclive-kernel
journalctl -u oclive-kernel -f
```

单元内已配置 **`MemoryHigh` / `MemoryMax`**、**`Restart=always`**、**`NoNewPrivileges=true`** 等；可按机器内存调整 `MemoryMax`（嵌入式可降至 384M–512M）。

---

## 6. 运维脚本

| 脚本 | 用途 |
|------|------|
| [`../scripts/verify_deploy.sh`](../scripts/verify_deploy.sh) | **一键验收**：环境变量、角色目录与 DB 权限、端口监听、`GET /health` 与 **`GET /health?verbose=true`**（需 `python3`）；`OCLIVE_HEALTH_URL` / `OOCP_API_PORT` / **`OOCP_API_TOKEN`**（若启用鉴权） |
| [`../scripts/health_check.sh`](../scripts/health_check.sh) | `curl` 检查 **`/health`** 是否为 `ok`；`OCLIVE_HEALTH_URL` 可改基址 |
| [`../scripts/backup_kernel_db.sh`](../scripts/backup_kernel_db.sh) | 备份 **`OCLIVE_DB_PATH`**；优先 **`sqlite3 .backup`**；`OCLIVE_BACKUP_KEEP_DAYS` 清理旧文件 |

裸机示例（内核已启动且变量已导出）：

```bash
chmod +x scripts/verify_deploy.sh
./scripts/verify_deploy.sh
```

cron 示例见各脚本头部注释。

---

## 7. 与 CI 对齐

仓库 **`.github/workflows/ci.yml`** 在 Ubuntu 上包含：

- 全 workspace 测试（与其它 OS 矩阵）；  
- 专用任务 **`cargo test -p oclive_kernel_runtime --features kernel-http-api`**；  
- **`cargo build -p oclive_kernel_server --release`**。

发版前建议本地至少执行：

```bash
cargo test -p oclive_kernel_runtime --features kernel-http-api
cargo build -p oclive_kernel_server --release
```

---

## 8. 静态存储加密（v1 路线）

**应用层 SQLCipher**：当前 **`oclive_kernel_runtime` / `sqlx` 栈未集成 SQLCipher**；在嵌入式玩偶等场景若需「设备丢失后难以直接 strings 出对话」，优先在 **块设备或文件系统层** 做整卷加密，而非在应用内二次封装 SQLite。

| 方案 | 说明 |
|------|------|
| **LUKS**（磁盘 / 镜像） | 整机或独立数据分区加密，适合固定设备镜像交付。 |
| **fscrypt**（目录级） | 将 **`OCLIVE_DB_PATH`** / **`OCLIVE_APP_DATA_DIR`** 所在目录置于加密目录树（内核密钥与用户登录绑定）。 |
| **容器 / 宿主卷** | Docker / k8s 将数据卷挂到宿主已加密路径。 |

后续若在 Cargo 增加可选 **`db-encryption`** 并接 SQLCipher，须单独评估 `sqlx` 迁移、跨平台链接与密钥托管；在此之前以本表为集成方默认指引。**Python OOCP 客户端**见仓库 [`sdk/python/README.md`](../sdk/python/README.md)。

---

## 9. 相关文档与示例

- 特性裁剪（嵌入式体积）：[`../creator-docs/kernel/LIGHTWEIGHT_PROFILE.md`](../creator-docs/kernel/LIGHTWEIGHT_PROFILE.md)  
- 远程 HTTP 试聊示例（Linux 步骤）：[`../examples/kernel_remote_simple/README.md`](../examples/kernel_remote_simple/README.md)  
- 内核 crate 说明：[`../crates/oclive_kernel_server/README.md`](../crates/oclive_kernel_server/README.md)

---

## 10. 故障排查简表

| 现象 | 检查 |
|------|------|
| 启动即退出码 2 | 是否 `OCLIVE_REQUIRE_EXPLICIT_PATHS=1` 且三路径未齐 |
| 连接被拒绝 | `OOCP_API_BIND` / 防火墙 / 端口映射 |
| 401 / 403 | 是否设置 `OOCP_API_TOKEN` 但未带 `Authorization: Bearer` |
| 角色加载失败 | `role_path` 是否为目录且含合法 `manifest.json`；`OCLIVE_ROLES_DIR` 是否正确 |
