# Python OOCP 客户端（最小 SDK）

面向 **`oclive_kernel_server`** 或 **`oclive_kernel_runtime` HTTP API** 的轻量调用封装：标准库 **`urllib`**，无强制 pip 依赖（可选安装为包以便其它项目引用）。

## 安装

在仓库根或任意项目内将 **`sdk/python`** 加入 `PYTHONPATH`，或使用可编辑安装：

```bash
pip install -e /path/to/oclivenewnew/sdk/python
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

## API 摘要

| 成员 | 说明 |
|------|------|
| **`health()`** | `GET /health`，返回纯文本。 |
| **`health_verbose()`** | `GET /health?verbose=true`，JSON。 |
| **`health_db()`** | `GET /health/db`，JSON。 |
| **`chat(role_path=..., message=..., session_id=..., scene_id=...)`** | `POST /chat`，返回完整 JSON（含 **`reply`**）。 |
| **`max_retries` / `retry_backoff_s`** | 对 **`URLError`**（连接失败等）的简单重试。 |

更完整的 curl 示例见 **[`examples/kernel_remote_simple/README.md`](../../examples/kernel_remote_simple/README.md)**。
