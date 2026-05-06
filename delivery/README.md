# Oclive Linux 内核 — 交付与集成速览

本目录为 **机器人 / Linux 无头场景** 的合成模板，与桌面版（`src-tauri`）无关。

## 内含文件

| 文件 | 说明 |
|------|------|
| `docker-compose.yml` | 构建并启动 `Dockerfile.kernel-server`，挂载角色与持久化数据卷 |
| `config.example.env` | 环境变量示例；复制为 `.env` 后由 compose 读取 |
| `systemd/oclive-kernel.service.example` | systemd 单元模板（裸机安装） |

完整阶段说明、安全与 ARM：**[`docs/LINUX_KERNEL_ENGINE.md`](../docs/LINUX_KERNEL_ENGINE.md)**。

## 快速启动（Docker）

在 **仓库根目录**：

```bash
cp delivery/config.example.env .env
# 编辑 .env：至少确认 OCLIVE_ROLES_HOST_PATH；生产请设置 OOCP_API_TOKEN
docker compose -f delivery/docker-compose.yml up --build
```

健康检查：

```bash
curl -s http://127.0.0.1:48888/health
```

若设置了 `OOCP_API_TOKEN`：

```bash
curl -s http://127.0.0.1:48888/health
curl -s -H "Authorization: Bearer $OOCP_API_TOKEN" -H "Content-Type: application/json" \
  -d "{\"role_path\":\"/roles/shimeng\",\"message\":\"你好\"}" \
  http://127.0.0.1:48888/chat
```

（容器内示例路径 `/roles/shimeng` 对应挂载卷；请按实际角色目录调整。）

## English (for partners)

- **Kernel binary**: `oclive_kernel_server` — HTTP `GET /health`, `POST /chat`, OOCP WebSocket `/oocp`.
- **Contract**: Same as Windows desktop kernel; role packs under `roles/` (see `roles/README_MANIFEST.md`).
- **Security**: Default local bind is `127.0.0.1`; Docker defaults to `0.0.0.0` — set **`OOCP_API_TOKEN`** and use TLS/reverse proxy in production.
