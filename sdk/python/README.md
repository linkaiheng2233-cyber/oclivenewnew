# Python OOCP 客户端（最小 SDK）

面向 **`oclive_kernel_server`** 或 **`oclive_kernel_runtime` HTTP API** 的轻量调用封装：标准库 **`urllib`**，无强制运行时 pip 依赖；安装为包后便于版本固定与测试。

## 安装

**可编辑安装**（开发推荐，路径换成本机克隆位置）：

```bash
pip install -e "D:/path/to/oclivenewnew/sdk/python[dev]"
```

或 **普通安装**（从仓库根）：

```bash
pip install "./sdk/python"
```

仅引用源码而不安装时，可将 **`sdk/python`** 加入 **`PYTHONPATH`**。

**运行单元测试**（需已安装 **`[dev]`** 依赖中的 pytest）：

```bash
cd sdk/python && python -m pytest tests -q
```

## 快速用法

```python
from oclive import OcliveClient

with OcliveClient(base_url="http://127.0.0.1:48888", bearer_token=None) as c:
    assert c.health() == "ok"
    out = c.chat(
        role_path="/srv/oclive/roles/shimeng",
        message="你好",
        session_id=None,
        scene_id=None,
    )
    print(out["reply"])
```

若设置了 **`OOCP_API_TOKEN`**，传入 **`bearer_token=...`** 以发送 **`Authorization: Bearer`**。

## 常见问题

| 现象 | 处理 |
|------|------|
| **`ModuleNotFoundError: oclive`** | 先执行 **`pip install -e ./sdk/python`** 或设置 **`PYTHONPATH`** 指向 **`sdk/python`** 的父目录。 |
| **`OcliveError: HTTP 401`** | 服务端启用了 **`OOCP_API_TOKEN`** 时，客户端必须传 **`bearer_token`**；探活 **`health()`** 一般仍可用。 |
| **连接超时 / 拒绝** | 确认 **`oclive_kernel_server`** 已启动，**`base_url`** 与 **`OOCP_API_PORT`** / **`OOCP_API_BIND`** 一致。 |
| **响应缺 `reply`** | 服务端非 2xx 或返回非 JSON 时会抛 **`OcliveError`**；用同一仓库的 **`examples/kernel_remote_simple`** 对照请求体。 |

## API 摘要

| 成员 | 说明 |
|------|------|
| **`health()`** | `GET /health`，返回纯文本。 |
| **`health_verbose()`** | `GET /health?verbose=true`，JSON。 |
| **`health_db()`** | `GET /health/db`，JSON。 |
| **`chat(role_path=..., message=..., session_id=..., scene_id=...)`** | `POST /chat`，返回完整 JSON（含 **`reply`**）。 |
| **`max_retries` / `retry_backoff_s`** | 对 **`URLError`**（连接失败等）的简单重试。 |

更完整的 curl 示例见 **[`examples/kernel_remote_simple/README.md`](../../examples/kernel_remote_simple/README.md)**。
